#![feature(mpmc_channel)]

use std::sync::atomic::AtomicBool;

pub mod helper;
pub mod utils;

pub static DONE: AtomicBool = AtomicBool::new(false);

pub mod stdio;
