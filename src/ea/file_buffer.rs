#![allow(dead_code)]

use std::fs::File;
use std::io::{Read, Seek};

#[derive(Debug, PartialEq)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
pub struct FileBuffer {
    pub folder: String,
    pub name: String,
    pub data: File,
    pub read_offset: usize,
    pub length: u64,
    pub endian: Endian,
}

impl FileBuffer {
    pub fn new(name: String) -> FileBuffer {
        Self::with_endian(name, Endian::Little)
    }

    pub fn with_endian(mut path: String, endian: Endian) -> FileBuffer {
        path = path.replace("\\", "/");
        // println!("Reading file: {}", path);

        let name = path
            .split("/")
            .last()
            .unwrap()
            .split(".")
            .next()
            .unwrap()
            .to_string();
        let extension = path.split(".").last().unwrap();
        let folder = path.replace(&format!("{}.{}", name, extension), "");
        // println!("Folder: {}", folder);
        // println!("Name: {}", name);

        let data = File::open(path).unwrap();

        let metadata = data.metadata().unwrap();
        FileBuffer {
            folder,
            name,
            data,
            read_offset: 0,
            length: metadata.len(),
            endian,
        }
    }

    pub fn get_real_offset(&mut self) -> u64 {
        self.data.stream_position().unwrap()
    }

    pub fn skip(&mut self, bytes: usize) {
        self.read_offset += bytes;
        self.data
            .seek(std::io::SeekFrom::Current(bytes as i64))
            .unwrap();
    }

    pub fn seek(&mut self, offset: usize) {
        self.read_offset = offset;
        self.data
            .seek(std::io::SeekFrom::Start(offset as u64))
            .unwrap();
    }

