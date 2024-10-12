use bevy::prelude::*;

pub fn move_player(keyboard_input: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut query: Query<&mut Transform, With<Camera>>) {
    for mut transform in query.iter_mut() {

        let mut direction = Vec3::ZERO;
        let mut speed = 0.03125;

        if keyboard_input.pressed(KeyCode::ControlLeft){
            speed = 0.125;
        }

        // direction avant-arrière:
        // (les customs transforms servent éviter de controller l'altitude avec les touches Z et S 
        // comme dans minecraft en créatif plutôt qu'en spectateur)
        if keyboard_input.pressed(KeyCode::KeyW) {
            let mut custom_transform = *transform.forward();
            custom_transform.y = 0.0;
            direction += custom_transform.abs() * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            let mut custom_transform = *transform.forward();
            custom_transform.y = 0.0;
            direction -= custom_transform.abs() * time.delta_seconds();
        }
        // Direction gauche-droite
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *transform.right() * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *transform.right() * time.delta_seconds();  
        }

        // Direction haut-bas
        if keyboard_input.pressed(KeyCode::ShiftLeft){
            direction.y -= 1.5 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Space){
            direction.y += 1.5 * time.delta_seconds();
        }

        if direction.length() > 0.0 {
            direction = direction.normalize() * speed;
        }

        transform.translation += direction;

    }
}