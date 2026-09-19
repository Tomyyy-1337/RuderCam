use std::{collections::VecDeque, time::{Duration, Instant}};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(100);
const PEAK_HISTORY: Duration = Duration::from_secs(60);
const FILTER_ALPHA: f32 = 0.4;
const BASELINE_ALPHA: f32 = 0.02;
const NOISE_ALPHA: f32 = 0.05;
// The expected rowing range is 10-60 strokes per minute.
const MIN_PEAK_DISTANCE: Duration = Duration::from_secs(1);
const NOISE_MULTIPLIER: f32 = 1.2;
const MIN_SIGNAL_AMPLITUDE: f32 = 0.5;
const POLARITY_CONFIRMATION_SAMPLES: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AccelerationPhase {
    Unknown,
    Positive,
    Negative,
}

pub struct BpmPeak {
    filtered_sample: Option<f32>,
    baseline: Option<f32>,
    noise: f32,
    phase: AccelerationPhase,
    pending_phase: AccelerationPhase,
    pending_samples: u8,
    last_peak: Option<Instant>,
    peaks: VecDeque<Instant>,
    total_strokes: u32,
    next_sample: Instant,
    schläge_at_last_poll: u32,
}

impl Default for BpmPeak {
    fn default() -> Self {
        Self {
            filtered_sample: None,
            baseline: None,
            noise: 0.0,
            phase: AccelerationPhase::Unknown,
            pending_phase: AccelerationPhase::Unknown,
            pending_samples: 0,
            last_peak: None,
            peaks: VecDeque::new(),
            total_strokes: 0,
            next_sample: Instant::now(),
            schläge_at_last_poll: 0
        }
    }
}


impl BpmPeak {
    pub fn add_sample(&mut self, sample: i16) {
        let now = self.next_sample;
        self.next_sample += SAMPLE_INTERVAL;

        let sample = sample as f32;
        let filtered = match self.filtered_sample {
            Some(previous) => previous + FILTER_ALPHA * (sample - previous),
            None => sample,
        };
        self.filtered_sample = Some(filtered);

        let baseline = match self.baseline {
            Some(previous) => previous + BASELINE_ALPHA * (filtered - previous),
            None => filtered,
        };
        self.baseline = Some(baseline);

        let deviation = filtered - baseline;
        self.noise += NOISE_ALPHA * (deviation.abs() - self.noise);
        let threshold = (self.noise * NOISE_MULTIPLIER).max(MIN_SIGNAL_AMPLITUDE);

        let detected_phase = if deviation > threshold {
            AccelerationPhase::Positive
        } else if deviation < -threshold {
            AccelerationPhase::Negative
        } else {
            AccelerationPhase::Unknown
        };

        if detected_phase == AccelerationPhase::Unknown {
            // Brief zero crossings are common in a weak signal. Do not discard
            // a pending polarity change because of one such sample.
        } else if detected_phase == self.pending_phase {
            self.pending_samples = self.pending_samples.saturating_add(1);
        } else {
            self.pending_phase = detected_phase;
            self.pending_samples = 1;
        }

        if self.pending_samples >= POLARITY_CONFIRMATION_SAMPLES
            && self.phase != self.pending_phase
        {
            let previous_phase = self.phase;
            self.phase = self.pending_phase;

            if previous_phase == AccelerationPhase::Negative
                && self.phase == AccelerationPhase::Positive
                && self.last_peak.is_none_or(|last| now.duration_since(last) >= MIN_PEAK_DISTANCE)
            {
                self.peaks.push_back(now);
                self.last_peak = Some(now);
                self.total_strokes += 1;
            }
        }

        self.remove_old_peaks(now);
    }

    pub fn get_incremental_schläge(&mut self) -> u32 {
        let incr = self.total_strokes.saturating_sub(self.schläge_at_last_poll);
        self.schläge_at_last_poll = self.total_strokes;
        incr
    }

    pub fn get_current_schläge_pro_minute(&mut self) -> f32 {
        self.remove_old_peaks(self.next_sample);
        self.peaks.len() as f32
    }

    fn remove_old_peaks(&mut self, now: Instant) {
        while self.peaks.front().is_some_and(|peak| now.duration_since(*peak) > PEAK_HISTORY) {
            self.peaks.pop_front();
        }
    }
}