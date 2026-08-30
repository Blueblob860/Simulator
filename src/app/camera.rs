use core::f32;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use cgmath::*;
use cgmath::Vector3 as vec3;
use cgmath::Vector2 as vec2;
use cgmath::Matrix4 as mat4;
use cgmath::Matrix3 as mat3;

pub struct Camera {
    pub pos: vec3<f32>,
    pub dir: vec3<f32>,
    pub vel: vec3<f32>,
    pub last_update: Instant,
    pub aspect: f32,
    pub fovy: Rad<f32>,
    pub near: f32,
    pub far: f32,
    pub uniform: CameraUniform
}

impl Camera {
    #[rustfmt::skip]
    pub const OPENGL_PROJ_TO_WGPU: mat4<f32> = mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    );

    pub fn new(pos: vec3<f32>, dir: vec3<f32>, width: u32, height: u32, fovy: f32, near: f32, far: f32) -> Self {
        let mut out = Self {
            pos, dir,
            vel: vec3::new(0., 0., 0.),
            last_update: Instant::now(), 
            aspect: width as f32 / height as f32,
            fovy: Rad(fovy.to_radians()),
            near, far,
            uniform: CameraUniform::new()
        };
        out.uniform = CameraUniform::from_camera(&out);
        out
    }

    pub fn update(&mut self, keys: (bool, bool, bool, bool, bool, bool), mouse_dt: vec2<f32>, mouse_down: bool) {
        let now = Instant::now();
        let elapsed = self.last_update.elapsed().as_secs_f32();
        self.last_update = now;

        if mouse_down {
            self.dir += vec3(
                mouse_dt.y.to_degrees() * f32::consts::PI,
                mouse_dt.x.to_degrees() * f32::consts::TAU,
                0.0,
            );
        }

        let rot = mat3::from_angle_y(Deg(self.dir.y));
        let fwd = rot * vec3(0.0, 0.0, 1.0);
        let right = rot * vec3(1.0, 0.0, 0.0);
        let mut new_vel = self.vel;
        new_vel -= vec3(new_vel.x.signum() * (5.0_f32 * elapsed).min(new_vel.x.abs()),
            new_vel.y.signum() * (5.0_f32 * elapsed).min(new_vel.y.abs()),
            new_vel.z.signum() * (5.0_f32 * elapsed).min(new_vel.z.abs())
        );
        if keys.0 { new_vel += fwd; } if keys.2 { new_vel -= fwd; }
        if keys.1 { new_vel -= right; } if keys.3 { new_vel += right; }
        if keys.4 { new_vel += vec3(0., 1., 0.); }
        if keys.5 { new_vel -= vec3(0., 1., 0.); }
        if new_vel.magnitude2() > 1.0 { new_vel /= new_vel.magnitude(); }
        self.vel = new_vel;
        self.pos += 0.2 * new_vel * elapsed;
        self.uniform.view = self.view_matrix().into();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
        self.uniform.proj = self.proj_matrix().into();
    }
    
    pub fn view_matrix(&self) -> mat4<f32> {
        let new_pos = vec3(-self.pos.x, -self.pos.y, self.pos.z);
        let translation = mat4::from_translation(new_pos);
        mat4::from_angle_x(Deg(self.dir.x))
             * mat4::from_angle_y(Deg(self.dir.y))
             * mat4::from_angle_z(Deg(self.dir.z))
             * translation
    }

    pub fn proj_matrix(&self) -> mat4<f32> {
        Self::OPENGL_PROJ_TO_WGPU * cgmath::perspective(self.fovy, self.aspect, self.near, self.far)
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct CameraUniform {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { view: Matrix4::identity().into(), proj: Matrix4::identity().into() }
    }

    pub fn from_camera(cam: &Camera) -> Self {
        let world = cam.view_matrix();
        let proj = cam.proj_matrix();
        Self { view: world.into(), proj: proj.into() }
    }
    
    pub fn update_view(&mut self, cam: &Camera) {
        let world = cam.view_matrix();
        self.view = world.into();
    }

    pub fn update_projection(&mut self, cam: &Camera) {
        let proj = cam.proj_matrix();
        self.proj = proj.into();
    }
}