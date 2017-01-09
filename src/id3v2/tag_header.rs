use std::vec;
use id3v2::to_synchsafe;

pub struct TagHeader {
    version: u8,
    minor_version: u8,
    header_flag: u8,
    size: u32
}

impl TagHeader {
    pub fn new(bytes: vec::Vec<u8>) -> Self {
        if !(bytes[0] as char == 'I' && bytes[1] as char == 'D' && bytes[2] as char == '3') {
            debug!("Invalid IDv2: `{}`", String::from_utf8_lossy(&bytes[0..4]));
            return TagHeader {
                version: 0, minor_version: 0, header_flag: 0, size: 0
            };
        }

        let version = bytes[3] as u8;
        let minor_version = bytes[4] as u8;
        let header_flag = bytes[5] as u8;
        let size = to_synchsafe(&bytes[6..10]);

        TagHeader {
            version: version, minor_version: minor_version, header_flag: header_flag, size: size
        }
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_minor_version(&self) -> u8 {
        self.minor_version
    }

    pub fn has_unsynchronisation(&self) -> bool {
        self.header_flag & 0x01 << 7 != 0
    }

    pub fn has_extended(&self) -> bool {
        self.header_flag & 0x01 << 6 != 0
    }

    pub fn has_experimental(&self) -> bool {
        self.header_flag & 0x01 << 5 != 0
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}