use std::io::{
    Seek,
    SeekFrom,
    Result,
    Write
};

pub struct Writable<I> where I: Write + Seek {
    input: I,
    total: i64
}

impl<I> Writable<I> where I: Write + Seek {
    pub fn new(input: I) -> Self {
        Writable {
            input: input,
            total: 0
        }
    }

    pub fn u8(&mut self, v: u8) -> Result<()> {
        let mut buf = [v];
        let _ = self.input.write(&mut buf);
        self.total = self.total + 1;

        Ok(())
    }

    pub fn u16(&mut self, v: u16) -> Result<()> {
        let b1: u8 = ((v >> 8) & 0xff) as u8;
        let b2: u8 = (v & 0xff) as u8;
        let mut buf = [b1, b2];
        let _ = self.input.write(&mut buf);
        self.total = self.total + 2;

        Ok(())
    }

    pub fn u24(&mut self, v: u32) -> Result<()> {
        let b1: u8 = ((v >> 16) & 0xff) as u8;
        let b2: u8 = ((v >> 8) & 0xff) as u8;
        let b3: u8 = (v & 0xff) as u8;
        let mut buf = [b1, b2, b3];
        let _ = self.input.write(&mut buf);
        self.total = self.total + 3;

        Ok(())
    }

    pub fn u32(&mut self, v: u32) -> Result<()> {
        let b1: u8 = ((v >> 24) & 0xff) as u8;
        let b2: u8 = ((v >> 16) & 0xff) as u8;
        let b3: u8 = ((v >> 8) & 0xff) as u8;
        let b4: u8 = (v & 0xff) as u8;
        let mut buf = [b1, b2, b3, b4];
        let _ = self.input.write(&mut buf);
        self.total = self.total + 4;

        Ok(())
    }

    pub fn synchsafe(&mut self, v: u32) -> Result<()> {
        let b1: u8 = ((v >> 21) & 0x7f) as u8;
        let b2: u8 = ((v >> 14) & 0x7f) as u8;
        let b3: u8 = ((v >> 7) & 0x7f) as u8;
        let b4: u8 = (v & 0x7f) as u8;
        let _ = self.write(&[b1, b2, b3, b4]);
        self.total = self.total + 4;

        Ok(())
    }

    pub fn string(&mut self, v: &str) -> Result<()> {
        let b = v.as_bytes();
        self.write(&b)
    }

    pub fn utf16_string(&mut self, v: &str) -> Result<()> {
        self.string(v)?;
        self.u8(0)?;
        self.u8(0)
    }

    pub fn non_utf16_string(&mut self, v: &str) -> Result<()> {
        self.string(v)?;
        self.u8(0)
    }

    pub fn write(&mut self, v: &[u8]) -> Result<()> {
        let _ = self.input.write(v);
        self.total = self.total + v.len() as i64;

        Ok(())
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

    pub fn total_write(&mut self) -> i64 {
        self.total
    }
}

impl<T> AsMut<T> for Writable<T> where T: Write + Seek {
    fn as_mut(&mut self) -> &mut T {
        &mut self.input
    }
}

pub trait WritableFactory<T> where T: Write + Seek {
    fn to_writable(self) -> Writable<T>;
}

impl<T: Write + Seek> WritableFactory<T> for T {
    fn to_writable(self) -> Writable<T> {
        Writable::new(self)
    }
}