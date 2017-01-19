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

use std::io;
use std::error::Error;

type ReadResult<T> = ::std::result::Result<T, ::errors::ParsingError>;

pub struct FrameReader<'a, T: 'a> where T: io::Read + io::Seek {
    reader: TagReader<'a, T>
}

impl<'a, T> FrameReader<'a, T> where T: io::Read + io::Seek {
    pub fn new(readable: &'a mut ::readable::Readable<T>) -> ReadResult<Self> {
        let mut reader = TagReader::new(readable)?;
        // skip extended header
        reader.get_extended_header();

        Ok(FrameReader {
            reader: reader
        })
    }
}

pub trait FrameIterator {
    fn has_next_frame(&mut self) -> bool;
    fn next_frame(&mut self) -> ReadResult<::id3v2::frame::Frame>;
}

impl<'a, T> FrameIterator for FrameReader<'a, T> where T: io::Read + io::Seek {
    fn has_next_frame(&mut self) -> bool {
        self.reader.has_next_frame()
    }

    fn next_frame(&mut self) -> ReadResult<::id3v2::frame::Frame> {
        self.reader.next_frame()
    }
}

impl<'a, T> ::std::iter::Iterator for FrameReader<'a, T> where T: io::Read + io::Seek {

    type Item = ::id3v2::frame::Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.has_next_frame() {
            match self.reader.next_frame() {
                Ok(frame) => Some(frame),
                Err(err) => {
                    error!("{}", err.description());
                    None
                }
            }
        } else {
            None
        }
    }
}

pub struct TagReader<'a, T: 'a> where T: io::Read + io::Seek {
    header: ::id3v2::header::Header,
    readable: &'a mut ::readable::Readable<T>
}

impl<'a, T> TagReader<'a, T> where T: io::Read + io::Seek {
    pub fn new(mut readable: &'a mut ::readable::Readable<T>) -> ReadResult<Self> {
        // head 10 bytes
        let header = ::id3v2::header::Header::new(readable.as_bytes(10)?)?;
        Ok(TagReader {
            header: header,
            readable: readable
        })
    }

    pub fn get_extended_header(&mut self) -> ReadResult<::id3v2::header::ExtendedHeader> {
        if !self.header.has_flag(::id3v2::header::HeaderFlag::ExtendedHeader) {
            return Err(::errors::ParsingError::BadData("Extended header does not exist.".to_string()));
        }

        // extended header 4bytes
        let head_bytes = self.readable.as_bytes(4)?;
        let size = match self.header.get_version() {
            // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
            3 => ::id3v2::bytes::to_u32(&head_bytes),
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => ::id3v2::bytes::to_synchsafe(&head_bytes),
        };

        Ok(::id3v2::header::ExtendedHeader::new(size, &self.readable.as_bytes(size as usize)?))
    }
}

impl<'a, T> FrameIterator for TagReader<'a, T> where T: io::Read + io::Seek {
    fn has_next_frame(&mut self) -> bool {
        ::id3v2::frame::Frame::has_next_frame(&mut self.readable)
    }

    fn next_frame(&mut self) -> ReadResult<::id3v2::frame::Frame> {
        ::id3v2::frame::Frame::new(&mut self.readable, self.header.get_version())
    }
}