use cgmath::{Deg, Matrix4, perspective, Point3, SquareMatrix, Vector3};

pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    up: Vector3<f32>,
    pub aspect_ratio: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    pub fn new(position: Point3<f32>, target: Point3<f32>, aspect_ratio: f32) -> Self {
        Camera {
            position: position,
            target: target,
            up: cgmath::Vector3::unit_y(),
            aspect_ratio: aspect_ratio,
            fov_y: 65.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.position, self.target, self.up);
        let proj = perspective(Deg(self.fov_y), self.aspect_ratio, self.z_near, self.z_far);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: Matrix4::identity().into(),
        }
    }

    pub fn set_view_projection(&mut self, matrix: Matrix4<f32>) {
        self.view_projection = matrix.into();
    }
}
