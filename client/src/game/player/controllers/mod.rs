use cgmath::Point3;

use crate::game::{player::camera::Camera, systems::inputs::InputState};

pub mod free;

pub trait PlayerController {
    fn update(&self, dt: f32, inputs: &mut InputState, player_pos: &mut Point3<f32>, camera: &Camera);
}

pub trait CameraController {
    fn update(&self, dt: f32, inputs: &mut InputState, camera: &mut Camera, player_pos: &Point3<f32>);
}
