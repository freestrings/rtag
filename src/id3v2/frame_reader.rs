extern crate regex;

use id3v2;
use std::io;

pub struct FrameReader<'a> {
    tag_header: id3v2::tag_header::TagHeader,
    has_error: bool,
    scanner: &'a mut id3v2::scanner::Scanner
}

impl<'a> FrameReader<'a> {
    pub fn new(scanner: &'a mut id3v2::scanner::Scanner) -> io::Result<Self> {
        let bytes = try!(scanner.read_as_bytes(10));
        Ok(FrameReader {
            tag_header: id3v2::tag_header::TagHeader::new(bytes),
            has_error: false,
            scanner: scanner
        })
    }

    pub fn get_header(&mut self) -> &id3v2::tag_header::TagHeader {
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

    pub fn next_frame(&mut self) -> io::Result<id3v2::frame::Frame> {
        id3v2::frame::Frame::new(self.scanner)
    }
}