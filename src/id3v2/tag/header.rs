//MIT License
//
//Copyright (c) [2017] [Mark Han]
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

use id3v2;
use std::vec;

//
// see references/id3v2.md#ID3v2 Header
const VERSION_OFFSET: usize = 3;
const MINOR_VERSION_OFFSET: usize = 4;
const HEAD_FLAG_OFFSET: usize = 5;

const UNSYNCHRONISATION_OFFSET: u8 = 7;
const EXTENDED_HEADER_OFFSET: u8 = 6;
const EXPERIMENTAL_INDICATOR_OFFSET: u8 = 5;
const FOOTER_PRESENT_OFFSET: u8 = 4;

pub enum HeaderFlag {
    Unsynchronisation,
    ExtendedHeader,
    ExperimentalIndicator,
    FooterPresent
}

pub struct Header {
    version: u8,
    minor_version: u8,
    flag: u8,
    size: u32
}

impl Header {
    fn head_size(bytes: &vec::Vec<u8>) -> u32 {
        id3v2::bytes::to_synchsafe(&bytes[6..10])
    }

    fn is_valid_id(bytes: &vec::Vec<u8>) -> bool {
        let is_valid = bytes[0] as char == 'I' && bytes[1] as char == 'D' && bytes[2] as char == '3';
        if !is_valid {
            debug!("Invalid IDv2: `{}`", String::from_utf8_lossy(&bytes[0..4]));
        }

        is_valid
    }

    pub fn new(bytes: vec::Vec<u8>) -> Self {
        if !Self::is_valid_id(&bytes) {
            return Header {
                version: 0, minor_version: 0, flag: 0, size: 0
            };
        }

        Header {
            version: bytes[VERSION_OFFSET] as u8,
            minor_version: bytes[MINOR_VERSION_OFFSET] as u8,
            flag: bytes[HEAD_FLAG_OFFSET] as u8,
            size: Self::head_size(&bytes)
        }
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_minor_version(&self) -> u8 {
        self.minor_version
    }

    pub fn has_flag(&self, flag: HeaderFlag) -> bool {
        if self.version == 3 {
            match flag {
                HeaderFlag::Unsynchronisation => self.flag & 0x01 << UNSYNCHRONISATION_OFFSET != 0,
                HeaderFlag::ExtendedHeader => self.flag & 0x01 << EXTENDED_HEADER_OFFSET != 0,
                HeaderFlag::ExperimentalIndicator => self.flag & 0x01 << EXPERIMENTAL_INDICATOR_OFFSET != 0,
                _ => false
            }
        } else if self.version == 4 {
            match flag {
                HeaderFlag::Unsynchronisation => self.flag & 0x01 << UNSYNCHRONISATION_OFFSET != 0,
                HeaderFlag::ExtendedHeader => self.flag & 0x01 << EXTENDED_HEADER_OFFSET != 0,
                HeaderFlag::ExperimentalIndicator => self.flag & 0x01 << EXPERIMENTAL_INDICATOR_OFFSET != 0,
                HeaderFlag::FooterPresent => self.flag & 0x01 << FOOTER_PRESENT_OFFSET != 0
            }
        } else {
            warn!("Header.has_flag=> Unknown version!");
            false
        }
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}

// TODO
pub struct ExtendedHeader {
    size: u32
}

impl ExtendedHeader {
    pub fn new(size: u32, bytes: &vec::Vec<u8>) -> Self {
        ExtendedHeader {
            size: size
        }
    }
}