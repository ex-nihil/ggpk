use byteorder::{LittleEndian, ReadBytesExt};
use memmap::Mmap;
use memmap::MmapOptions;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct GGPK<'a> {
    mmap: Mmap,
    files: HashMap<String, &'a FileRecord>,
}

#[derive(Debug)]
pub struct FileRecord {
    pub name: String,
    pub path: String,
    pub signature: [u8; 32],
    pub begin: usize,
    pub bytes: u32,
}

impl GGPK<'_> {
    pub fn from_install(install_path: &str) -> GGPK<'_> {
        let content_path = format!("{}/Content.ggpk", install_path);

        let file = File::open(content_path).expect("Failed opening GGPK file");
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .expect("Mapping GGPK file to memory")
        };

        GGPK {
            mmap,
            files: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> GGPK<'_> {
        let file = File::open(path).expect("Failed opening GGPK file");
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .expect("Mapping GGPK file to memory")
        };

        GGPK {
            mmap,
            files: HashMap::new(),
        }
    }
}

pub trait GGPKRead {
    fn get_file(&self, path: &str) -> Vec<u8>;
    fn list_files(&self) -> Vec<String>;
}

impl GGPKRead for GGPK<'_> {
    fn get_file(&self, path: &str) -> Vec<u8> {
        let files_list = read_record(
            &self.mmap,
            0,
            &String::from(""),
            &mut Some(path.to_string()),
        );

        files_list
            .iter()
            .take(1)
            .map(|record| {
                let mut dst = Vec::with_capacity(record.bytes as usize);
                get_record(record, &self.mmap, &mut dst);
                return dst;
            })
            .flatten()
            .collect()
    }

    fn list_files(&self) -> Vec<String> {
        let files_list = read_record(&self.mmap, 0, &String::from(""), &mut None);

        files_list
            .iter()
            .map(|r| format!("{}/{}", r.path, r.name))
            .collect()
    }
}

fn get_record(record: &FileRecord, mmap: &Mmap, dst: &mut Vec<u8>) {
    let file_end = record.begin + usize::try_from(record.bytes).unwrap();
    match mmap.get(record.begin..file_end) {
        Some(bytes) => {
            dst.write_all(bytes).expect("Write failed");
        }
        _ => warn!("Read failed: {}/{}", record.path, record.name),
    }
}

// TODO: refactor this
fn read_record(
    mmap: &Mmap,
    offset: u64,
    base: &String,
    wanted: &Option<String>,
) -> LinkedList<FileRecord> {
    let mut c = Cursor::new(mmap);
    c.set_position(offset);

    let record_size = c.read_u32::<LittleEndian>().unwrap();
    let record_type = read_record_tag(&mut c).unwrap();

    trace!(
        "RECORD {} offset({}) {} bytes",
        record_type,
        offset,
        record_size
    );
    match record_type.as_str() {
        "GGPK" => {
            let ggpk_version = c.read_u32::<LittleEndian>().unwrap();
            trace!("GGPK version {}", ggpk_version);

            let records = (record_size - 12) / 8;
            return (0..records)
                .map(|_| c.read_u64::<LittleEndian>().unwrap())
                .map(|offset| read_record(mmap, offset, &base, wanted))
                .fold(LinkedList::new(), |mut acc, mut x| {
                    acc.append(&mut x);
                    acc
                });
        }
        "PDIR" => {
            c.seek(SeekFrom::Current(4)).unwrap(); // ignore name length
            let entries_length = c.read_u32::<LittleEndian>().unwrap();
            c.seek(SeekFrom::Current(32)).unwrap(); // ignore hash value
            let name = read_utf16(&mut c);
            let path = if base.len() == 0 {
                name.clone()
            } else {
                format!("{}/{}", base, name)
            };

            let wtf = wanted;
            if !wanted
                .as_ref()
                .map(|s| s.starts_with(&path))
                .unwrap_or(true)
            {
                return LinkedList::new();
            }

            return (0..entries_length)
                .into_iter()
                .into_par_iter()
                .map(|i| {
                    let mut c2 = Cursor::new(&mmap);
                    c2.set_position(c.position() + (i * 12) as u64);
                    c2.seek(SeekFrom::Current(4)).unwrap(); // ignore hash value
                    let offset = c2.read_u64::<LittleEndian>().unwrap();
                    return read_record(mmap, offset, &path, wtf);
                })
                .fold(
                    || LinkedList::new(),
                    |mut acc, mut x| {
                        acc.append(&mut x);
                        acc
                    },
                )
                .reduce(
                    || LinkedList::new(),
                    |mut acc, mut x| {
                        acc.append(&mut x);
                        acc
                    },
                );
        }
        "FILE" => {
            let name_length = c.read_u32::<LittleEndian>().unwrap();
            let signature = read_file_signature(&mut c).unwrap();
            let filename = read_utf16(&mut c);
            if !wanted
                .as_ref()
                .map(|s| s.ends_with(&filename))
                .unwrap_or(true)
            {
                return LinkedList::new();
            }
            let name_size = (filename.len() + 1) * 2;
            if usize::try_from(name_length).unwrap() != filename.len() + 1 {
                warn!(
                    "Length of '{}' different than specified. Expected: {}",
                    filename, name_length
                );
            }

            let data_start = usize::try_from(c.position()).unwrap();
            let data_size = record_size - 44 - name_size as u32; // 44 = length + tag + strlen + hash

            let mut list = LinkedList::new();
            list.push_back(FileRecord {
                name: filename,
                path: format!("{}", base),
                signature,
                begin: data_start,
                bytes: data_size,
            });
            return list;
        }
        "FREE" => LinkedList::new(), // Unused space, ignore
        _ => {
            warn!("Found undefined type: {}", record_type);
            return LinkedList::new();
        }
    }
}

fn read_record_tag(c: &mut Cursor<&memmap::Mmap>) -> Result<String, Box<dyn Error>> {
    let mut bytes = [0u8; 4];
    c.read_exact(&mut bytes)?;
    Ok(String::from_utf8(bytes.to_vec())?)
}

fn read_file_signature(c: &mut Cursor<&memmap::Mmap>) -> Result<[u8; 32], Box<dyn Error>> {
    let mut bytes = [0u8; 32];
    c.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_utf16(c: &mut Cursor<&memmap::Mmap>) -> String {
    let raw = (0..)
        .map(|_| c.read_u16::<LittleEndian>().unwrap())
        .take_while(|&x| x != 0u16)
        .collect::<Vec<u16>>();
    String::from_utf16(&raw).expect("Failed reading utf16")
}
