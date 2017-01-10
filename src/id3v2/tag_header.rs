use id3v2;
use std::vec;

//
// see ./resources/id3v2.md#ID3v2 Header
const VERSION_OFFSET: usize = 3;
const MINOR_VERSION_OFFSET: usize = 4;
const HEAD_FLAG_OFFSET: usize = 5;
const UNSYNCHRONISATION_FLAG_OFFSET: u8 = 7;
const EXTENDED_FLAG_OFFSET: u8 = 6;
const EXPERIMENTAL_FLAG_OFFSET: u8 = 5;

fn head_size(bytes: &vec::Vec<u8>) -> u32 {
    id3v2::to_synchsafe(&bytes[6..10])
}

fn is_valid_id(bytes: &vec::Vec<u8>) -> bool {
    let is_valid = bytes[0] as char == 'I' && bytes[1] as char == 'D' && bytes[2] as char == '3';
    if !is_valid {
        debug!("Invalid IDv2: `{}`", String::from_utf8_lossy(&bytes[0..4]));
    }

    is_valid
}

pub struct TagHeader {
    version: u8,
    minor_version: u8,
    header_flag: u8,
    size: u32
}

impl TagHeader {
    pub fn new(bytes: vec::Vec<u8>) -> Self {
        if !is_valid_id(&bytes) {
            return TagHeader {
                version: 0, minor_version: 0, header_flag: 0, size: 0
            };
        }

        TagHeader {
            version: bytes[VERSION_OFFSET] as u8,
            minor_version: bytes[MINOR_VERSION_OFFSET] as u8,
            header_flag: bytes[HEAD_FLAG_OFFSET] as u8,
            size: head_size(&bytes)
        }
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_minor_version(&self) -> u8 {
        self.minor_version
    }

    pub fn has_unsynchronisation(&self) -> bool {
        self.header_flag & 0x01 << UNSYNCHRONISATION_FLAG_OFFSET != 0
    }

    pub fn has_extended(&self) -> bool {
        self.header_flag & 0x01 << EXTENDED_FLAG_OFFSET != 0
    }

    pub fn has_experimental(&self) -> bool {
        self.header_flag & 0x01 << EXPERIMENTAL_FLAG_OFFSET != 0
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}