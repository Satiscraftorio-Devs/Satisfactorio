use cgmath::num_traits::ToPrimitive;

use crate::world::CHUNK_SIZE;

/// Must be odd for semantic reasons (otherwise it will render one chunk more than this value)
const DEBUG_HORIZONTAL_RENDER_DISTANCE: u16 = 9;
/// Must be odd for semantic reasons (otherwise it will render one chunk more than this value)
const DEBUG_VERTICAL_RENDER_DISTANCE: u16 = 7;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub horizontal_render_distance: u16,
    pub vertical_render_distance: u16,
}

impl Player {
    pub fn new() -> Player {
        return Player {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            horizontal_render_distance: DEBUG_HORIZONTAL_RENDER_DISTANCE, 
            vertical_render_distance: DEBUG_VERTICAL_RENDER_DISTANCE
        };
    }

    pub fn get_rendered_chunk_range(&self) -> [i32; 6] {
        let halfed_hrd = self.horizontal_render_distance.to_f32().unwrap().div_euclid(2.0);
        let halfed_vrd = self.vertical_render_distance.to_f32().unwrap().div_euclid(2.0);

        let cx = self.x.div_euclid(CHUNK_SIZE as f32);
        let cy = self.y.div_euclid(CHUNK_SIZE as f32);
        let cz = self.z.div_euclid(CHUNK_SIZE as f32);

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
        return ((max_cx - min_cx) * (max_cy - min_cy) * (max_cz - min_cz)).to_u32().unwrap_or(1);
    }
}