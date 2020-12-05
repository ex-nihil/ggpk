use memmap::Mmap;
use memmap::MmapOptions;
use std::fs::File;
use std::io::Error;

pub fn to_mmap_unsafe(path: &str) -> Result<Mmap, Error> {
    let file = File::open(path)?;
    unsafe { MmapOptions::new().map(&file) }
}
