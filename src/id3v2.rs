extern crate regex;

use std::{fs, vec, io};
use std::io::{Read, Seek, SeekFrom};

struct Scanner {
    file: fs::File,
    len: u64,
    offset: usize
}

impl Scanner {
    fn new(file_path: &'static str) -> io::Result<Self> {
        let file = try!(fs::File::open(file_path));
        let metadata = try!(file.metadata());
        Ok(Scanner { file: file, offset: 0, len: metadata.len() })
    }

    fn read_as_bytes(&mut self, amount: usize) -> io::Result<vec::Vec<u8>> {
        let mut buf = vec![0u8; amount];
        let read = try!(self.file.read(buf.as_mut_slice()));
        if read < amount {
            buf.split_off(read);
        }
        self.offset = self.offset + read;
        trace!("Scanner.read=> amount:{}, offset:{}", amount, self.offset);
        Ok(buf)
    }

    fn read_as_string(&mut self, amount: usize) -> io::Result<String> {
        let bytes = try!(self.read_as_bytes(amount));
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn _seek(&mut self, amount: i64) -> io::Result<u64> {
        let seek = try!(self.file.seek(SeekFrom::Current(amount)));
        self.offset = seek as usize;
        Ok(seek)
    }

    fn skip(&mut self, amount: u64) -> io::Result<u64> {
        let skip = self._seek(amount as i64);
        trace!("Scanner.skip=> amount:{}, offset:{}", amount, self.offset);
        skip
    }

    fn rewind(&mut self, amount: u64) -> io::Result<u64> {
        let rewind = self._seek(amount as i64 * -1);
        trace!("Scanner.rewind=> amount:{}, offset:{}", amount, self.offset);
        rewind
    }

    fn has_next(&mut self) -> bool {
        trace!("Scanner.has_next=> len:{}, offset:{}", self.len, self.offset);
        self.len > self.offset as u64
    }
}

struct FrameReader<'a> {
    tag_header: TagHeader,
    has_error: bool,
    scanner: &'a mut Scanner
}

impl<'a> FrameReader<'a> {
    fn new(scanner: &'a mut Scanner) -> io::Result<Self> {
        let bytes = try!(scanner.read_as_bytes(10));
        Ok(FrameReader {
            tag_header: TagHeader::new(bytes),
            has_error: false,
            scanner: scanner
        })
    }

    fn get_header(&mut self) -> &TagHeader {
        &self.tag_header
    }

    fn has_next_frame(&mut self) -> bool {
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

    fn next_frame(&mut self) -> io::Result<Frame> {
        Frame::new(self.scanner)
    }
}

struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8
}

impl Frame {
    fn new(scanner: &mut Scanner) -> io::Result<Frame> {
        fn to_u32(bytes: &[u8]) -> u32 {
            let mut v: u32 = (bytes[3] & 0xff) as u32;
            v = v | ((bytes[2] & 0xff) as u32) << 8;
            v = v | ((bytes[1] & 0xff) as u32) << 16;
            v = v | ((bytes[0] & 0xff) as u32) << 24;
            v
        }

        let frame_header_bytes = try!(scanner.read_as_bytes(10));
        let frame_size = to_u32(&frame_header_bytes[4..8]);
        trace!("Frame.new=> size: {}", frame_size);

        if frame_size == 0 {
            warn!("FrameReader.next_frame: frame size is zero!");
        }

        let id = String::from_utf8_lossy(&frame_header_bytes[0..4]).into_owned();
        let frame_body_bytes = try!(scanner.read_as_bytes(frame_size as usize));

        Ok(Frame {
            id: id,
            size: frame_size,
            data: frame_body_bytes,
            status_flag: frame_header_bytes[8],
            encoding_flag: frame_header_bytes[9]
        })
    }

    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_size(&self) -> u32 {
        self.size
    }

    fn has_preserve_tag(&self) -> bool {
        self.status_flag & 0x01 << 7 != 0
    }

    fn has_preserve_file(&self) -> bool {
        self.status_flag & 0x01 << 6 != 0
    }

    fn has_readonly(&self) -> bool {
        self.status_flag & 0x01 << 5 != 0
    }

