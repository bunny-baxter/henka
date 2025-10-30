use std::collections::VecDeque;

pub struct MovingAverage {
    window_size: usize,
    samples: VecDeque<f64>,
}

impl MovingAverage {
    pub fn new(window_size: usize) -> Self {
        MovingAverage {
            window_size,
            samples: VecDeque::new(),
        }
    }

    pub fn add_sample(&mut self, sample: f64) {
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn get_average(&self) -> f64 {
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }
}
