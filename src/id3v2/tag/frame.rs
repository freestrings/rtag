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
use std::{vec, io, fs, result};

//
// references/id3v2.md#프레임헤더
const ID_REGEX: &'static str = r"^[A-Z][A-Z0-9]{3}$";
const HEAD_LEN: usize = 10;
const FRAME_ID_LEN: usize = 4;
// flag offsets
const STATUS_FLAG_OFFSET: usize = 8;
const ENCODING_FLAG_OFFSET: usize = 9;

// text encoding
const TEXT_ENCODING_OFFSET: usize = 0;
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
    Unsynchronisation,
    //2.4
    DataLength //2.4
}

pub struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8
}

impl Frame {
    fn frame_id(bytes: &vec::Vec<u8>) -> String {
        String::from_utf8_lossy(&bytes[0..4]).into_owned()
    }

    fn frame_size(bytes: &vec::Vec<u8>) -> u32 {
        id3v2::bytes::to_u32(&bytes[4..8])
    }

    pub fn has_next_frame(readable: &mut readable::Readable<fs::File>, header: &id3v2::tag::header::Header) -> bool {
        match readable.as_string(FRAME_ID_LEN) {
            Ok(id) => {
                readable.skip(FRAME_ID_LEN as i64 * -1);
                let re = regex::Regex::new(ID_REGEX).unwrap();
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

    pub fn new(readable: &mut readable::Readable<fs::File>) -> io::Result<Frame> {
        let header_bytes = try!(readable.as_bytes(HEAD_LEN));
        let id = Self::frame_id(&header_bytes);
        let frame_size = Self::frame_size(&header_bytes);
        let body_bytes = try!(readable.as_bytes(frame_size as usize));

        debug!("Frame.new=> frame size: {}", frame_size);
        if frame_size == 0 {
            warn!("Frame.new: frame size is 0!");
        }

        Ok(Frame {
            id: id,
            size: frame_size,
            data: body_bytes,
            status_flag: header_bytes[STATUS_FLAG_OFFSET],
            encoding_flag: header_bytes[ENCODING_FLAG_OFFSET]
        })
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

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

    pub fn get_data(&self) -> result::Result<String, String> {
        let data = self.data.clone().split_off(TEXT_ENCODING_OFFSET + 1);

        if self.data[TEXT_ENCODING_OFFSET] == ISO_8859_1 {
            if let Ok(decoded) = encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[TEXT_ENCODING_OFFSET] == UTF_16LE {
            if let Ok(decoded) = encoding::all::UTF_16LE.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[TEXT_ENCODING_OFFSET] == UTF_16BE {
            if let Ok(decoded) = encoding::all::UTF_16BE.decode(&data, encoding::DecoderTrap::Strict) {
                return Ok(decoded);
            }
        } else if self.data[TEXT_ENCODING_OFFSET] == UTF_8 {
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