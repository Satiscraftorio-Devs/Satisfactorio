use shared::{network::messages::Position, world::constants::WALK_SPEED};

use crate::world::WorldState;

pub fn is_movement_plausible(old: &Position, new: &Position, dt_sec: f32) -> bool {
    let dx = new.x - old.x;
    let dy = new.y - old.y;
    let dz = new.z - old.z;
    let distance_sq = dx * dx + dy * dy + dz * dz;
    let expected_distance = WALK_SPEED * dt_sec * 2.0; // <=> Coeficient de tolérance
    distance_sq <= expected_distance * expected_distance
}
