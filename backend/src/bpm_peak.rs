use std::{collections::VecDeque, time::Instant};


pub struct BpmPeak {
    sample_rate: f32,
    peak_count: usize,
    sample_index: usize,
    previous_previous: Option<f32>,
    previous: Option<f32>,
    baseline: Option<f32>,
    noise_level: f32,
    candidate_peak_index: Option<usize>,
    timestamps: VecDeque<Instant>,
    pub bpm: f32,
}

impl BpmPeak {
    pub fn new() -> Self {
        Self {
            sample_rate: 10.0,// Hz
            peak_count: 0,
            sample_index: 0,
            previous_previous: None,
            previous: None,
            baseline: None,
            noise_level: 0.0,
            candidate_peak_index: None,
            timestamps: VecDeque::new(),
            bpm: 0.0,
        }
    }

    /// Inserts one sample and returns true only for a peak in the 10..=50 BPM range.
    pub fn new_data_point(&mut self, data_point: f32) -> bool {
        if !data_point.is_finite() {
            return false;
        }

        let baseline = match self.baseline {
            Some(baseline) => baseline * 0.98 + data_point * 0.02,
            None => data_point,
        };
        self.baseline = Some(baseline);
        let signal = data_point - baseline;
        let peak_index = self.sample_index.saturating_sub(1);
        let is_local_maximum = match (self.previous_previous, self.previous) {
            (Some(left), Some(center)) => {
                center > left
                    && center >= signal
                    && center - left >= Self::minimum_prominence(self.noise_level)
                    && center - signal >= Self::minimum_prominence(self.noise_level)
            }
            _ => false,
        };

        let confirmed = if is_local_maximum {
            match self.candidate_peak_index {
                Some(previous_peak) => {
                    let distance = peak_index - previous_peak;
                    let min_distance = (self.sample_rate * 60.0 / 50.0).ceil() as usize;
                    let max_distance = (self.sample_rate * 60.0 / 10.0).floor() as usize;

                    if (min_distance..=max_distance).contains(&distance) {
                        self.peak_count += 1;
                        self.candidate_peak_index = Some(peak_index);
                        self.timestamps.push_back(Instant::now());
                        if self.timestamps.len() > 3 {
                            self.timestamps.pop_front();
                        }
                        self.bpm = 30.0 / ((self.timestamps.back().unwrap().duration_since(*self.timestamps.front().unwrap()).as_secs_f32()) / (self.timestamps.len() - 1) as f32);
                        true
                    } else if distance > max_distance {
                        self.candidate_peak_index = Some(peak_index);
                        false
                    } else {
                        false
                    }
                }
                None => {
                    self.candidate_peak_index = Some(peak_index);
                    false
                }
            }
        } else {
            false
        };

        if let Some(previous) = self.previous {
            let sample_difference = (signal - previous).abs();
            self.noise_level = self.noise_level * 0.95 + sample_difference * 0.05;
        }

        self.previous_previous = self.previous;
        self.previous = Some(signal);
        self.sample_index += 1;
        confirmed
    }

    fn minimum_prominence(noise_level: f32) -> f32 {
        0.5_f32.max(noise_level * 2.0)
    }
}
