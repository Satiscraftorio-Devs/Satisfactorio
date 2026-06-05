#[cfg(feature = "tui")]
pub mod app;
#[cfg(feature = "tui")]
pub mod bridge;
#[cfg(feature = "tui")]
pub mod log;

#[cfg(feature = "tui")]
pub use bridge::{TuiBridge, TuiCommand};