    fn has_compression(&self) -> bool {
        self.encoding_flag & 0x01 << 7 != 0
    }

    fn has_encryption(&self) -> bool {
        self.encoding_flag & 0x01 << 6 != 0
    }

    fn has_group(&self) -> bool {
        self.encoding_flag & 0x01 << 5 != 0
    }
}

struct TagHeader {
    version: u8,
    minor_version: u8,
    header_flag: u8,
    size: u32
}

impl TagHeader {
    fn new(bytes: vec::Vec<u8>) -> Self {
        if !(bytes[0] as char == 'I' && bytes[1] as char == 'D' && bytes[2] as char == '3') {
            debug!("Invalid IDv2: `{}`", String::from_utf8_lossy(&bytes[0..4]));
            return TagHeader {
                version: 0, minor_version: 0, header_flag: 0, size: 0
            };
        }

        // Sizes are 4bytes long big-endian but first bit is 0
        fn to_synchsafe(bytes: &[u8]) -> u32 {
            let mut v: u32 = (bytes[3] & 0x7f) as u32;
            v = v | ((bytes[2] & 0x7f) as u32) << 7;
            v = v | ((bytes[1] & 0x7f) as u32) << 14;
            v = v | ((bytes[0] & 0x7f) as u32) << 21;
            v
        }

        let version = bytes[3] as u8;
        let minor_version = bytes[4] as u8;
        let header_flag = bytes[5] as u8;
        let size = to_synchsafe(&bytes[6..10]);

        TagHeader {
            version: version, minor_version: minor_version, header_flag: header_flag, size: size
        }
    }

    fn get_version(&self) -> u8 {
        self.version
    }

    fn get_minor_version(&self) -> u8 {
        self.minor_version
    }

    fn has_unsynchronisation(&self) -> bool {
        self.header_flag & 0x01 << 7 != 0
    }

    fn has_extended(&self) -> bool {
        self.header_flag & 0x01 << 6 != 0
    }

    fn has_experimental(&self) -> bool {
        self.header_flag & 0x01 << 5 != 0
    }

    fn get_size(&self) -> u32 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    #[test]
    fn forwardscanner() {
        let _ = env_logger::init();

        match super::Scanner::new("./resources/file1.txt") {
            Ok(mut scanner) => {
                assert_eq! ( match scanner.read_as_bytes(10) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "1234567890");
                assert! (scanner.has_next());
                assert! (scanner.skip(5).is_ok());
                assert! (scanner.has_next());
                assert! (scanner.rewind(5).is_ok());
                assert_eq! ( match scanner.read_as_bytes(15) {
                    Ok(ref bytes) => String::from_utf8_lossy(bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "abcdefghij");
                assert! ( !scanner.has_next());
            },
            Err(_) => assert! ( false)
        }
    }

    #[test]
    fn idv3_230_header() {
        let _ = env_logger::init();

        match super::Scanner::new("./resources/230.mp3") {
            Ok(mut scanner) => {
                if let Ok(bytes) = scanner.read_as_bytes(10) {
                    let tag_header = super::TagHeader::new(bytes);
                    assert_eq!(tag_header.get_version(), 3);
                    assert_eq!(tag_header.get_minor_version(), 0);
                    assert_eq!(tag_header.has_unsynchronisation(), false);
                    assert_eq!(tag_header.has_extended(), false);
                    assert_eq!(tag_header.has_experimental(), false);
                    assert_eq!(tag_header.get_size(), 1182);
                }
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn idv3_230_frame_reader() {
        let _ = env_logger::init();

        match super::Scanner::new("./resources/ID3v1-ID3v2.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::FrameReader::new(&mut scanner) {
                    let mut v = vec!["TIT2", "TPE1", "TALB", "TPE2", "TCON", "COMM", "TRCK", "TPOS"];
                    v.reverse();
                    loop {
                        if frame_reader.has_next_frame() {
                            if let Ok(frame) = frame_reader.next_frame() {
                                assert_eq!(v.pop().unwrap(), frame.get_id())
                            }
                        } else {
                            break;
                        }
                    }
                }
            },
            Err(_) => assert!(false)
        }
    }
}