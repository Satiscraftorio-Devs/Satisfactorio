use bevy::{input::mouse::MouseMotion, prelude::*};



pub fn move_camera(keyboard_input: Res<ButtonInput<KeyCode>>, mut mouse_input: EventReader<MouseMotion>, _time: Res<Time>, mut query: Query<&mut Transform, With<Camera>>) {
    let sensitivity = 0.00048828125;


    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut speed = 1.0;


        for ev in mouse_input.read() {
            
            transform.rotate_local_x((-ev.delta.y * sensitivity).clamp(-1.57, 1.75));
            transform.rotate_y(-ev.delta.x * sensitivity);

        }

        if keyboard_input.pressed(KeyCode::ControlLeft){
            speed = 5.0;
        }
        
       
        // direction avant-arrière:
        // (les customs transforms servent éviter de controller l'altitude avec les touches Z et S 
        // comme dans minecraft en créatif plutôt qu'en spectateur)
        if keyboard_input.pressed(KeyCode::KeyW) {
            let mut custom_transform = *transform.forward();
            custom_transform.y = 0.0;
            direction += custom_transform;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            let mut custom_transform = *transform.forward();
            custom_transform.y = 0.0;
            direction -= custom_transform;
        }
        // direction droite-gauche
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *transform.right();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *transform.right();  
        }

        // Direction haut-bas
        if keyboard_input.pressed(KeyCode::ShiftLeft){
            direction.y -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Space){
            direction.y += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize() * speed;
        }

        transform.translation += direction * 0.125;
    }
}

