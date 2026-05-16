pub const HORIZONTAL_RENDER_DISTANCE: u16 = 9;
pub const VERTICAL_RENDER_DISTANCE: u16 = 5;
pub const HORIZONTAL_SIMULATION_DISTANCE: u16 = HORIZONTAL_RENDER_DISTANCE + 2;
pub const VERTICAL_SIMULATION_DISTANCE: u16 = VERTICAL_RENDER_DISTANCE + 2;

pub const CHUNK_PRIORITY_DISTANCE: f32 = 32.0;
pub const CHUNK_PRIORITY_DISTANCE_SQR: f32 = CHUNK_PRIORITY_DISTANCE * CHUNK_PRIORITY_DISTANCE;

pub const fn max_chunks_in_queue() -> u32 {
    let h_chunks = HORIZONTAL_SIMULATION_DISTANCE as u32;
    let v_chunks = VERTICAL_SIMULATION_DISTANCE as u32;
    h_chunks * h_chunks * v_chunks
}

// Player physics

/// Demi-taille de la hitbox du joueur
pub const PLAYER_HALF_SIZE: f32 = 0.4;

/// Petit epsilon anti-interpénétration flottante
pub const COLLISION_EPSILON: f32 = 1e-3;

/// Gravité appliquée au joueur en survival (m/s²)
pub const GRAVITY: f32 = -25.0;

/// Vitesse de saut initiale (m/s)
pub const JUMP_SPEED: f32 = 8.0;

/// Vitesse de marche maximale (m/s)
pub const WALK_SPEED: f32 = 4.3;
