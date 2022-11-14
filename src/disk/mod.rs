mod partition;

use partition::Partition;
use uuid::Uuid;
use core::fmt;
use std::fs::File;
use std::io::{self, Seek, Read};
use std::os::unix::prelude::FileExt;

pub struct Disk {
    _image: File,
    partitions: Vec<Partition>,
}

impl Disk {
    pub fn load(file: &String) -> Result<Disk, io::Error> {
        let mut file = File::open(file)?;

        Ok(Disk{ 
            partitions: parse_partitions(&mut file)?,
            _image: file, 
        })
    }
}

impl fmt::Display for Disk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  | Type\tStart\tEnd\tSize (MB)")?;
        writeln!(f, "    ---\t\t---\t---\t---")?;

        for (i, partition) in self.partitions.iter().enumerate() {
            writeln!(f, "{} | {}", i+1, partition)?;
        }
        Ok(())
    }
}

fn parse_partitions(file: &mut File) -> Result<Vec<Partition>, io::Error> {
    // read partition type of first entry
    let mut buf: [u8; 1] = [0; 1];
    file.read_exact_at(&mut buf, 0x01BE + 0x04)?;

    if buf[0] == 0xEE {
        Ok(parse_gpt(file)?)
    } else {
        Ok(parse_mbr(file)?)
    }
}

fn parse_gpt(file: &mut File) -> Result<Vec<Partition>, io::Error> {
    let mut partitions: Vec<Partition> = Vec::new();

    // read number of entires in array
    let mut buf: [u8; 4] = [0; 4];
    file.read_exact_at(&mut buf, 0x200 + 0x50)?;

    //seek to first partition entry
    file.seek(io::SeekFrom::Start(0x400))?;

    for _ in 0..u32::from_le_bytes(buf) {
        let mut raw_entry: [u8; 128] = [0; 128];
        file.read_exact(&mut raw_entry)?;

        let type_uuid = Uuid::from_bytes_le(raw_entry[0x00..0x10].try_into().expect("type uuid not 16 bytes"));
        if type_uuid.is_nil() {
            continue;
        }

        partitions.push(Partition::new1(
            type_uuid.to_string(), 
            u64::from_le_bytes(raw_entry[0x20..0x28].try_into().expect("start lba not 8 bytes")),
            u64::from_le_bytes(raw_entry[0x28..0x30].try_into().expect("end lba not 8 bytes")),
        ));
    }
    
    Ok(partitions)
}

fn parse_mbr(file: &mut File) -> Result<Vec<Partition>, io::Error> {
    let mut partitions: Vec<Partition> = Vec::new();

    //seek to table
    file.seek(io::SeekFrom::Start(0x1BE))?;

    for _ in 0..4 {
        let mut raw_entry: [u8; 16] = [0; 16];
        file.read_exact(&mut raw_entry)?;

        partitions.push(Partition::new2(
            mbr_partition_type(raw_entry[0x04]), 
            u32::from_le_bytes(raw_entry[0x08..0x0C].try_into().expect("start lba not 4 bytes")) as u64,
            u32::from_le_bytes(raw_entry[0x0C..0x10].try_into().expect("num sectors not 4 bytes")) as u64,
        ));
    }

    Ok(partitions)
}

fn mbr_partition_type(partition_type: u8) -> String {
    match partition_type {
        0x00 => String::from("Empty"),
        0x01 => String::from("FAT12"),
        0x04|0x06|0x0E => String::from("FAT16"),
        0x05 => String::from("Extended"),
        0x07 => String::from("NTFS"),
        0x0B|0x0C => String::from("FAT32"),
        0x82 => String::from("Linux Swap"),
        0x83 => String::from("ext2/3/4"),
        0xEE => String::from("GPT Protective MBR"),
        _ => String::from("Unknown"),
    }
}
