use bevy::{input::mouse::MouseMotion, prelude::*};

pub fn move_camera(mut mouse_input: EventReader<MouseMotion>, mut query: Query<&mut Transform, With<Camera>>) {
    let sensitivity = 0.00048828125;

    for mut transform in query.iter_mut() {

        // Orientation Caméra
        for ev in mouse_input.read() {
            // Axe Vertical
            transform.rotate_local_x(-ev.delta.y * sensitivity);
            // Axe Horizontal
            transform.rotate_y(-ev.delta.x * sensitivity);
            
            // Debug pour la rotation
            // Il faut qu'on arrive à clamp l'axe vertical pour pas se faire un torticoli car là ça devient chaud miskine le joueur il meurt h24 😭😭😭
            // println!("X: {} Y: {} Z: {} W: {}", transform.rotation.x, transform.rotation.y, transform.rotation.z, transform.rotation.w/*, transform.local_y().z*/);
        }

    }
}

