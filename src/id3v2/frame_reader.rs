extern crate regex;

use std::io;
use id3v2::frame::Frame;
use id3v2::scanner::Scanner;
use id3v2::tag_header::TagHeader;

pub struct FrameReader<'a> {
    tag_header: TagHeader,
    has_error: bool,
    scanner: &'a mut Scanner
}

impl<'a> FrameReader<'a> {
    pub fn new(scanner: &'a mut Scanner) -> io::Result<Self> {
        let bytes = try!(scanner.read_as_bytes(10));
        Ok(FrameReader {
            tag_header: TagHeader::new(bytes),
            has_error: false,
            scanner: scanner
        })
    }

    pub fn get_header(&mut self) -> &TagHeader {
        &self.tag_header
    }

    pub fn has_next_frame(&mut self) -> bool {
        match self.scanner.read_as_string(4) {
            Ok(id) => {
                let re = regex::Regex::new(r"^[A-Z][A-Z0-9]{3}$").unwrap();
                self.scanner.rewind(4);
                let matched = re.is_match(&id);
                debug!("FrameReader.has_next_frame=> FRAME Id {}:{}", id, matched);
                matched
            },
            Err(_) => {
                debug!("FrameReader.has_next_frame=> Fail");
                false
            }
        }
    }

    pub fn next_frame(&mut self) -> io::Result<Frame> {
        Frame::new(self.scanner)
    }
}