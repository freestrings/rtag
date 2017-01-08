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
        debug!("read=> amount:{}, offset:{}", amount, self.offset);
        Ok(buf)
    }

    fn read_as_string(&mut self, amount: usize) -> io::Result<String> {
        let bytes = try!(self.read_as_bytes(amount));
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn _seek(&mut self, amount: i64) -> io::Result<u64> {
        let seek = try!(self.file.seek(SeekFrom::Current(amount)));
        self.offset = seek as usize;
        trace!("_seek=> amount:{}, offset:{}", amount, self.offset);
        Ok(seek)
    }

    fn skip(&mut self, amount: u64) -> io::Result<u64> {
        let skip = self._seek(amount as i64);
        debug!("skip=> amount:{}, offset:{}", amount, self.offset);
        skip
    }

    fn rewind(&mut self, amount: u64) -> io::Result<u64> {
        let rewind = self._seek(amount as i64 * -1);
        debug!("rewind=> amount:{}, offset:{}", amount, self.offset);
        rewind
    }

    fn has_next(&mut self) -> bool {
        debug!("has_next=> len:{}, offset:{}", self.len, self.offset);
        self.len > self.offset as u64
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn forwardscanner() {
        match super::Scanner::new("./resources/file1.txt") {
            Ok(mut scanner) => {
                assert_eq!(match scanner.read_as_bytes(10) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "1234567890");
                assert!(scanner.has_next());
                assert!(scanner.skip(5).is_ok());
                assert!(scanner.has_next());
                assert!(scanner.rewind(5).is_ok());
                assert_eq!(match scanner.read_as_bytes(15) {
                    Ok(ref bytes) => String::from_utf8_lossy(bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "abcdefghij");
                assert!(!scanner.has_next());
            },
            Err(_) => assert!(false)
        }
    }
}