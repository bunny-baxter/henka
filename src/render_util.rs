use std::collections::VecDeque;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

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
