use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Debug, Resource)]
pub struct CameraRotRelative(pub f32);


pub fn move_camera(mut mouse_input: EventReader<MouseMotion>, mut query: Query<&mut Transform, With<Camera>>, mut camera_rot_relative: ResMut<CameraRotRelative>) {
    let sensitivity = 0.00048828125;
    let pi_sur_deux = 1.570796326794897;
    let mut rot_x = 0.00;
    for mut transform in query.iter_mut() {
        // Orientation Caméra
        for ev in mouse_input.read() {
            // Axe Vertical
            rot_x = -ev.delta.y * sensitivity;
            if camera_rot_relative.0 > pi_sur_deux{
                rot_x = -0.001;
            } 
            else if camera_rot_relative.0 < -pi_sur_deux{
                rot_x = 0.001;
            }
            camera_rot_relative.0 += rot_x;
            transform.rotate_local_x(rot_x);

            // Axe Horizontal
            transform.rotate_y(-ev.delta.x * sensitivity);
        }

    }
}

