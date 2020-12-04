use memmap::Mmap;
use memmap::MmapOptions;
use std::fs::File;

pub fn to_mmap_unsafe(path: &str) -> Mmap {
    let file = File::open(path).expect("Failed opening GGPK file");
    unsafe { MmapOptions::new().map(&file).expect("Failed creating mmap") }
}
