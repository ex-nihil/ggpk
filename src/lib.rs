
#[macro_use] extern crate log;

mod shared;
pub use shared::GGPK;
pub use shared::FileRecord;

mod ggpk;
pub use ggpk::read;
pub use ggpk::get_record;