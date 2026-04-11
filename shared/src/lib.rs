pub mod network;
pub mod parallel;
pub mod world;

#[macro_export]
macro_rules! log_impl {
    ($prefix:expr, $($args:tt)*) => {
        println!("{} {}", $prefix, format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        #[cfg(feature = "server")]
        log_impl!("[SERVER]$> ", $($args)*);
        #[cfg(feature = "client")]
        log_impl!("[CLIENT]$> ", $($args)*);
        #[cfg(not(any(feature = "server", feature = "client")))]
        log_impl!("> ", $($args)*);
    };
}

#[macro_export]
macro_rules! log_err {
    ($($args:tt)*) => {
        #[cfg(feature = "server")]
        { eprintln!("[SERVER]$> {}", format_args!($($args)*)); }
        #[cfg(feature = "client")]
        { eprintln!("[CLIENT]$> {}", format_args!($($args)*)); }
        #[cfg(not(any(feature = "server", feature = "client")))]
        { eprintln!("> {}", format_args!($($args)*)); }
    };
}

pub use network::messages::{
    create_chat, create_disconnect, create_handshake, create_handshake_ack, create_ping, create_player_update, create_pong,
    create_world_data, ChunkData, ContenuPaquet, Paquet, Position, Rotation, TypePaquet, CURRENT_VERSION, MAX_PAQUET_SIZE,
};

pub use network::crypto::{compute_shared_secret, generate_server_id, server_id_to_hex, xor_crypt};

pub use network::network_protocol::{
    create_codec, create_server_id, perform_client_handshake, perform_server_handshake, Cipher, EncryptedCodec,
};
