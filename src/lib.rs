
#[macro_use] extern crate log;

pub mod ggpk;
pub mod version;
pub mod util;
pub mod file;

pub use crate::ggpk::{GGPK, GGPKRead};
pub use crate::file::{GGPKFile, FileRecord, FileRecordFn};