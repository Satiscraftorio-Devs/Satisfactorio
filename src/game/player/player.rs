use crate::{engine::render::camera::{Camera, CameraUniform}, game::{player::camera::CameraController, world::chunk::CHUNK_SIZE}};
use cgmath::{InnerSpace, Vector3, num_traits::ToPrimitive};
use wgpu::{Buffer, Queue};

/// Must be odd for semantic reasons (otherwise it will render one chunk more than this value)
const DEBUG_HORIZONTAL_RENDER_DISTANCE: u16 = 1;
/// Must be odd for semantic reasons (otherwise it will render one chunk more than this value)
const DEBUG_VERTICAL_RENDER_DISTANCE: u16 = 3;

pub struct Player {
    pub pos: cgmath::Point3<f32>,
    pub vel: cgmath::Vector3<f32>,
    pub horizontal_render_distance: u16,
    pub vertical_render_distance: u16,
}

impl Player {
    pub fn new() -> Player {
        return Player {
            pos: cgmath::Point3::new(0.0, 0.0, 0.0),
            vel: cgmath::Vector3::new(0.0, 0.0, 0.0),
            horizontal_render_distance: DEBUG_HORIZONTAL_RENDER_DISTANCE,
            vertical_render_distance: DEBUG_VERTICAL_RENDER_DISTANCE,
        };
    }

    pub fn set_render_distance(&mut self, horizontal: u16, vertical: u16) {
        self.horizontal_render_distance = horizontal;
        self.vertical_render_distance = vertical;
    }

    pub fn update(&mut self, dt: f32, camera: &mut Camera, camera_controller: &mut CameraController, camera_uniform: &mut CameraUniform, camera_buffer: &Buffer, queue: &Queue) {
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
        }
        else {
            self.vel = Vector3::new(0.0, 0.0, 0.0);
        }

        self.pos += self.vel;

        camera_controller.update_camera(camera, &self);
        camera_uniform.update_view_proj(&camera);

        // println!("Player pos: ({:.2}, {:.2}, {:.2})", self.pos.x, self.pos.y, self.pos.z);
        // println!("Camera pos: ({:.2}, {:.2}, {:.2})", camera.eye.x, camera.eye.y, camera.eye.z);
        // println!("Camera forward: ({:.2}, {:.2}, {:.2})", camera.forward().x, camera.forward().y, camera.forward().z);
        
        queue.write_buffer(
            &camera_buffer,
            0,
            bytemuck::cast_slice(&[*camera_uniform]),
        );
    }

    pub fn get_pos(&self) -> cgmath::Point3<f32> {
        self.pos
    }

    /// returns ``[min_cx, max_cx, min_cy, max_cy, min_cz, max_cz]``
    pub fn get_rendered_chunk_range(&self) -> [i32; 6] {
        let halfed_hrd = self
            .horizontal_render_distance
            .to_f32()
            .unwrap()
            .div_euclid(2.0);
        let halfed_vrd = self
            .vertical_render_distance
            .to_f32()
            .unwrap()
            .div_euclid(2.0);

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

    pub fn get_rendered_chunk_number(&self) -> u32 {
        let [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz] = self.get_rendered_chunk_range();
        return ((max_cx - min_cx) * (max_cy - min_cy) * (max_cz - min_cz))
            .to_u32()
            .unwrap_or(1);
    }

    /// returns ``([min_cx, max_cx, min_cy, max_cy, min_cz, max_cz, chunk_number], chunk_number)``
    pub fn get_rendered_chunk_data(&self) -> ([i32; 6], u32) {
        let halfed_hrd = self
            .horizontal_render_distance
            .to_f32()
            .unwrap()
            .div_euclid(2.0);
        let halfed_vrd = self
            .vertical_render_distance
            .to_f32()
            .unwrap()
            .div_euclid(2.0);

        let cx = self.pos.x.div_euclid(CHUNK_SIZE as f32);
        let cy = self.pos.y.div_euclid(CHUNK_SIZE as f32);
        let cz = self.pos.z.div_euclid(CHUNK_SIZE as f32);

        let min_cx = (cx - halfed_hrd).floor().to_i32().unwrap();
        let max_cx = (cx + halfed_hrd).floor().to_i32().unwrap();
        let min_cy = (cy - halfed_vrd).floor().to_i32().unwrap();
        let max_cy = (cy + halfed_vrd).floor().to_i32().unwrap();
        let min_cz = (cz - halfed_hrd).floor().to_i32().unwrap();
        let max_cz = (cz + halfed_hrd).floor().to_i32().unwrap();

        let chunk_number = ((max_cx - min_cx) * (max_cy - min_cy) * (max_cz - min_cz))
            .to_u32()
            .unwrap_or(1);

        return ([min_cx, max_cx, min_cy, max_cy, min_cz, max_cz], chunk_number);
    }
}
