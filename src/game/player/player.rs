use std::f32::consts::PI;

use crate::engine::render::camera::Camera;
use crate::game::player::camera::CameraController;
use crate::game::world::chunk::CHUNK_SIZE;
use cgmath::{num_traits::ToPrimitive, InnerSpace, Point3, Vector3};

const DEBUG_HORIZONTAL_RENDER_DISTANCE: u16 = 9;
const DEBUG_VERTICAL_RENDER_DISTANCE: u16 = 7;

pub struct Player {
    uuid: i32,
    pub pos: cgmath::Point3<f32>,
    pub vel: cgmath::Vector3<f32>,
    yaw: f32,
    pub horizontal_render_distance: u16,
    pub vertical_render_distance: u16,
}

impl Player {
    pub fn new() -> Player {
        return Player {
            uuid: -1,
            pos: cgmath::Point3::new(0.0, 0.0, 0.0),
            vel: cgmath::Vector3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            horizontal_render_distance: DEBUG_HORIZONTAL_RENDER_DISTANCE,
            vertical_render_distance: DEBUG_VERTICAL_RENDER_DISTANCE,
        };
    }

    pub fn update(&mut self, dt: f32, camera: &mut Camera, camera_controller: &mut CameraController) {
        let forward = camera.forward();
        let right = camera.right();
        let up = cgmath::Vector3::unit_y();
        let mut direction = Vector3::new(0.0, 0.0, 0.0);

        if camera_controller.is_forward_pressed {
            direction += forward;
        }
        if camera_controller.is_backward_pressed {
            direction -= forward
        }
        if camera_controller.is_right_pressed {
            direction += right;
        }
        if camera_controller.is_left_pressed {
            direction -= right;
        }
        if camera_controller.is_up_pressed {
            direction += up;
        }
        if camera_controller.is_down_pressed {
            direction -= up;
        }

        if direction.magnitude2() > 0.0 {
            self.vel = direction.normalize() * (camera_controller.speed * dt);
        } else {
            self.vel = Vector3::new(0.0, 0.0, 0.0);
        }

        self.pos += self.vel;
        self.yaw = camera.yaw % (2.0 * PI);

        camera_controller.update_camera(camera, self);
    }

    pub fn set_render_distance(&mut self, horizontal: u16, vertical: u16) {
        self.horizontal_render_distance = horizontal;
        self.vertical_render_distance = vertical;
    }

    pub fn get_pos(&self) -> cgmath::Point3<f32> {
        self.pos
    }

    pub fn set_pos(&mut self, pos: cgmath::Point3<f32>) {
        self.pos = pos;
    }

    pub fn teleport(&mut self, x: f32, y: f32, z: f32) {
        println!(
            "The player {} has been teleported from {:?} to {:?}",
            self.uuid,
            self.get_pos(),
            Point3 { x: x, y: y, z: z }
        );
        self.set_pos(Point3 { x: x, y: y, z: z });
    }

    pub fn break_block_at(_block_pos: Point3<f32>) {}

    pub fn get_rendered_chunk_range(&self) -> [i32; 6] {
        let halfed_hrd = self.horizontal_render_distance.to_f32().unwrap().div_euclid(2.0);
        let halfed_vrd = self.vertical_render_distance.to_f32().unwrap().div_euclid(2.0);

        let cx = self.pos.x.div_euclid(CHUNK_SIZE as f32);
        let cy = self.pos.y.div_euclid(CHUNK_SIZE as f32);
        let cz = self.pos.z.div_euclid(CHUNK_SIZE as f32);

        let min_cx = (cx - halfed_hrd).floor().to_i32().unwrap();
        let max_cx = (cx + halfed_hrd).floor().to_i32().unwrap();
        let min_cy = (cy - halfed_vrd).floor().to_i32().unwrap();
        let max_cy = (cy + halfed_vrd).floor().to_i32().unwrap();
        let min_cz = (cz - halfed_hrd).floor().to_i32().unwrap();
        let max_cz = (cz + halfed_hrd).floor().to_i32().unwrap();

        return [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz];
    }

    pub fn get_rendered_chunk_data(&self) -> ([i32; 6], u32) {
        let halfed_hrd = self.horizontal_render_distance.to_f32().unwrap().div_euclid(2.0);
        let halfed_vrd = self.vertical_render_distance.to_f32().unwrap().div_euclid(2.0);

        let cx = self.pos.x.div_euclid(CHUNK_SIZE as f32);
        let cy = self.pos.y.div_euclid(CHUNK_SIZE as f32);
        let cz = self.pos.z.div_euclid(CHUNK_SIZE as f32);

        let min_cx = (cx - halfed_hrd).floor().to_i32().unwrap();
        let max_cx = (cx + halfed_hrd).floor().to_i32().unwrap();
        let min_cy = (cy - halfed_vrd).floor().to_i32().unwrap();
        let max_cy = (cy + halfed_vrd).floor().to_i32().unwrap();
        let min_cz = (cz - halfed_hrd).floor().to_i32().unwrap();
        let max_cz = (cz + halfed_hrd).floor().to_i32().unwrap();

        let chunk_number = ((max_cx - min_cx) * (max_cy - min_cy) * (max_cz - min_cz)).to_u32().unwrap_or(1);

        return ([min_cx, max_cx, min_cy, max_cy, min_cz, max_cz], chunk_number);
    }
}
