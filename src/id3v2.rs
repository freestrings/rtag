use std::{fs, vec, io};
use std::io::{Read, Seek, SeekFrom};

struct ForwardScanner {
    file: fs::File,
    len: u64,
    offset: usize
}

impl ForwardScanner {
    fn new(file_path: &'static str) -> io::Result<Self> {
        let file = try!(fs::File::open(file_path));
        let metadata = try!(file.metadata());
        Ok(ForwardScanner { file: file, offset: 0, len: metadata.len() })
    }

    fn read(&mut self, amount: usize) -> io::Result<vec::Vec<u8>> {
        let mut buf = vec![0u8; amount];
        let read = try!(self.file.read(buf.as_mut_slice()));
        if read < amount {
            buf.split_off(read);
        }
        self.offset = self.offset + read;
        debug!("read=> amount:{}, offset:{}", amount, self.offset);
        Ok(buf)
    }

    fn skip(&mut self, amount: i64) -> io::Result<u64> {
        let skip = try!(self.file.seek(SeekFrom::Current(amount)));
        self.offset = skip as usize;
        debug!("skip=> amount:{}, offset:{}", amount, self.offset);
        Ok(skip)
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
        match super::ForwardScanner::new("./resources/file1.txt") {
            Ok(mut scanner) => {
                assert_eq!(match scanner.read(10) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "1234567890");
                assert!(scanner.has_next());
                assert!(scanner.skip(5).is_ok());
                assert!(scanner.has_next());
                assert_eq!(match scanner.read(15) {
                    Ok(ref bytes) => String::from_utf8_lossy(bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "fghij");
                assert!(!scanner.has_next());
            },
            Err(_) => assert!(false)
        }
    }
}