use bevy::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup,
        start
    )
    .add_systems(Startup, 
        add_people
    )
    .add_systems(Update, 
        greet_people
    )
    .run();

}

fn start(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Caméra
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(1.0, -0.5, 0.0), Vec3::Y),
        ..default()
    });

    // Cube
    let mesh = meshes.add(Cuboid::mesh(&Cuboid::new(1.0, 1.0, 2.0)));
    let material = materials.add(Color::srgb(1.0, 0.0, 0.0));

    commands.spawn(PbrBundle {
        mesh,
        material,
        transform: Transform::from_xyz(3.0, -1.5, 0.0),
        ..default()
    });
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
