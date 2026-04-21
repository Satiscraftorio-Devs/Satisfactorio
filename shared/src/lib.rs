pub mod network;
pub mod parallel;
pub mod world;

#[macro_export]
macro_rules! time {
    ($label:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        let millis = duration.as_millis();
        let micros = duration.as_micros();
        let nanos = duration.as_nanos();
        println!("{}: {}ms/{}µs/{}ns", $label, millis, micros, nanos);
        result
    }};
}

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        println!("> {}", format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! log_err {
    ($($args:tt)*) => {
        eprintln!("> {}", format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! log_server {
    ($($args:tt)*) => {
        println!("[SERVER]$> {}", format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! log_err_server {
    ($($args:tt)*) => {
        eprintln!("[SERVER]$> {}", format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! log_client {
    ($($args:tt)*) => {
        println!("[CLIENT]$> {}", format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! log_err_client {
    ($($args:tt)*) => {
        eprintln!("[CLIENT]$> {}", format_args!($($args)*));
    };
}
