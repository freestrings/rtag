extern crate encoding;
extern crate regex;

use id3v2;
use self::encoding::{Encoding, DecoderTrap};
use std::{vec, io, result};

//
// see ./reference/id3v2.md#프레임헤더
const ID_REGEX: &'static str = r"^[A-Z][A-Z0-9]{3}$";
const HEAD_LEN: usize = 10;
const FRAME_ID_LEN: usize = 4;
// flag 2bytes
const STATUS_FLAG_OFFSET: usize = 8;
const ENCODING_FLAG_OFFSET: usize = 9;
// flag offsets
const PRESERVE_TAG_STATUS_FLAG_OFFSET: u8 = 7;
const PRESERVE_FILE_STATUS_FLAG_OFFSET: u8 = 6;
const READONLY_STATUS_FLAG_OFFSET: u8 = 5;
const COMPRESSION_ENCODING_FLAG_OFFSET: u8 = 7;
const ENCRYPTION_ENCODING_FLAG_OFFSET: u8 = 6;
const GROUP_ENCODING_FLAG_OFFSET: u8 = 5;
const FRAME_DATA_ENCODING_OFFSET: usize = 1;
// text encoding
const TEXT_ENCODING_ISO_8859_1: u8 = 0;
const TEXT_ENCODING_UTF_16LE: u8 = 1;
const TEXT_ENCODING_UTF_16BE: u8 = 2;
const TEXT_ENCODING_UTF_8: u8 = 3;

fn frame_id(bytes: &vec::Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes[0..4]).into_owned()
}

fn frame_size(bytes: &vec::Vec<u8>) -> u32 {
    id3v2::to_u32(&bytes[4..8])
}

pub struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8
}

impl Frame {
    pub fn new(scanner: &mut id3v2::scanner::Scanner) -> io::Result<Frame> {
        let header_bytes = try!(scanner.read_as_bytes(HEAD_LEN));
        let id = frame_id(&header_bytes);
        let frame_size = frame_size(&header_bytes);
        let body_bytes = try!(scanner.read_as_bytes(frame_size as usize));

        debug!("Frame.new=> frame size: {}", frame_size);
        if frame_size == 0 { warn!("Frame.new: frame size is 0!"); }

        Ok(Frame {
            id: id,
            size: frame_size,
            data: body_bytes,
            status_flag: header_bytes[STATUS_FLAG_OFFSET],
            encoding_flag: header_bytes[ENCODING_FLAG_OFFSET]
        })
    }

    pub fn has_next_frame(scanner: &mut id3v2::scanner::Scanner) -> bool {
        match scanner.read_as_string(FRAME_ID_LEN) {
            Ok(id) => {
                let re = regex::Regex::new(ID_REGEX).unwrap();
                scanner.rewind(FRAME_ID_LEN as u64);
                let matched = re.is_match(&id);
                debug!("Frame.has_next_frame=> Frame Id:{}, matched: {}", id, matched);
                matched
            },
            Err(_) => {
                debug!("Frame.has_next_frame=> Fail");
                false
            }
        }
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn has_preserve_tag(&self) -> bool {
        self.status_flag & 0x01 << PRESERVE_TAG_STATUS_FLAG_OFFSET != 0
    }

    pub fn has_preserve_file(&self) -> bool {
        self.status_flag & 0x01 << PRESERVE_FILE_STATUS_FLAG_OFFSET != 0
    }

    pub fn has_readonly(&self) -> bool {
        self.status_flag & 0x01 << READONLY_STATUS_FLAG_OFFSET != 0
    }

    pub fn has_compression(&self) -> bool {
        self.encoding_flag & 0x01 << COMPRESSION_ENCODING_FLAG_OFFSET != 0
    }

    pub fn has_encryption(&self) -> bool {
        self.encoding_flag & 0x01 << ENCRYPTION_ENCODING_FLAG_OFFSET != 0
    }

    pub fn has_group(&self) -> bool {
        self.encoding_flag & 0x01 << GROUP_ENCODING_FLAG_OFFSET != 0
    }

    pub fn get_data(&self) -> result::Result<String, String> {
        let data = self.data.clone().split_off(FRAME_DATA_ENCODING_OFFSET);

        if self.data[0] == TEXT_ENCODING_ISO_8859_1 {
            if let Ok(decoded) = encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[0] == TEXT_ENCODING_UTF_16LE {
            if let Ok(decoded) = encoding::all::UTF_16LE.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[0] == TEXT_ENCODING_UTF_16BE {
            if let Ok(decoded) = encoding::all::UTF_16BE.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[0] == TEXT_ENCODING_UTF_8 {
            if let Ok(decoded) = encoding::all::UTF_8.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        }

        match encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict) {
            Ok(decoded) => Ok(decoded),
            Err(e) => Err(e.to_string())
        }
    }
}