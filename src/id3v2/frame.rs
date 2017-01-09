extern crate encoding;

use std::{vec, io, result};
use id3v2::to_u32;
use id3v2::scanner::Scanner;
use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::{ISO_8859_1, UTF_16LE, UTF_16BE, UTF_8};

pub struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8
}

impl Frame {
    pub fn new(scanner: &mut Scanner) -> io::Result<Frame> {
        let frame_header_bytes = try!(scanner.read_as_bytes(10));
        let frame_size = to_u32(&frame_header_bytes[4..8]);
        trace!("Frame.new=> size: {}", frame_size);

        if frame_size == 0 {
            warn!("FrameReader.next_frame: frame size is zero!");
        }

        let id = String::from_utf8_lossy(&frame_header_bytes[0..4]).into_owned();
        let frame_body_bytes = try!(scanner.read_as_bytes(frame_size as usize));

        Ok(Frame {
            id: id,
            size: frame_size,
            data: frame_body_bytes,
            status_flag: frame_header_bytes[8],
            encoding_flag: frame_header_bytes[9]
        })
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn has_preserve_tag(&self) -> bool {
        self.status_flag & 0x01 << 7 != 0
    }

    pub fn has_preserve_file(&self) -> bool {
        self.status_flag & 0x01 << 6 != 0
    }

    pub fn has_readonly(&self) -> bool {
        self.status_flag & 0x01 << 5 != 0
    }

    pub fn has_compression(&self) -> bool {
        self.encoding_flag & 0x01 << 7 != 0
    }

    pub fn has_encryption(&self) -> bool {
        self.encoding_flag & 0x01 << 6 != 0
    }

    pub fn has_group(&self) -> bool {
        self.encoding_flag & 0x01 << 5 != 0
    }

    pub fn get_data(&self) -> result::Result<String, String> {
        let data = self.data.clone().split_off(1);

        if self.data[0] == 0 {
            if let Ok(decoded) = ISO_8859_1.decode(&data, DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[0] == 1 {
            if let Ok(decoded) = UTF_16LE.decode(&data, DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[0] == 2 {
            if let Ok(decoded) = UTF_16BE.decode(&data, DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[0] == 3 {
            if let Ok(decoded) = UTF_8.decode(&data, DecoderTrap::Strict) {
                return Ok(decoded);
            }
        }

        match ISO_8859_1.decode(&data, DecoderTrap::Strict) {
            Ok(decoded) => Ok(decoded),
            Err(e) => Err(e.to_string())
        }
    }
}