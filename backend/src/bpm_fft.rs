use std::collections::VecDeque;
use rustfft::{FftPlanner, num_complex::Complex};

pub struct FFTBPMDetector {
    samples: VecDeque<f32>,
    sample_rate: f32, // Samples per second
    window_size: usize, // Samples in the FFT window
}

impl FFTBPMDetector {
    pub const fn new(sample_rate: f32, window_seconds: f32) -> Self {
        let window_size = (sample_rate * window_seconds) as usize;
        
        Self {
            samples: VecDeque::new(),
            sample_rate,
            window_size,
        }
    }

    pub fn add_accelerometer_data(&mut self, x: i16, y: i16, z: i16) {
        let x = x as f32 / 512.0;
        let y = y as f32 / 512.0;
        let z = z as f32 / 512.0;
        let magnitude = (x*x + y*y + z*z).sqrt();
        self.samples.push_back(magnitude);
        
        if self.samples.len() > self.window_size {
            self.samples.pop_front();
        }
    }

    pub fn get_current_bpm(&self) -> f32 {
        if self.samples.len() < self.window_size {
            return 0.0;
        }

        let n = self.window_size;

        let samples: Vec<f32> = self.samples.iter().copied().collect();
        let mean = samples.iter().sum::<f32>() / n as f32;

        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let window = 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / (n as f32 - 1.0)).cos();
                Complex {
                    re: (sample - mean) * window,
                    im: 0.0,
                }
            })
            .collect();

        // FFT berechnen
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(n);
        fft.process(&mut buffer);

        let min_bpm = 10.0;
        let max_bpm = 55.0;

        let min_freq = min_bpm / 60.0;
        let max_freq = max_bpm / 60.0;

        let min_bin = ((min_freq * n as f32) / self.sample_rate).ceil() as usize;
        let max_bin = ((max_freq * n as f32) / self.sample_rate).floor() as usize;


        let nyquist_bin = n / 2;
        let max_bin = max_bin.min(nyquist_bin);

        if min_bin >= max_bin || max_bin >= buffer.len() {
            return 0.0;
        }

        let mut best_bin = min_bin;
        let mut best_power = 0.0_f32;

        for bin in min_bin..=max_bin {
            let c = buffer[bin];

            let power = c.re * c.re + c.im * c.im;

            if power > best_power {
                best_power = power;
                best_bin = bin;
            }
        }

        if best_power <= 0.0 {
            return 0.0;
        }

        let interpolated_bin = if best_bin > 0 && best_bin < nyquist_bin {
            let power_at = |bin: usize| {
                let c = buffer[bin];
                c.re * c.re + c.im * c.im
            };

            let left = power_at(best_bin - 1);
            let center = power_at(best_bin);
            let right = power_at(best_bin + 1);

            let denominator = left - 2.0 * center + right;

            if denominator.abs() > f32::EPSILON {
                let delta = 0.5 * (left - right) / denominator;
                best_bin as f32 + delta.clamp(-0.5, 0.5)
            } else {
                best_bin as f32
            }
        } else {
            best_bin as f32
        };

        let frequency_hz = interpolated_bin * self.sample_rate / n as f32;
        let bpm = frequency_hz * 60.0;

        bpm
    }
}