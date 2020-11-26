use memmap::MmapOptions;
use std::io::{Read, SeekFrom, Seek, Cursor};
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};
use rayon::prelude::*;
use std::convert::TryFrom;
use std::error::Error;
use memmap::Mmap;
use std::collections::HashMap;
use std::collections::LinkedList;

use super::shared::GGPK;
use super::shared::FileRecord;

pub fn read<F>(filepath: &str, callback: F) where F: Fn(GGPK) {
    let file = File::open(filepath).expect("Opening GGPK file");
    let mmap = unsafe { MmapOptions::new().map(&file).expect("Mapping GGPK file to memory") };
    let files_list = read_record(&mmap, 0, &String::from(""));

    let mut files = HashMap::new();
    files_list.iter().for_each(|record| {
        files.insert(format!("{}/{}", record.path, record.name), record);
    });

    callback(GGPK { mmap, files });
}

pub fn get_record<F>(record: &FileRecord, mmap: &Mmap, callback: F) where F: Fn(&[u8]) {
    let file_end = record.begin + usize::try_from(record.bytes).unwrap();
    match mmap.get(record.begin..file_end) {
        Some(bytes) => {
            callback(bytes);
        },
        None => error!("Failed reading bytes for file {}/{}", record.path, record.name),
    }
}

fn read_record(mmap: &Mmap, offset: u64, base: &String) -> LinkedList<FileRecord> {
    let mut c = Cursor::new(mmap);
    c.set_position(offset);

    let record_size = c.read_u32::<LittleEndian>().unwrap();
    let record_type = read_record_tag(&mut c).unwrap();

    trace!("RECORD {} offset({}) {} bytes", record_type, offset, record_size);
    match record_type.as_str() {
        "GGPK" => {
            let ggpk_version = c.read_u32::<LittleEndian>().unwrap();
            trace!("GGPK version {}", ggpk_version);

            let records = (record_size - 12) / 8;
            return (0..records)
                .map(|_| c.read_u64::<LittleEndian>().unwrap())
                .map(|offset| read_record(mmap, offset, &base))
                .fold(LinkedList::new(), |mut acc, mut x| {acc.append(&mut x); acc} );
        },
        "PDIR" => {
            c.seek(SeekFrom::Current(4)).unwrap(); // ignore name length
            let entries_length = c.read_u32::<LittleEndian>().unwrap();
            c.seek(SeekFrom::Current(32)).unwrap(); // ignore hash value
            let name = read_utf16(&mut c);
            let path = if base.len() == 0 { name.clone() } else { format!("{}/{}", base, name) };

            return (0..entries_length)
                .into_iter()    
                .into_par_iter()
                .map(|i| {
                    let mut c2 = Cursor::new(&mmap);
                    c2.set_position(c.position() + (i * 12) as u64);
                    c2.seek(SeekFrom::Current(4)).unwrap(); // ignore hash value
                    let offset = c2.read_u64::<LittleEndian>().unwrap();
                    return read_record(mmap, offset, &path);
                })
                .fold(|| LinkedList::new(), |mut acc, mut x| {acc.append(&mut x); acc} )
                .reduce(|| LinkedList::new(), |mut acc, mut x| {acc.append(&mut x); acc} );
        },
        "FILE" => {
            let name_length = c.read_u32::<LittleEndian>().unwrap();
            let signature = read_file_signature(&mut c).unwrap();
            let filename = read_utf16(&mut c);
            let name_size = (filename.len() + 1) * 2;
            if usize::try_from(name_length).unwrap() != filename.len() + 1 {
                warn!("Length of '{}' different than specified. Expected: {}", filename, name_length);
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
        "FREE" => 
             LinkedList::new(), // Unused space, ignore
        _ => {
            warn!("Found undefined type: {}", record_type);
            return LinkedList::new();
        }
    }
}

fn read_record_tag(c: &mut Cursor<&memmap::Mmap>) -> Result<String, Box<dyn Error>> { 
    let mut bytes = [0u8; 4];
    c.read_exact(&mut bytes)?;
    let tag = String::from_utf8((&bytes).to_vec())?;
    return Ok(tag);
}

fn read_file_signature(c: &mut Cursor<&memmap::Mmap>) -> Result<[u8; 32], Box<dyn Error>> { 
    let mut bytes = [0u8; 32];
    c.read_exact(&mut bytes)?;
    return Ok(bytes);
}

fn read_utf16(c: &mut Cursor<&memmap::Mmap>) -> String {
    let raw = (0..)
        .map(|_| c.read_u16::<LittleEndian>().unwrap())
        .take_while(|&x| x != 0u16)
        .collect::<Vec<u16>>();
    return String::from_utf16(&raw).unwrap();
}
