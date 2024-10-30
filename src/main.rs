mod camera;
mod movement;
mod debug_controls;
mod world_generator;
mod world_generator_beta;

use movement::*;
use debug_controls::*;
use world_generator::*;
use camera::*;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, WindowMode};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::DiagnosticsStore;

fn main() {
    App::new()
    .add_plugins((DefaultPlugins, WireframePlugin, FrameTimeDiagnosticsPlugin))
    .insert_resource(ChunkManager::default())
    .insert_resource(RenderDistance(12))
    .insert_resource(ChunkWidth(62.0))
    .insert_resource(CameraRotRelative(0.0))
    .init_resource::<WireframeConfig>()
    .insert_resource(ClearColor(Color::srgb_u8(37, 179, 226)))
    .add_systems(Startup, (
        start,
        setup_light_world,
    ),
                
    )
    .add_systems(Update,(
        move_camera,
        unload_chunks,
        display_chunk_mesh,
        move_player,
        check_debug_controls,
    ))
    //.add_systems(PostUpdate, fps_count)
    .run();
}


fn start(mut commands: Commands, mut windows: Query<&mut Window>, mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = false;
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-15.0, 25.0, -20.0).with_rotation(Quat::default()),
        ..default()
    });   

    if let Ok(mut window) = windows.get_single_mut() {
        window.mode = WindowMode::Fullscreen;
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Confined;
    }   
}


fn fps_count(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            println!("Fps: {:.2?}", value);
        }
    }
}