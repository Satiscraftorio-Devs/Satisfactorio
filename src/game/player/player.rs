use crate::{
    engine::{core::inputs::InputState, render::camera::Camera},
    game::{player::camera::CameraController, world::chunk::CHUNK_SIZE},
};
use cgmath::{num_traits::ToPrimitive, Point3};

/// Must be odd for semantic reasons (otherwise it will render one chunk more than this value)
const DEBUG_HORIZONTAL_RENDER_DISTANCE: u16 = 1;
/// Must be odd for semantic reasons (otherwise it will render one chunk more than this value)
const DEBUG_VERTICAL_RENDER_DISTANCE: u16 = 1;

pub struct Player {
    pub position: Point3<f32>,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub horizontal_render_distance: u16,
    pub vertical_render_distance: u16,
}

impl Player {
    pub fn new() -> Player {
        return Player {
            position: Point3::new(0.0, 0.0, 0.0),
            camera: Camera::new(Point3::new(0.0, 1.0, 0.0), 70.0),
            camera_controller: CameraController::new(32.0, 0.015),
            horizontal_render_distance: DEBUG_HORIZONTAL_RENDER_DISTANCE,
            vertical_render_distance: DEBUG_VERTICAL_RENDER_DISTANCE,
        };
    }

    pub fn update(&mut self, dt: f32, inputs: &InputState) {
        self.camera_controller.update_camera(dt, &mut self.camera, inputs);
        self.position = self.camera.position;
        // println!("Player position: ({:.2}, {:.2}, {:.2})", self.position.x, self.position.y, self.position.z);
        // println!("Camera yaw/pitch: {:.2} {:.2}", self.camera.yaw, self.camera.pitch);
    }

    /// returns ``[min_cx, max_cx, min_cy, max_cy, min_cz, max_cz]``
    pub fn get_rendered_chunk_range(&self) -> [i32; 6] {
        let halfed_hrd = self.horizontal_render_distance.to_f32().unwrap().div_euclid(2.0);
        let halfed_vrd = self.vertical_render_distance.to_f32().unwrap().div_euclid(2.0);

        let cx = self.position.x.div_euclid(CHUNK_SIZE as f32);
        let cy = self.position.y.div_euclid(CHUNK_SIZE as f32);
        let cz = self.position.z.div_euclid(CHUNK_SIZE as f32);

        let min_cx = (cx - halfed_hrd).floor().to_i32().unwrap();
        let max_cx = (cx + halfed_hrd).floor().to_i32().unwrap();
        let min_cy = (cy - halfed_vrd).floor().to_i32().unwrap();
        let max_cy = (cy + halfed_vrd).floor().to_i32().unwrap();
        let min_cz = (cz - halfed_hrd).floor().to_i32().unwrap();
        let max_cz = (cz + halfed_hrd).floor().to_i32().unwrap();

        return [min_cx, max_cx, min_cy, max_cy, min_cz, max_cz];
    }

    /// returns ``([min_cx, max_cx, min_cy, max_cy, min_cz, max_cz], chunk_number)``
    pub fn get_rendered_chunk_data(&self) -> ([i32; 6], u32) {
        let halfed_hrd = self.horizontal_render_distance.to_f32().unwrap().div_euclid(2.0);
        let halfed_vrd = self.vertical_render_distance.to_f32().unwrap().div_euclid(2.0);

        let cx = self.position.x.div_euclid(CHUNK_SIZE as f32);
        let cy = self.position.y.div_euclid(CHUNK_SIZE as f32);
        let cz = self.position.z.div_euclid(CHUNK_SIZE as f32);

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
