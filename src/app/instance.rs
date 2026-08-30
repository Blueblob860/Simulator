use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;

pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4]
}

impl From<Instance> for InstanceRaw {
    fn from(value: Instance) -> Self {
        Self {
            model: (cgmath::Matrix4::from_translation(value.position) * cgmath::Matrix4::from(value.rotation)).into()
        }
    }
}