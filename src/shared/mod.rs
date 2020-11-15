use memmap::Mmap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GGPK<'a> {
    pub mmap: Mmap,
    pub files: HashMap<String, &'a FileRecord>,
}

#[derive(Debug)]
pub struct FileRecord {
    pub name: String,
    pub path: String,
    pub signature: [u8; 32],
    pub begin: usize,
    pub bytes: u32,
}