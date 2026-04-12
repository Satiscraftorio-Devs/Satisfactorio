use shared::*;

use rand::RngExt;
use std::sync::atomic::{AtomicU32, Ordering};

static SERVER_SEED: AtomicU32 = AtomicU32::new(0);

pub fn init_server_seed() {
    let mut rng = rand::rng();
    let seed = rng.random();
    SERVER_SEED.store(seed, Ordering::SeqCst);
    log_server!("Server seed: {}", seed);
}

pub fn get_server_seed() -> u32 {
    SERVER_SEED.load(Ordering::SeqCst)
}
