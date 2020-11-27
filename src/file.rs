use std::fs::{create_dir_all, File};
use std::io::Error;
use std::io::ErrorKind::InvalidData;
use std::io::Write;
use std::path::Path;

use super::ggpk::GGPK;

#[derive(Debug)]
pub struct GGPKFile<'a> {
    pub ggpk: &'a GGPK<'a>,
    pub record: FileRecord,
}

pub trait GGPKFileFn {
    fn write_file(&self, path: &str) -> Result<usize, Error>;
    fn write_into(&self, dst: &mut impl Write) -> Result<usize, Error>;
}

impl GGPKFileFn for GGPKFile<'_> {
    fn write_file(&self, path: &str) -> Result<usize, Error> {
        let record = &self.record;
        self.ggpk.mmap
            .get(record.begin..(record.begin + record.bytes as usize))
            .map(|bytes| {
                Path::new(path).parent().map(|path| create_dir_all(path));
                File::create(path).and_then(|mut file| file.write(bytes))
            })
            .unwrap_or_else(|| Err(Error::new(InvalidData, "Read outside GGPK")))
    }

    fn write_into(&self, dst: &mut impl Write) -> Result<usize, Error> {

        let record = &self.record;
        self.ggpk.mmap
            .get(record.begin..(record.begin + record.bytes as usize))
            .map(|bytes| {
                dst.write(bytes)
            })
            .unwrap_or_else(|| Err(Error::new(InvalidData, "Read outside GGPK")))

    }
}

#[derive(Debug)]
pub struct FileRecord {
    pub name: String,
    pub path: String,
    pub signature: [u8; 32],
    pub begin: usize,
    pub bytes: u32,
}

pub trait FileRecordFn {
    fn absolute_path(&self) -> String;
    fn clone(&self) -> FileRecord;
}

impl FileRecordFn for FileRecord {

    fn absolute_path(&self) -> String {
        format!("{}/{}", self.path, self.name)
    }

    fn clone(&self) -> FileRecord {
        FileRecord {
            name: self.name.clone(),
            path: self.path.clone(),
            signature: self.signature,
            begin: self.begin,
            bytes: self.bytes,
        }
    }
}
