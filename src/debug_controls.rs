use bevy::prelude::*;
use bevy::window::{CursorGrabMode, WindowMode};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};

pub fn check_debug_controls(keyboard_input: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>, mut wireframe_global: ResMut<WireframeConfig>) {

    // Plein écran
    if keyboard_input.just_pressed(KeyCode::F1) {
        if let Ok(mut window) = windows.get_single_mut() {
            if window.mode == WindowMode::Fullscreen {
                window.mode = WindowMode::Windowed;
            }
            else {
                window.mode = WindowMode::Fullscreen;
            }
        } 
    }

    // Souris
    if keyboard_input.just_pressed(KeyCode::F2) {
        if let Ok(mut window) = windows.get_single_mut() {
            if window.cursor.visible == false {
                window.cursor.visible = true;
                window.cursor.grab_mode = CursorGrabMode::None;
            }
            else {
                window.cursor.visible = false;
                window.cursor.grab_mode = CursorGrabMode::Confined;
            }
        } 
    }

    // Wireframe
    if keyboard_input.just_pressed(KeyCode::F3) {
        if let Ok(mut window) = windows.get_single_mut() {
            wireframe_global.global = !wireframe_global.global;
        } 
    }

}