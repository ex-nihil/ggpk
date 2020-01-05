use memmap::MmapOptions;
use std::io::{Read, Write, SeekFrom, Seek, Cursor};
use std::fs::{File, create_dir};
use byteorder::{LittleEndian, ReadBytesExt};
use rayon::prelude::*;
use std::time::Instant;
use std::convert::TryFrom;

fn read_record_tag(rdr: &mut Cursor<&memmap::Mmap>) -> String {
    let mut tag = [0u8; 4];
    rdr.read_exact(&mut tag).unwrap();
    return String::from_utf8((&tag).to_vec()).expect("Reading UTF-8 record tag");
}

fn read_utf16(rdr: &mut Cursor<&memmap::Mmap>) -> String {
    let raw = (0..)
        .map(|_| rdr.read_u16::<LittleEndian>().expect("Read UTF-16 until NULL term"))
        .take_while(|&x| x != 0u16)
        .collect::<Vec<u16>>();
    return String::from_utf16(&raw).expect("Decode a UTF-16 String")
}

fn read_record(mmap: &memmap::Mmap, offset: u64, base: &String) {
    let mut rdr = Cursor::new(mmap);
    rdr.set_position(offset);

    let byte_size = rdr.read_u32::<LittleEndian>().unwrap();
    let record_type = read_record_tag(&mut rdr);
    match record_type.as_str() {
        "GGPK" => { // Root record of the file.
            let records = rdr.read_u32::<LittleEndian>().unwrap();
            (0..records)
                .map(|_| rdr.read_u64::<LittleEndian>().unwrap())
                .for_each(|offset| read_record(mmap, offset, &base));
        },
        "PDIR" => {
            rdr.seek(SeekFrom::Current(4)).unwrap(); // ignore name length
            let entries_length = rdr.read_u32::<LittleEndian>().unwrap();
            rdr.seek(SeekFrom::Current(32)).unwrap(); // ignore hash value
            let name = read_utf16(&mut rdr);
            let path = if base.len() == 0 { name.clone() } else { format!("{}/{}", base, name) };
            if path.len() > 0 {
                let _ = create_dir(format!("dump/{}", path.clone()));
            }

            (0..entries_length)
                .into_par_iter()
                .for_each(|i| {
                    let mut rdr2 = Cursor::new(mmap);
                    rdr2.set_position(rdr.position() + (i * 12) as u64);
                    rdr2.seek(SeekFrom::Current(4)).unwrap(); // ignore hash value
                    let offset = rdr2.read_u64::<LittleEndian>().unwrap();
                    read_record(mmap, offset, &path);
                });
        },
        "FILE" => {
            rdr.seek(SeekFrom::Current(4)).unwrap(); // ignore name length
            rdr.seek(SeekFrom::Current(32)).unwrap(); // ignore hash value
            let name = read_utf16(&mut rdr);
            let name_size = (name.len()+1) * 2;
            
            let path = if base.len() == 0 { name.clone() } else { format!("{}/{}", base, name) };
            let data_start = usize::try_from(rdr.position()).unwrap();
            let data_length = byte_size - 44 - name_size as u32; // 44 = length + tag + strlen + hash
            let data_end = data_start + usize::try_from(data_length).unwrap();
            
            match mmap.get(data_start..data_end) {
                Some(bytes) => {
                    if name.ends_with(".txt") {
                        File::create(format!("dump/{}", path.clone())).unwrap()
                        .write(bytes).unwrap();
                    }
                },
                None    => println!("Failed getting slice of file {} {}->{}", name, data_start, data_end),
            }
        }
        _ => println!("Record type {0} not implemented.", record_type),
    }
}

fn main() {
    let now = Instant::now();

    let file = File::open("/mnt/d/Path of Exile/Content_old.ggpk").unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    read_record(&mmap, 0, &String::from(""));
    
    println!("Time spent: {}ms", now.elapsed().as_millis());
}
