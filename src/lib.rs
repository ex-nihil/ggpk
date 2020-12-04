
#[macro_use] extern crate log;

pub mod ggpk;
pub use crate::ggpk::{GGPK, GGPKRead};

pub mod file;
pub use crate::file::{GGPKFile, FileRecord, FileRecordFn};

pub mod util;
pub use crate::util::to_mmap_unsafe;