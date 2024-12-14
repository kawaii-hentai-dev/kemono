use std::sync::{atomic::AtomicBool, Arc};

use once_cell::sync::Lazy;

pub mod utils;

pub static DONE: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
