use network::DEFAULT_SERVER_ADDRESS;
use std::{
    env::current_exe,
    process::{Child, Command},
};
pub enum LaunchMode {
    Singleplayer(String),
    Multiplayer(String),
}

pub fn set_play_mode(mode: LaunchMode) {
    match mode {
        LaunchMode::Singleplayer(save_path) => start_singleplayer(&save_path),
        LaunchMode::Multiplayer(address) => start_multiplayer(&address),
    }
}

pub fn start_singleplayer(save_path: &str) {
    let mut server = spawn_server(save_path);
    let mut client = spawn_client(DEFAULT_SERVER_ADDRESS);

    let client_exit = client.wait();
    if client_exit.is_err() {
        eprintln!("Le client s'est terminé avec une erreur : {client_exit:?}");
    }

    let _ = server.kill();
    let _ = server.wait();
}

pub fn start_multiplayer(address: &str) {
    let mut client = spawn_client(address);
    let status = client.wait();

    if status.is_err() {
        eprintln!("Le client s'est terminé avec une erreur : {status:?}");
    }
}

fn spawn_client(address: &str) -> Child {
    const CLIENT_FILE_NAME: &str = "Ascendustry";
    let client_path = current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(CLIENT_FILE_NAME)))
        .unwrap_or_else(|| CLIENT_FILE_NAME.into());

    Command::new(&client_path)
        .arg("--address")
        .arg(address)
        .spawn()
        .expect("Impossible de lancer le client")
}

fn spawn_server(save_path: &str) -> Child {
    const SERVER_FILE_NAME: &str = "server";
    let server_path = current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(SERVER_FILE_NAME)))
        .unwrap_or_else(|| SERVER_FILE_NAME.into());

    Command::new(&server_path)
        .arg("--no-tui")
        .arg("-p")
        .arg(save_path)
        .spawn()
        .expect("Impossible de lancer le serveur")
}