    pub fn read_uint_8(&mut self) -> u8 {
        let mut buffer = [0_u8; size_of::<u8>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u8>();
        u8::from_le_bytes(buffer)
    }

    pub fn read_uint_16(&mut self) -> u16 {
        if self.endian == Endian::Big {
            return self.read_uint_16_be();
        }
        self.read_uint_16_le()
    }

    pub fn read_uint_16_le(&mut self) -> u16 {
        let mut buffer = [0_u8; size_of::<u16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u16>();
        u16::from_le_bytes(buffer)
    }

    pub fn read_uint_16_be(&mut self) -> u16 {
        let mut buffer = [0_u8; size_of::<u16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u16>();
        u16::from_be_bytes(buffer)
    }

    pub fn read_uint_32(&mut self) -> u32 {
        if self.endian == Endian::Big {
            return self.read_uint_32_be();
        }
        self.read_uint_32_le()
    }

    pub fn read_uint_32_le(&mut self) -> u32 {
        let mut buffer = [0_u8; size_of::<u32>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u32>();
        u32::from_le_bytes(buffer)
    }

    pub fn read_uint_32_be(&mut self) -> u32 {
        let mut buffer = [0_u8; size_of::<u32>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u32>();
        u32::from_be_bytes(buffer)
    }

    pub fn read_uint_64(&mut self) -> u64 {
        if self.endian == Endian::Big {
            return self.read_uint_64_be();
        }
        self.read_uint_64_le()
    }

    pub fn read_uint_64_le(&mut self) -> u64 {
        let mut buffer = [0_u8; size_of::<u64>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u64>();
        u64::from_le_bytes(buffer)
    }

    pub fn read_uint_64_be(&mut self) -> u64 {
        let mut buffer = [0_u8; size_of::<u64>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u64>();
        u64::from_be_bytes(buffer)
    }

    pub fn read_int_8(&mut self) -> i8 {
        let mut buffer = [0_u8; size_of::<i8>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i8>();
        i8::from_le_bytes(buffer)
    }

    pub fn read_int_16(&mut self) -> i16 {
        if self.endian == Endian::Big {
            return self.read_int_16_be();
        }
        self.read_int_16_le()
    }

    pub fn read_int_16_le(&mut self) -> i16 {
        let mut buffer = [0_u8; size_of::<i16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i16>();
        i16::from_le_bytes(buffer)
    }

    pub fn read_int_16_be(&mut self) -> i16 {
        let mut buffer = [0_u8; size_of::<i16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i16>();
        i16::from_be_bytes(buffer)
    }

    pub fn read_int_32(&mut self) -> i32 {
        if self.endian == Endian::Big {
            return self.read_int_32_be();
        }
        self.read_int_32_le()
    }

    pub fn read_int_32_le(&mut self) -> i32 {
        let mut buffer = [0_u8; size_of::<i32>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i32>();
        i32::from_le_bytes(buffer)
    }

    pub fn read_int_32_be(&mut self) -> i32 {
        let mut buffer = [0_u8; size_of::<i32>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i32>();
        i32::from_be_bytes(buffer)
    }

    pub fn read_int_64(&mut self) -> i64 {
        if self.endian == Endian::Big {
            return self.read_int_64_be();
        }
        self.read_int_64_le()
    }

    pub fn read_int_64_le(&mut self) -> i64 {
        let mut buffer = [0_u8; size_of::<i64>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i64>();
        i64::from_le_bytes(buffer)
    }

    pub fn read_int_64_be(&mut self) -> i64 {
        let mut buffer = [0_u8; size_of::<i64>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i64>();
        i64::from_be_bytes(buffer)
    }

    pub fn read_float_32(&mut self) -> f32 {
        if self.endian == Endian::Big {
            return self.read_float_32_be();
        }
        self.read_float_32_le()
    }

    pub fn read_float_32_le(&mut self) -> f32 {
        let mut buffer = [0_u8; size_of::<f32>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<f32>();
        f32::from_le_bytes(buffer)
    }

    pub fn read_float_32_be(&mut self) -> f32 {
        let mut buffer = [0_u8; size_of::<f32>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<f32>();
        f32::from_be_bytes(buffer)
    }

    pub fn read_uint_16_as_float(&mut self) -> f32 {
        if self.endian == Endian::Big {
            return self.read_uint_16_be_as_float();
        }
        self.read_uint_16_le_as_float()
    }

    pub fn read_uint_16_le_as_float(&mut self) -> f32 {
        let mut buffer = [0_u8; size_of::<u16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u16>();
        u16::from_le_bytes(buffer) as f32 / 65535.0
    }

    pub fn read_uint_16_be_as_float(&mut self) -> f32 {
        let mut buffer = [0_u8; size_of::<u16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<u16>();
        u16::from_be_bytes(buffer) as f32 / 65535.0
    }

    pub fn read_int_16_le_as_float(&mut self) -> f32 {
        let mut buffer = [0_u8; size_of::<i16>()];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += size_of::<i16>();
        i16::from_le_bytes(buffer) as f32 / 65535.0
    }

    pub fn read_buffer(&mut self, length: usize) -> Vec<u8> {
        let mut buffer = vec![0_u8; length];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += length;
        buffer
    }

    pub fn read_remaining(&mut self) -> Vec<u8> {
        let remaining_length = self.length - self.read_offset as u64;
        let mut buffer = vec![0_u8; remaining_length as usize];
        self.data.read_exact(&mut buffer).unwrap();
        // println!("Read remaining: {}", buffer.len());
        self.read_offset = self.length as usize;
        buffer
    }

    pub fn read_string(&mut self, length: usize) -> String {
        let mut buffer = vec![0_u8; length];
        self.data.read_exact(&mut buffer).unwrap();
        self.read_offset += length;
        String::from_utf8(buffer)
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string()
    }

    pub fn read_string_8(&mut self) -> String {
        let mut length = self.read_uint_8() as usize;
        if length > 254 {
            length = 0;
        }
        self.read_string(length)
    }

    pub fn read_string_16(&mut self) -> String {
        if self.endian == Endian::Big {
            return self.read_string_16_be();
        }
        self.read_string_16_le()
    }

    pub fn read_string_16_le(&mut self) -> String {
        let length = self.read_uint_16_le() as usize;
        self.read_string(length)
    }

    pub fn read_string_16_be(&mut self) -> String {
        let length = self.read_uint_16_be() as usize;
        self.read_string(length)
    }

    pub fn read_string_32(&mut self) -> String {
        if self.endian == Endian::Big {
            return self.read_string_32_be();
        }
        self.read_string_32_le()
    }

    pub fn read_string_32_le(&mut self) -> String {
        let length = self.read_uint_32_le() as usize;
        self.read_string(length)
    }

    pub fn read_string_32_be(&mut self) -> String {
        let length = self.read_uint_32_be() as usize;
        self.read_string(length)
    }
}
