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
use readable;
use std::io;
use std::io::Result;

pub struct FrameReader<'a, T: 'a> where T: io::Read + io::Seek {
    reader: TagReader<'a, T>
}

impl<'a, T> FrameReader<'a, T> where T: io::Read + io::Seek {
    pub fn new(readable: &'a mut readable::Readable<T>) -> Result<Self> {
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
    fn next_frame(&mut self) -> Result<id3v2::tag::frame::Frame>;
}

impl<'a, T> FrameIterator for FrameReader<'a, T> where T: io::Read + io::Seek {
    fn has_next_frame(&mut self) -> bool {
        self.reader.has_next_frame()
    }

    fn next_frame(&mut self) -> Result<id3v2::tag::frame::Frame> {
        self.reader.next_frame()
    }
}

pub struct TagReader<'a, T: 'a> where T: io::Read + io::Seek {
    header: id3v2::tag::header::Header,
    readable: &'a mut readable::Readable<T>
}

impl<'a, T> TagReader<'a, T> where T: io::Read + io::Seek {
    pub fn new(mut readable: &'a mut readable::Readable<T>) -> Result<Self> {
        // head 10 bytes
        let header = id3v2::tag::header::Header::new(readable.as_bytes(10)?);
        Ok(TagReader {
            header: header,
            readable: readable
        })
    }

    pub fn get_extended_header(&mut self) -> Result<id3v2::tag::header::ExtendedHeader> {
        if !self.header.has_flag(id3v2::tag::header::HeaderFlag::ExtendedHeader) {
            return Err(io::Error::new(io::ErrorKind::Other, "Extended hader is not exist."));
        }

        // extended header 4bytes
        let head_bytes = self.readable.as_bytes(4)?;
        let size = match self.header.get_version() {
            // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
            3 => id3v2::bytes::to_u32(&head_bytes),
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => id3v2::bytes::to_synchsafe(&head_bytes),
        };

        Ok(id3v2::tag::header::ExtendedHeader::new(size, &self.readable.as_bytes(size as usize)?))
    }
}

impl<'a, T> FrameIterator for TagReader<'a, T> where T: io::Read + io::Seek {
    fn has_next_frame(&mut self) -> bool {
        id3v2::tag::frame::Frame::has_next_frame(&mut self.readable)
    }

    fn next_frame(&mut self) -> io::Result<id3v2::tag::frame::Frame> {
        id3v2::tag::frame::Frame::new(&mut self.readable, self.header.get_version())
    }
}