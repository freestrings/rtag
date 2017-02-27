use std::cmp;
use std::convert::AsRef;
use std::fs::File;
use std::boxed::Box;
use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Result, Write};
use std::vec::Vec;

const DEFAULT_BUF_SIZE: usize = 1024;

type BytesResult = io::Result<Vec<u8>>;
type UnsignedByteResult = io::Result<u8>;
type Unsigned16Result = io::Result<u16>;
type Unsigned32Result = io::Result<u32>;
type StringResult = io::Result<String>;
type UnsignedIntResult = io::Result<usize>;
type VoidResult = io::Result<()>;

pub trait Readable: Read + Seek {
    fn all_bytes(&mut self) -> BytesResult {
        let mut buf = vec![];
        let _ = self.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn all_string(&mut self) -> StringResult {
        let bytes = self.all_bytes()?;
        let ret = String::from_utf8_lossy(&bytes).into_owned();

        Ok(ret)
    }

    fn read_bytes(&mut self, amount: usize) -> BytesResult {
        let mut ret = vec![];

        let buf_size = if amount < DEFAULT_BUF_SIZE {
            amount
        } else {
            DEFAULT_BUF_SIZE
        };

        let mut total_read = 0;
        let mut buf = vec![0u8; buf_size];
        loop {

            if buf.len() == 0 {
                trace!("read skip");
                break;
            }

            let read = self.read(buf.as_mut_slice())?;

            if read == 0 && buf.len() > 0 {
                let err_msg = format!("read try: {}: but fail. (buf size: {})", amount, buf.len());
                warn!("{}", err_msg);
                return Err(Error::new(ErrorKind::Other, err_msg));
            }

            ret.append(&mut buf);

            total_read = total_read + read;

            if total_read >= amount {
                trace!("read done");
                break;
            }

            let remain = amount - total_read;
            buf.resize(if buf_size > remain { remain } else { buf_size }, 0);
        }

        Ok(ret)
    }

    fn read_string(&mut self, amount: usize) -> StringResult {
        let ret = String::from_utf8_lossy(&self.read_bytes(amount)?).into_owned();

        Ok(ret)
    }

    fn read_utf16_bytes(&mut self) -> BytesResult {
        let mut ret = vec![];
        let mut buf = vec![0u8; 1];
        loop {
            let read = self.read(&mut buf)?;

            if read <= 0 {
                break;
            }

            if buf[0] == 0x00 {
                let read = self.read(&mut buf)?;

                if read <= 0 {
                    break;
                }

                if buf[0] == 0x00 {
                    break;
                }

                ret.push(0x00);
                ret.push(buf[0]);
            } else {
                ret.push(buf[0]);
            }
        }

        Ok(ret)
    }

    fn read_utf16_string(&mut self) -> StringResult {
        let bytes = self.read_utf16_bytes()?;
        let ret = String::from_utf8_lossy(&bytes).into_owned();

        Ok(ret)

    }

    fn read_non_utf16_bytes(&mut self) -> BytesResult {
        let mut ret = vec![];
        let mut buf = vec![0u8; 1];
        loop {
            let read = self.read(&mut buf)?;
            if read <= 0 {
                break;
            }

            if buf[0] == 0x00 {
                break;
            } else {
                ret.push(buf[0]);
            }
        }

        Ok(ret)
    }

    fn read_non_utf16_string(&mut self) -> StringResult {
        let bytes = self.read_non_utf16_bytes()?;
        let ret = String::from_utf8_lossy(&bytes).into_owned();

        Ok(ret)
    }

    fn read_u8(&mut self) -> UnsignedByteResult {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u16(&mut self) -> Unsigned16Result {
        let bytes = self.read_bytes(2)?;

        let mut v: u16 = (bytes[1] & 0xff) as u16;
        v = v | ((bytes[0] & 0xff) as u16) << 8;

        Ok(v)
    }

    fn read_u24(&mut self) -> Unsigned32Result {
        let bytes = self.read_bytes(3)?;

        let mut v: u32 = (bytes[2] & 0xff) as u32;
        v = v | ((bytes[1] & 0xff) as u32) << 8;
        v = v | ((bytes[0] & 0xff) as u32) << 16;

        Ok(v)
    }

    fn read_u32(&mut self) -> Unsigned32Result {
        let bytes = self.read_bytes(4)?;

        let mut v: u32 = (bytes[3] & 0xff) as u32;
        v = v | ((bytes[2] & 0xff) as u32) << 8;
        v = v | ((bytes[1] & 0xff) as u32) << 16;
        v = v | ((bytes[0] & 0xff) as u32) << 24;

        Ok(v)
    }

    fn read_synchsafe(&mut self) -> Unsigned32Result {
        let bytes = self.read_bytes(4)?;

        let mut v: u32 = (bytes[3] & 0x7f) as u32;
        v = v | ((bytes[2] & 0x7f) as u32) << 7;
        v = v | ((bytes[1] & 0x7f) as u32) << 14;
        v = v | ((bytes[0] & 0x7f) as u32) << 21;

        Ok(v)
    }

    fn skip_bytes(&mut self, amount: isize) -> UnsignedIntResult {
        let ret = self.seek(SeekFrom::Current(amount as i64))?;

        Ok(ret as usize)
    }

    fn position(&mut self, offset: usize) -> UnsignedIntResult {
        let ret = self.seek(SeekFrom::Start(offset as u64))?;

        Ok(ret as usize)
    }

    fn look_bytes(&mut self, amount: usize) -> BytesResult {
        let v = self.read_bytes(amount)?;
        let _ = self.skip_bytes((amount as isize) * -1)?;

        Ok(v)
    }

    fn look_string(&mut self, amount: usize) -> StringResult {
        let v = self.read_string(amount)?;
        let _ = self.skip_bytes((amount as isize) * -1)?;

        Ok(v)
    }

    fn look_u8(&mut self) -> UnsignedByteResult {
        let v = self.read_u8()?;
        let _ = self.skip_bytes(-1)?;

        Ok(v)
    }

    fn look_u16(&mut self) -> Unsigned16Result {
        let v = self.read_u16()?;
        let _ = self.skip_bytes(-2)?;

        Ok(v)
    }

    fn look_u24(&mut self) -> Unsigned32Result {
        let v = self.read_u24()?;
        let _ = self.skip_bytes(-3)?;

        Ok(v)
    }

    fn look_u32(&mut self) -> Unsigned32Result {
        let v = self.read_u32()?;
        let _ = self.skip_bytes(-4)?;

        Ok(v)
    }

    fn look_synchsafe(&mut self) -> Unsigned32Result {
        let v = self.read_synchsafe()?;
        let _ = self.skip_bytes(-4)?;

        Ok(v)
    }

    fn to_readable(&mut self, amount: usize) -> Result<Cursor<Vec<u8>>> {
        Ok(Cursor::new(self.read_bytes(amount)?))
    }

    fn to_synchronize(&mut self, amount: usize) -> Result<Vec<u8>> {
        let mut bytes = self.read_bytes(amount)?;

        let mut copy = true;
        let mut to = 0;
        for i in 0..bytes.len() {
            let b = bytes[i];
            if copy || b != 0 {
                bytes[to] = b;
                to = to + 1
            }
            copy = (b & 0xff) != 0xff;
        }
        bytes.split_off(to);

        Ok(bytes)
    }

    fn to_unsynchronize(&mut self, amount: usize) -> Result<Vec<u8>> {
        let bytes = self.read_bytes(amount)?;

        fn require_unsync(bytes: &Vec<u8>) -> usize {
            let mut count = 0;
            let len = bytes.len();
            for i in 0..len - 1 {
                if bytes[i] & 0xff == 0xff && (bytes[i + 1] & 0xe0 == 0xe0 || bytes[i + 1] == 0) {
                    count = count + 1;
                }
            }
            if len > 0 && bytes[len - 1] == 0xff {
                count = count + 1;
            }
            count
        }

        let count = require_unsync(&bytes);
        if count == 0 {
            return Ok(bytes);
        }

        let len = bytes.len();
        let mut out = vec![0u8; len + count];
        let mut j = 0;
        for i in 0..len - 1 {
            out[j] = bytes[i];
            j = j + 1;
            if bytes[i] & 0xff == 0xff && (bytes[i + 1] & 0xe0 == 0xe0 || bytes[i + 1] == 0) {
                out[j] = 0;
                j = j + 1;
            }
        }
        out[j] = bytes[len - 1];
        j = j + 1;
        if bytes[len - 1] == 0xff {
            out[j] = 0;
        }

        Ok(out)
    }
}

pub trait Writable: Readable + Write {
    fn write_u8(&mut self, v: u8) -> VoidResult {
        let mut buf = [v];
        let _ = self.write(&mut buf);

        Ok(())
    }

    fn write_u16(&mut self, v: u16) -> VoidResult {
        let b1: u8 = ((v >> 8) & 0xff) as u8;
        let b2: u8 = (v & 0xff) as u8;
        let mut buf = [b1, b2];
        let _ = self.write(&mut buf);

        Ok(())
    }

    fn write_u24(&mut self, v: u32) -> VoidResult {
        let b1: u8 = ((v >> 16) & 0xff) as u8;
        let b2: u8 = ((v >> 8) & 0xff) as u8;
        let b3: u8 = (v & 0xff) as u8;
        let mut buf = [b1, b2, b3];
        let _ = self.write(&mut buf);

        Ok(())
    }

    fn write_u32(&mut self, v: u32) -> VoidResult {
        let b1: u8 = ((v >> 24) & 0xff) as u8;
        let b2: u8 = ((v >> 16) & 0xff) as u8;
        let b3: u8 = ((v >> 8) & 0xff) as u8;
        let b4: u8 = (v & 0xff) as u8;
        let mut buf = [b1, b2, b3, b4];
        let _ = self.write(&mut buf);

        Ok(())
    }

    fn write_synchsafe(&mut self, v: u32) -> VoidResult {
        let b1: u8 = ((v >> 21) & 0x7f) as u8;
        let b2: u8 = ((v >> 14) & 0x7f) as u8;
        let b3: u8 = ((v >> 7) & 0x7f) as u8;
        let b4: u8 = (v & 0x7f) as u8;
        let _ = self.write(&[b1, b2, b3, b4]);

        Ok(())
    }

    fn write_string(&mut self, v: &str) -> VoidResult {
        let b = v.as_bytes();
        self.write(&b)?;

        Ok(())
    }

    fn write_utf16_string(&mut self, v: &str) -> VoidResult {
        self.write_string(v)?;
        self.write_u8(0)?;
        self.write_u8(0)
    }

    fn write_non_utf16_string(&mut self, v: &str) -> VoidResult {
        self.write_string(v)?;
        self.write_u8(0)
    }

    fn unshift(&mut self, amount: usize) -> VoidResult {
        if amount == 0 {
            return Ok(());
        }

        // remember current position
        let curr_pos = self.seek(SeekFrom::Current(0))?;
        let end_pos = self.seek(SeekFrom::End(0))?;
        let mut related_pos = curr_pos + amount as u64;
        let mut buf = vec![];

        loop {
            if related_pos > end_pos {
                break;
            }

            let buf_size = cmp::min((end_pos - related_pos) as usize, DEFAULT_BUF_SIZE);
            if buf_size == 0 {
                break;
            }

            buf.resize(buf_size, 0);

            self.seek(SeekFrom::Start(related_pos))?;
            self.read(&mut buf)?;
            self.seek(SeekFrom::Start(related_pos - amount as u64))?;
            self.write(&buf)?;

            related_pos = related_pos + buf_size as u64;
        }
        self.seek(SeekFrom::End(amount as i64 * -1))?;

        // fill zero
        let mut buf = vec![0u8; amount];
        self.write(&mut buf)?;

        self.seek(SeekFrom::Start(curr_pos))?;

        Ok(())
    }

    //
    // Note, it need that a 'read' file permission.
    //
    fn shift(&mut self, amount: usize) -> VoidResult {
        if amount == 0 {
            return Ok(());
        }

        // remember current position
        let curr_pos = self.seek(SeekFrom::Current(0))?;
        let mut end_pos = self.seek(SeekFrom::End(0))?;
        // append empty buffer which have inserted size to end
        self.write(&mut vec![0u8; amount])?;
        // a last_position buffer to write
        let mut last_pos = end_pos + amount as u64;

        let mut buf = vec![];
        loop {
            if end_pos < curr_pos {
                break;
            }

            let remain = (end_pos - curr_pos) as usize;
            let buf_size = cmp::min(DEFAULT_BUF_SIZE, remain);

            if buf_size == 0 {
                break;
            }

            buf.resize(buf_size, 0);

            end_pos = end_pos - buf_size as u64;
            last_pos = last_pos - buf_size as u64;

            self.seek(SeekFrom::Start(end_pos))?;
            self.read(&mut buf)?;
            self.seek(SeekFrom::Start(last_pos))?;
            self.write(&buf)?;
        }
        self.seek(SeekFrom::Start(curr_pos))?;

        Ok(())
    }

    fn copy(&mut self, bytes: &mut Vec<u8>) -> VoidResult {
        let curr = self.seek(SeekFrom::Current(0))?;
        self.seek(SeekFrom::Start(0))?;
        self.read_to_end(bytes)?;
        self.seek(SeekFrom::Start(curr))?;

        Ok(())
    }
}

impl Readable for File {}
impl<T> Readable for Cursor<T> where T: AsRef<[u8]> {}

impl Writable for File {}
impl<'a> Writable for Cursor<&'a mut [u8]> {}
impl Writable for Cursor<Vec<u8>> {}
impl Writable for Cursor<Box<[u8]>> {}