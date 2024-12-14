use std::sync::atomic::AtomicBool;

pub mod batch;
pub mod utils;

pub static DONE: AtomicBool = AtomicBool::new(false);
