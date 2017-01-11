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

use std::io::Result;
use std::{io, vec};
use std::io::{Read, Seek, SeekFrom};

pub struct Readable<I: io::Read + io::Seek> {
    input: io::BufReader<I>
}

impl<I: io::Read + io::Seek> Readable<I> {
    pub fn new(input: I) -> Self {
        Readable {
            input: io::BufReader::new(input)
        }
    }

    pub fn as_bytes(&mut self, amount: usize) -> Result<vec::Vec<u8>> {
        let mut buf = vec![0u8; amount];
        let read = self.input.read(buf.as_mut_slice())?;
        if read == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "read by 0"));
        }
        if read < amount {
            buf.split_off(read);
        }
        Ok(buf)
    }

    pub fn as_string(&mut self, amount: usize) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.as_bytes(amount)?).into_owned())
    }

    pub fn skip(&mut self, amount: i64) -> Result<u64> {
        Ok(self.input.seek(SeekFrom::Current(amount))?)
    }

    pub fn position(&mut self, offset: u64) -> Result<u64> {
        Ok(self.input.seek(SeekFrom::Start(offset))?)
    }
}

pub mod utility {
    use std::{fs, io};
    use std::io::Result;

    pub fn from_path(path: &'static str) -> Result<super::Readable<fs::File>> {
        Ok(super::Readable::new(fs::File::open(path)?))
    }

    pub fn from_string(str: &'static str) -> Result<super::Readable<io::Cursor<String>>> {
        Ok(super::Readable::new(io::Cursor::new(str.to_string())))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bytes() {
        let valid = "0123456789";
        if let Ok(mut readable) = super::utility::from_string(valid) {
            assert!(readable.as_bytes(10).is_ok());
            assert!(readable.as_bytes(10).is_err());
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_file() {
        if let Ok(mut readable) = super::utility::from_path("./resources/file1.txt") {
            assert!(readable.as_bytes(10).is_ok());
            assert!(readable.as_bytes(10).is_ok());
            assert!(readable.skip(-5).is_ok());
            assert_eq!(readable.as_string(10).unwrap(), "fghij");
            assert!(readable.as_bytes(10).is_err());
        } else {
            assert!(false);
        }
    }
}