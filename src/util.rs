use memmap::Mmap;
use memmap::MmapOptions;
use std::fs::File;
use std::io::Error;
use std::path::Path;

pub fn to_mmap_unsafe(path: &Path) -> Result<Mmap, Error> {
    let file = File::open(path)?;
    unsafe { MmapOptions::new().map(&file) }
}
