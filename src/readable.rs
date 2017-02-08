use std::io::{
    Cursor,
    Error,
    ErrorKind,
    Read,
    Seek,
    SeekFrom,
    Result
};
use std::vec::Vec;

const DEFAULT_BUF_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Readable<I> where I: Read + Seek {
    input: I,
    total: i64
}

impl<I> Readable<I> where I: Read + Seek {
    pub fn new(input: I) -> Self {
        Readable {
            input: input,
            total: 0
        }
    }

    pub fn all_bytes(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![];
        let read = self.input.read_to_end(&mut buf)?;
        self.total = self.total + read as i64;

        Ok(buf)
    }

    pub fn all_string(&mut self) -> Result<String> {
        let bytes = self.all_bytes()?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn bytes(&mut self, amount: usize) -> Result<Vec<u8>> {
        let mut ret = vec![];
        let buf_size = if amount < DEFAULT_BUF_SIZE {
            amount
        } else {
            DEFAULT_BUF_SIZE
        };
        let mut buf = vec![0u8; buf_size];
        let mut total_read = 0;
        loop {
            let read = self.input.read(buf.as_mut_slice())?;

            self.total = self.total + read as i64;

            if read <= 0 {
                return Err(Error::new(ErrorKind::Other,
                                      format!("read try: {}, but read zero.",
                                              amount)));
            }
            ret.append(&mut buf);
            total_read = total_read + read;
            if total_read >= amount {
                break;
            }
            let remain = amount - total_read;
            buf.resize(if buf_size > remain { remain } else { buf_size }, 0);
        }

        Ok(ret)
    }

    pub fn string(&mut self, amount: usize) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.bytes(amount)?).into_owned())
    }

    pub fn utf16_bytes(&mut self) -> Result<Vec<u8>> {
        let mut ret = vec![];
        let mut read_all = 0;
        let mut buf = vec![0u8; 1];
        loop {
            let read = self.input.read(&mut buf)?;
            if read <= 0 {
                break;
            }
            read_all = read_all + read;
            if buf[0] == 0x00 {
                read_all = read_all + self.input.read(&mut buf)?;
                if buf[0] == 0x00 {
                    break;
                }
                ret.push(0x00);
                ret.push(buf[0]);
            } else {
                ret.push(buf[0]);
            }
        }

        let len = ret.len();
        self.total = self.total + len as i64;

        Ok(ret)
    }

    // <text>0x00 0x00
    pub fn utf16_string(&mut self) -> Result<String> {
        let ret = self.utf16_bytes()?;
        Ok(String::from_utf8_lossy(&ret).into_owned())
    }

    // <text>0x00
    pub fn non_utf16_bytes(&mut self) -> Result<Vec<u8>> {
        let mut ret = vec![];
        let mut read_all = 0;
        let mut buf = vec![0u8; 1];
        loop {
            let read = self.input.read(&mut buf)?;
            if read <= 0 {
                break;
            }
            read_all = read_all + read;
            if buf[0] == 0x00 {
                break;
            } else {
                ret.push(buf[0]);
            }
        }

        let len = ret.len();
        self.total = self.total + len as i64;

        Ok(ret)
    }

    pub fn non_utf16_string(&mut self) -> Result<String> {
        let ret = self.non_utf16_bytes()?;
        Ok(String::from_utf8_lossy(&ret).into_owned())
    }

    pub fn skip(&mut self, amount: i64) -> Result<u64> {
        let ret = self.input.seek(SeekFrom::Current(amount))?;
        self.total = self.total + amount;

        Ok(ret)
    }

    pub fn position(&mut self, offset: usize) -> Result<u64> {
        let ret = self.input.seek(SeekFrom::Start(offset as u64))?;
        self.total = offset as i64;

        Ok(ret)
    }

    pub fn total_read(&mut self) -> i64 {
        self.total
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16> {
        let bytes = self.bytes(2)?;

        let mut v: u16 = (bytes[1] & 0xff) as u16;
        v = v | ((bytes[0] & 0xff) as u16) << 8;

        Ok(v)
    }

    pub fn u24(&mut self) -> Result<u32> {
        let bytes = self.bytes(3)?;

        let mut v: u32 = (bytes[2] & 0xff) as u32;
        v = v | ((bytes[1] & 0xff) as u32) << 8;
        v = v | ((bytes[0] & 0xff) as u32) << 16;

        Ok(v)
    }

    pub fn u32(&mut self) -> Result<u32> {
        let bytes = self.bytes(4)?;

        let mut v: u32 = (bytes[3] & 0xff) as u32;
        v = v | ((bytes[2] & 0xff) as u32) << 8;
        v = v | ((bytes[1] & 0xff) as u32) << 16;
        v = v | ((bytes[0] & 0xff) as u32) << 24;

        Ok(v)
    }

    // Sizes are 4bytes long big-endian but first bit is 0
    // @see http://id3.org/id3v2.4.0-structure > 6.2. Synchsafe integers
    pub fn synchsafe(&mut self) -> Result<u32> {
        let bytes = self.bytes(4)?;

        let mut v: u32 = (bytes[3] & 0x7f) as u32;
        v = v | ((bytes[2] & 0x7f) as u32) << 7;
        v = v | ((bytes[1] & 0x7f) as u32) << 14;
        v = v | ((bytes[0] & 0x7f) as u32) << 21;

        Ok(v)
    }

    pub fn look_bytes(&mut self, amount: usize) -> Result<Vec<u8>> {
        let v = self.bytes(amount)?;
        let _ = self.skip((amount as i64) * -1)?;

        Ok(v)
    }

    pub fn look_string(&mut self, amount: usize) -> Result<String> {
        let v = self.string(amount)?;
        let _ = self.skip((amount as i64) * -1)?;

        Ok(v)
    }

    pub fn look_u8(&mut self) -> Result<u8> {
        let v = self.u8()?;
        let _ = self.skip(-1)?;

        Ok(v)
    }

    pub fn look_u16(&mut self) -> Result<u16> {
        let v = self.u16()?;
        let _ = self.skip(-2)?;

        Ok(v)
    }

    pub fn look_u32(&mut self) -> Result<u32> {
        let v = self.u32()?;
        let _ = self.skip(-4)?;

        Ok(v)
    }

    pub fn look_u24(&mut self) -> Result<u32> {
        let v = self.u24()?;
        let _ = self.skip(-3)?;

        Ok(v)
    }

    pub fn look_synchsafe(&mut self) -> Result<u32> {
        let v = self.synchsafe()?;
        let _ = self.skip(-4)?;

        Ok(v)
    }

    pub fn to_readable(&mut self, amount: usize) -> Result<Readable<Cursor<Vec<u8>>>> {
        let bytes = self.bytes(amount)?;
        Ok(Cursor::new(bytes).to_readable())
    }
}

pub trait ReadableFactory<T> where T: Read + Seek {
    fn to_readable(self) -> Readable<T>;
}

impl<T: Read + Seek> ReadableFactory<T> for T {
    fn to_readable(self) -> Readable<T> {
        Readable::new(self)
    }
}