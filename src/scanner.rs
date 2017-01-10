use std::{fs, vec, io};
use std::io::{Read, Seek, SeekFrom};

pub struct Scanner {
    file: fs::File,
    len: u64,
    offset: usize
}

impl Scanner {
    pub fn new(file_path: &'static str) -> io::Result<Self> {
        let file = try!(fs::File::open(file_path));
        let metadata = try!(file.metadata());
        Ok(Scanner { file: file, offset: 0, len: metadata.len() })
    }

    pub fn read_as_bytes(&mut self, amount: usize) -> io::Result<vec::Vec<u8>> {
        let mut buf = vec![0u8; amount];
        let read = try!(self.file.read(buf.as_mut_slice()));
        if read < amount {
            buf.split_off(read);
        }
        self.offset = self.offset + read;
        trace!("Scanner.read=> amount:{}, offset:{}", amount, self.offset);
        Ok(buf)
    }

    pub fn read_as_string(&mut self, amount: usize) -> io::Result<String> {
        let bytes = try!(self.read_as_bytes(amount));
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn _seek(&mut self, amount: i64) -> io::Result<u64> {
        let seek = try!(self.file.seek(SeekFrom::Current(amount)));
        self.offset = seek as usize;
        Ok(seek)
    }

    pub fn skip(&mut self, amount: u64) -> io::Result<u64> {
        let skip = self._seek(amount as i64);
        trace!("Scanner.skip=> amount:{}, offset:{}", amount, self.offset);
        skip
    }

    pub fn rewind(&mut self, amount: u64) -> io::Result<u64> {
        let rewind = self._seek(amount as i64 * -1);
        trace!("Scanner.rewind=> amount:{}, offset:{}", amount, self.offset);
        rewind
    }

    pub fn has_next(&mut self) -> bool {
        trace!("Scanner.has_next=> len:{}, offset:{}", self.len, self.offset);
        self.len > self.offset as u64
    }

    pub fn get_len(&mut self) -> u64 {
        self.len
    }
}