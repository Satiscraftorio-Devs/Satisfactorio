use std::f32::consts::PI;

use bevy::{input::mouse::{self, MouseMotion}, prelude::*};

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup,
        start
    )
    .add_systems(Startup, 
        add_people
    )
    /*.add_systems(Update, 
        greet_people
    ) */
    .add_systems(Update,
        move_camera
    )
    .run();

}

fn start(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Caméra
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0.0, 0.0, 1.0), Vec3::Y),
        ..default()
    });

    // Cube
    let mesh = meshes.add(Cuboid::mesh(&Cuboid::new(5.0, 10.0, 5.0)));
    let material = materials.add(Color::srgb(1.0, 0.0, 0.0));

    commands.spawn(PbrBundle {
        mesh,
        material,
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..default()
    });
}

fn move_camera(keyboard_input: Res<ButtonInput<KeyCode>>, mut mouse_input: EventReader<MouseMotion>, time: Res<Time>, mut query: Query<&mut Transform, With<Camera>>) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;  

        for ev in mouse_input.read() {
            let mut rotation = transform.rotation.to_euler(EulerRot::XYZ);

            rotation.1 += ev.delta.x * 0.00048828125; // constante de sensibilité

            rotation.0 += -ev.delta.y * 0.00048828125; // constante de sensibilité
            rotation.0 = rotation.0.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

            transform.rotation = Quat::from_euler(EulerRot::YXZ, rotation.1, rotation.0, 0.0);

        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;  
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * 0.125;
    }
}


// Tests des fonctionnalités

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Person;

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(query: Query<&Name, With<Person>>, time: Res<Time>) {
    for name in &query {
        println!("hello {}!", name.0);
    }
    println!("fps: {} Δt: {}", (1.0/time.delta_seconds()).round() as i32, time.delta_seconds());
}
