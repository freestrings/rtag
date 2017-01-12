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

extern crate encoding;
extern crate regex;

use id3v2;
use readable;
use self::encoding::{Encoding, DecoderTrap};
use std::{vec, io, result, ops, borrow};
use std::io::Result;

// text encoding
const ISO_8859_1: u8 = 0;
const UTF_16LE: u8 = 1;
const UTF_16BE: u8 = 2;
const UTF_8: u8 = 3;

pub enum FrameHeaderFlag {
    TagAlter,
    FileAlter,
    ReadOnly,
    Compression,
    Encryption,
    GroupIdentity,
    //2.4 only
    Unsynchronisation,
    //2.4 only
    DataLength
}

pub struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8
}

impl Frame {
    fn _frame_id(bytes: &vec::Vec<u8>) -> String {
        String::from_utf8_lossy(&bytes[0..4]).into_owned()
    }

    fn _frame_size(bytes: &vec::Vec<u8>, tag_version: u8) -> u32 {
        match tag_version {
            3 => id3v2::bytes::to_u32(&bytes[4..8]),
            _ => id3v2::bytes::to_synchsafe(&bytes[4..8])
        }
    }

    pub fn has_next_frame<T: io::Read + io::Seek>(readable: &mut readable::Readable<T>) -> bool {
        // read frame id 4 bytes
        match readable.as_string(4) {
            Ok(id) => {
                // rewind
                readable.skip(-4);
                // @see http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let re = regex::Regex::new(r"^[A-Z][A-Z0-9]{3}$").unwrap();
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

    pub fn new<T: io::Read + io::Seek>(readable: &mut readable::Readable<T>, tag_version: u8) -> Result<Frame> {
        // head 10 bytes
        let header_bytes = readable.as_bytes(10)?;
        let id = Self::_frame_id(&header_bytes);
        let frame_size = Self::_frame_size(&header_bytes, tag_version);
        let body_bytes = readable.as_bytes(frame_size as usize)?;

        debug!("Frame.new=> frame size: {}", frame_size);
        if frame_size == 0 {
            warn!("Frame.new: frame size is 0!");
        }

        Ok(Frame {
            id: id,
            size: frame_size,
            data: body_bytes,
            // status_flag offset is 8
            status_flag: header_bytes[8],
            // encoding_flag offset is 9
            encoding_flag: header_bytes[9]
        })
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    // @see http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
    pub fn has_flag(&self, flag: FrameHeaderFlag, major_version: u8) -> bool {
        if major_version == 3 {
            match flag {
                FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 7 != 0,
                FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 5 != 0,
                FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 7 != 0,
                FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 5 != 0,
                _ => false
            }
        } else if major_version == 4 {
            match flag {
                FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 5 != 0,
                FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 4 != 0,
                FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 3 != 0,
                FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 2 != 0,
                FrameHeaderFlag::Unsynchronisation => self.encoding_flag & 0x01 << 1 != 0,
                FrameHeaderFlag::DataLength => self.encoding_flag & 0x01 != 0
            }
        } else {
            warn!("Frame.has_flag=> Unknown version!");
            false
        }
    }

    // @see http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
    pub fn get_data(&self) -> result::Result<String, String> {
        let data = self.data.clone().split_off(1);

        Ok(match self.data[0] {
            ISO_8859_1 => encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| err.to_string())?,

            UTF_16LE => encoding::all::UTF_16LE.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| err.to_string())?,

            UTF_16BE => encoding::all::UTF_16BE.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| err.to_string())?,

            UTF_8 => encoding::all::UTF_8.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| err.to_string())?,

            _ => encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| err.to_string())?
        })
    }
}