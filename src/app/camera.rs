use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32;4];4],
}

pub struct Camera {
    pub position: Vec3,
    pub up: Vec3,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub yaw: f32,
    pub pitch: f32,
}   


impl Camera {

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(
            self.position,
            self.position + self.direction(),
            self.up,
        );

        let projection = Mat4::perspective_rh(
            self.fovy,
            self.aspect,
            self.znear,
            self.zfar,
        );

        projection * view
    }

    
    pub fn direction(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }
}
