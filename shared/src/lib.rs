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
