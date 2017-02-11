pub extern crate regex;
extern crate flate2;

use self::flate2::Compression;
use self::flate2::read::ZlibDecoder;
use self::flate2::write::ZlibEncoder;

use errors::*;
use frame::*;
use frame::id::*;
use util;
use readable::{Readable, ReadableFactory};
use writable::{Writable, WritableFactory};

use std::cell::RefCell;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, Cursor, Read, Write};
use std::iter::Iterator;
use std::rc::Rc;
use std::result;
use std::vec::Vec;

type RefHead = Rc<RefCell<Box<Head>>>;
type RefFileReader = Rc<RefCell<Box<Readable<File>>>>;
type RefByteReader = Rc<RefCell<Box<Readable<Cursor<Vec<u8>>>>>>;

#[derive(Debug)]
enum Status {
    Head(RefFileReader),
    ExtendedHeader(RefHead, RefFileReader),
    Frame(RefHead, RefFileReader, RefByteReader),
    None,
}

#[derive(Debug)]
pub enum Unit {
    Header(Head),
    // TODO not yet implemented
    ExtendedHeader(Vec<u8>),
    FrameV1(Frame1),
    FrameV2(FrameHeader, FrameBody),
    Unknown(String),
}

pub struct MetadataReader {
    next: Status,
    file_len: u64,
}

impl MetadataReader {
    pub fn new(path: &str) -> result::Result<Self, ParsingError> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len();
        let readable = file.to_readable();

        Ok(MetadataReader {
            next: Status::Head(Rc::new(RefCell::new(Box::new(readable)))),
            file_len: file_len,
        })
    }

    fn has_frame_id(&mut self, readable: &mut Readable<Cursor<Vec<u8>>>) -> bool {
        match readable.look_string(4) {
            Ok(id) => {
                //
                // http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let regex = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}").unwrap();
                let matched = regex.is_match(&id);
                debug!("Frame Id:'{}', reg matched: {}", id, matched);

                matched
            }
            _ => false,
        }
    }

    fn head(&mut self, readable_wrap: RefFileReader) -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();
        let head = Head::read(&mut readable.to_readable(10)?)?;
        let is_extended = head.has_flag(HeadFlag::ExtendedHeader);
        let head_wrap = Rc::new(RefCell::new(Box::new(head.clone())));

        debug!("{:?}", head);

        self.next = if is_extended {
            Status::ExtendedHeader(head_wrap, readable_wrap.clone())
        } else {
            let head_size = head.size as usize;
            let frame_bytes = if head.has_flag(HeadFlag::Unsynchronisation) {
                let mut bytes = readable.bytes(head_size)?;
                util::to_synchronize(&mut bytes);
                bytes
            } else {
                readable.bytes(head_size)?
            };
            let frame_readable = Cursor::new(frame_bytes).to_readable();
            let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

            Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap)
        };


        Ok(Unit::Header(head))
    }

    // optional unit
    fn extended_head(&mut self,
                     head_wrap: RefHead,
                     readable_wrap: RefFileReader)
                     -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();
        let size = match head_wrap.borrow().version {
            //
            // Did not explained for whether big-endian or synchsafe
            // in "http://id3.org/id3v2.3.0".
            3 => readable.u32()?,
            //
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => readable.synchsafe()?,
        };
        let extended_bytes = readable.bytes(size as usize)?;
        let head_size = head_wrap.borrow().size as usize;
        let frame_bytes = readable.bytes(head_size)?;
        let frame_readable = Cursor::new(frame_bytes).to_readable();
        let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

        self.next = Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap);

        Ok(Unit::ExtendedHeader(extended_bytes))
    }

    fn frame1(&self, readable: &mut Readable<File>) -> result::Result<Frame1, ParsingError> {
        if self.file_len < 128 {
            return Err(ParsingError::BadData(ParsingErrorKind::InvalidFrameLength));
        }

        readable.skip((self.file_len - 128) as i64)?;

        if readable.string(3)? != "TAG" {
            let _ = readable.skip(-3);
            debug!("{}", util::to_hex(&readable.bytes(3)?));
            return Err(ParsingError::BadData(ParsingErrorKind::InvalidV1FrameId));
        }

        Frame1::read(&mut Cursor::new(readable.all_bytes()?).to_readable())
    }

    pub fn frame2(&mut self,
                  readable: &mut Readable<Cursor<Vec<u8>>>)
                  -> result::Result<Unit, ParsingError> {
        let frame_header = FrameHeaderV2::read(readable)?;

        let frame_body = if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            FrameBody::SKIP(frame_header.id.to_owned(),
                            readable.bytes(frame_header.size as usize)?)
        } else {
            let frame_readable = readable.to_readable(frame_header.size as usize)?;
            frame_data(frame_header.id.as_str(), 2, frame_readable)?
        };

        Ok(Unit::FrameV2(FrameHeader::V22(frame_header), frame_body))
    }

    pub fn frame3(&mut self,
                  readable: &mut Readable<Cursor<Vec<u8>>>)
                  -> result::Result<Unit, ParsingError> {
        let frame_header = FrameHeaderV3::read(readable)?;

        println!("frame header {:?}", frame_header);

        let mut extra_size: u32 = 0;
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        let body_bytes = if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("compression");

            let _ = readable.u32()?;
            extra_size = extra_size + 4;

            let actual_size = frame_header.size - extra_size as u32;
            let body_bytes = readable.bytes(actual_size as usize)?;
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&body_bytes[..]);

            let _ = decoder.read_to_end(&mut out);

            out
        } else {
            let actual_size = frame_header.size - extra_size as u32;
            readable.bytes(actual_size as usize)?
        };

        let frame_body = if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            debug!("encryption");

            FrameBody::SKIP(frame_header.id.to_owned(), body_bytes)
        } else {
            let frame_readable = Cursor::new(body_bytes).to_readable();
            frame_data(frame_header.id.as_str(), 3, frame_readable)?
        };

        Ok(Unit::FrameV2(FrameHeader::V23(frame_header), frame_body))
    }

    pub fn frame4(&mut self,
                  readable: &mut Readable<Cursor<Vec<u8>>>)
                  -> result::Result<Unit, ParsingError> {
        let frame_header = FrameHeaderV4::read(readable)?;

        let mut extra_size: u32 = 0;
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::DataLength) {
            let _ = readable.u32()?;
            extra_size = extra_size + 4;
        }

        let actual_size = frame_header.size - extra_size as u32;
        let mut body_bytes = readable.bytes(actual_size as usize)?;

        if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
            debug!("'{}' is unsynchronised", frame_header.id);

            let mut out = body_bytes[..].to_vec();
            let sync_size = util::to_synchronize(&mut out);

            //cut to synchrosized size
            out.split_off(sync_size);

            body_bytes = out;
        }

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("'{}' is compressed", frame_header.id);

            let real_frame = body_bytes.clone();
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&real_frame[..]);
            let _ = decoder.read_to_end(&mut out);
            body_bytes = out;
        }
        let frame_body = if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            FrameBody::SKIP(frame_header.id.to_owned(), body_bytes)
        } else {
            let frame_readable = Cursor::new(body_bytes).to_readable();
            frame_data(frame_header.id.as_str(), 4, frame_readable)?
        };

        Ok(Unit::FrameV2(FrameHeader::V24(frame_header), frame_body))
    }

    fn frame(&mut self,
             head_wrap: RefHead,
             readable_wrap: RefFileReader,
             frame_readable_wrap: RefByteReader)
             -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();
        let mut frame_readable = frame_readable_wrap.borrow_mut();
        //
        // frame v1
        if !self.has_frame_id(&mut frame_readable) {
            self.next = Status::None;
            return Ok(Unit::FrameV1(self.frame1(&mut readable)?));
        }

        //
        // frame v2
        match head_wrap.borrow().version {
            2 => self.frame2(&mut frame_readable),
            3 => self.frame3(&mut frame_readable),
            4 => self.frame4(&mut frame_readable),
            _ => self.frame4(&mut frame_readable),
        }
    }
}

impl Iterator for MetadataReader {
    type Item = Unit;

    fn next(&mut self) -> Option<(Self::Item)> {
        match self.next {
            Status::Head(_) => debug!("next: Head"),
            Status::ExtendedHeader(_, _) => debug!("next: ExtendedHeader"),
            Status::Frame(_, _, _) => debug!("next: Frame"),
            Status::None => debug!("next: None"),
        };

        fn head(next: &Status) -> Option<RefFileReader> {
            match next {
                &Status::Head(ref readable) => Some(readable.clone()),
                _ => None,
            }
        }

        fn extended_head(next: &Status) -> Option<(RefHead, RefFileReader)> {
            match next {
                &Status::ExtendedHeader(ref head, ref readable) => {
                    Some((head.clone(), readable.clone()))
                }
                _ => None,
            }
        }

        fn frame(next: &Status) -> Option<(RefHead, RefFileReader, RefByteReader)> {
            match next {
                &Status::Frame(ref head, ref readable, ref frame_readable) => {
                    Some((head.clone(), readable.clone(), frame_readable.clone()))
                }
                _ => None,
            }
        }

        let head = head(&self.next);
        let extended_header = extended_head(&self.next);
        let frame = frame(&self.next);

        match self.next {
            Status::Head(_) => {
                match self.head(head.unwrap()) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        debug!("Stop on 'Head': {}", msg);
                        None
                    }
                }
            }
            Status::ExtendedHeader(_, _) => {
                let (head, readable) = extended_header.unwrap();
                match self.extended_head(head, readable) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        debug!("Stop on 'Extended Head': {}", msg);
                        None
                    }
                }
            }
            Status::Frame(_, _, _) => {
                let (head, readable, frame_readable) = frame.unwrap();
                match self.frame(head, readable, frame_readable) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        debug!("Ignored 'Frame': {}", msg);
                        Some(Unit::Unknown(msg.description().to_string()))
                    }
                }
            }
            _ => None,
        }
    }
}

pub struct MetadataWriter<'a> {
    path: &'a str,
}

impl<'a> MetadataWriter<'a> {
    pub fn new(path: &'a str) -> result::Result<Self, WriteError> {
        Ok(MetadataWriter { path: path })
    }

    pub fn head(&self, head: Head) -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));
        head.write(&mut writable)?;

        let mut buf = Vec::new();
        let _ = writable.copy(&mut buf);

        Ok(buf)
    }

    pub fn frame1(&self, frame1: Frame1) -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));
        frame1.write(&mut writable)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn frame2(&self,
                  frame_header: &mut FrameHeaderV2,
                  frame_data: FrameBody)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            if let FrameBody::OBJECT(_) = frame_data {
                //
            } else {
                return Err(WriteError::BadInput("Encrypted frame must be FrameBody::OBJECT."
                    .to_string()));
            }
        }

        let (id, bytes) = write_frame_data(&frame_data, 2)?;
        frame_header.id = id.to_string();
        frame_header.size = bytes.len() as u32;
        frame_header.write(&mut writable)?;
        writable.write(&bytes)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn frame3(&self,
                  frame_header: &mut FrameHeaderV3,
                  frame_data: FrameBody)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            if let FrameBody::OBJECT(object) = frame_data {
                frame_header.size = object.data.len() as u32;
                let _ = frame_header.write(&mut writable);
                let _ = writable.write(&object.data);

                let mut buf = Vec::new();
                writable.copy(&mut buf)?;

                return Ok(buf);
            } else {
                return Err(WriteError::BadInput("Encrypted frame must be FrameBody::OBJECT."
                    .to_string()));
            }
        }

        let (id, mut bytes) = write_frame_data(&frame_data, 3)?;
        frame_header.id = id.to_string();
        frame_header.size = if frame_header.has_flag(FrameHeaderFlag::Compression) {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
            let _ = encoder.write(&bytes);
            bytes = encoder.finish()?;
            bytes.len() as u32
        } else {
            bytes.len() as u32
        };

        frame_header.write(&mut writable)?;
        writable.write(&bytes)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn frame4(&self,
                  frame_header: &mut FrameHeaderV4,
                  frame_data: FrameBody)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            if let FrameBody::OBJECT(object) = frame_data {
                frame_header.size = object.data.len() as u32;
                let _ = frame_header.write(&mut writable);
                let _ = writable.write(&object.data);

                let mut buf = Vec::new();
                writable.copy(&mut buf)?;

                return Ok(buf);
            } else {
                return Err(WriteError::BadInput("Encrypted frame must be FrameBody::OBJECT."
                    .to_string()));
            }
        }

        let (id, mut bytes) = write_frame_data(&frame_data, 4)?;

        frame_header.id = id.to_string();
        frame_header.size = bytes.len() as u32;

        if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
            debug!("write {} unsynchronization", id);

            bytes = util::to_unsynchronize(&bytes);
            frame_header.size = bytes.len() as u32
        }

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("write {} compression", id);

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
            let _ = encoder.write(&bytes);
            bytes = encoder.finish()?;
            frame_header.size = bytes.len() as u32
        }

        frame_header.write(&mut writable)?;
        writable.write(&bytes)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn frame(&self, frame: (FrameHeader, FrameBody)) -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        let (mut frame_header, frame_data) = frame;

        match frame_header {
            FrameHeader::V22(ref mut frame_header) => {
                let bytes = &self.frame2(frame_header, frame_data)?;
                debug!("write frame2: {}, {}", frame_header.id, bytes.len());
                writable.write(bytes)?;
            }
            FrameHeader::V23(ref mut frame_header) => {
                let bytes = &self.frame3(frame_header, frame_data)?;
                debug!("write frame3: {}, {}", frame_header.id, bytes.len());
                writable.write(bytes)?;
            }
            FrameHeader::V24(ref mut frame_header) => {
                let bytes = &self.frame4(frame_header, frame_data)?;
                debug!("write frame4: {}, {}", frame_header.id, bytes.len());
                writable.write(bytes)?;
            }
        }

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn frames(&self,
                  frames: Vec<(FrameHeader, FrameBody)>)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));
        for frame in frames {
            let _ = writable.write(&self.frame(frame)?);
        }

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn to_bytes(&self, units: Vec<Unit>) -> result::Result<(bool, u32, Vec<u8>), WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        let mut head_wrap = None;
        let mut frame1_wrap = None;
        let mut frames = Vec::new();

        for unit in units {
            match unit {
                Unit::Header(head) => head_wrap = Some(head),
                Unit::FrameV1(frame) => frame1_wrap = Some(frame),
                Unit::FrameV2(frame_header, frame_data) => frames.push((frame_header, frame_data)),
                _ => (),
            }
        }

        let mut head = if head_wrap.is_none() {
            Head {
                version: 4,
                minor_version: 0,
                flag: 0,
                size: 0,
            }
        } else {
            head_wrap.unwrap()
        };

        let mut frame_bytes = self.frames(frames)?;

        if head.has_flag(HeadFlag::Unsynchronisation) {
            debug!("head unsynchronisation");

            frame_bytes = util::to_unsynchronize(&frame_bytes);
        };

        head.size = frame_bytes.len() as u32;

        let head_size = head.size;

        writable.write(&self.head(head)?)?;
        writable.write(&frame_bytes)?;

        let has_frame1 = if let Some(frame1) = frame1_wrap {
            writable.write(&self.frame1(frame1)?)?;
            true
        } else {
            false
        };

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok((has_frame1, head_size, buf))
    }

    pub fn write(&self, units: Vec<Unit>) -> result::Result<(), WriteError> {
        let (has_frame1, head_len, all_bytes) = self.to_bytes(units)?;
        let (orig_head_len, file_len) = self.metadata_length()?;

        let mut writable = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.path)?
            .to_writable();

        let head_diff_len = orig_head_len as i32 - head_len as i32;

        println!("has_frame1: {}", has_frame1);
        println!("head_len: {}", head_len);
        println!("orig_head_len: {}", orig_head_len);
        println!("file_len: {}", file_len);
        println!("head_diff_len: {}", head_diff_len);

        if head_diff_len > 0 && file_len > head_diff_len as u64 {
            writable.unshift(head_diff_len as usize)?;

            let len = file_len - head_diff_len as u64;
            OpenOptions::new().write(true).open(self.path)?.set_len(len)?;
        } else if head_diff_len < 0 && file_len > head_diff_len.abs() as u64 {
            writable.shift(head_diff_len.abs() as usize)?;
        }

        let (head_bytes, frames) = all_bytes.split_at(head_len as usize);

        let (frame_bytes, frame1_bytes) = if has_frame1 {
            frames.split_at(frames.len() - 128)
        } else {
            frames.split_at(frames.len())
        };

        writable.write(&head_bytes)?;
        writable.write(&frame_bytes)?;
        writable.write(&frame1_bytes)?;

        Ok(())
    }

    fn metadata_length(&self) -> io::Result<(u32, u64)> {
        match self::MetadataReader::new(self.path) {
            Ok(meta_reader) => {
                let mut i = meta_reader.filter(|m| match m {
                    &Unit::Header(_) => true,
                    _ => false,
                });

                let header_length = if let Some(Unit::Header(head)) = i.next() {
                    head.size
                } else {
                    0
                };

                let file_len = File::open(self.path)?.metadata()?.len();

                Ok((header_length, file_len))
            }
            _ => Ok((0, 0)),
        }
    }
}

fn frame_data(id: &str,
              version: u8,
              mut readable: Readable<Cursor<Vec<u8>>>)
              -> result::Result<FrameBody, ParsingError> {
    let frame_data = match id.as_ref() {
        BUF_STR => FrameBody::BUF(BUF::read(&mut readable)?),
        CNT_STR => FrameBody::PCNT(PCNT::read(&mut readable)?),
        COM_STR => FrameBody::COMM(COMM::read(&mut readable)?),
        CRA_STR => FrameBody::AENC(AENC::read(&mut readable)?),
        CRM_STR => FrameBody::CRM(CRM::read(&mut readable)?),
        ETC_STR => FrameBody::ETCO(ETCO::read(&mut readable)?),
        EQU_STR => FrameBody::EQUA(EQUA::read(&mut readable)?),
        GEO_STR => FrameBody::GEOB(GEOB::read(&mut readable)?),
        IPL_STR => FrameBody::IPLS(IPLS::read(&mut readable)?),
        LNK_STR => FrameBody::LINK(LINK::read(&mut readable, version)?),
        MCI_STR => FrameBody::MCDI(MCDI::read(&mut readable)?),
        MLL_STR => FrameBody::MLLT(MLLT::read(&mut readable)?),
        PIC_STR => FrameBody::PIC(PIC::read(&mut readable)?),
        POP_STR => FrameBody::POPM(POPM::read(&mut readable)?),
        REV_STR => FrameBody::RVRB(RVRB::read(&mut readable)?),
        RVA_STR => FrameBody::RVAD(RVA2::read(&mut readable)?),
        SLT_STR => FrameBody::SYLT(SYLT::read(&mut readable)?),
        STC_STR => FrameBody::SYTC(SYTC::read(&mut readable)?),
        TAL_STR => FrameBody::TALB(TEXT::read(&mut readable, id)?),
        TBP_STR => FrameBody::TBPM(TEXT::read(&mut readable, id)?),
        TCM_STR => FrameBody::TCOM(TEXT::read(&mut readable, id)?),
        TCO_STR => FrameBody::TCON(TEXT::read(&mut readable, id)?),
        TCR_STR => FrameBody::TCOP(TEXT::read(&mut readable, id)?),
        TDA_STR => FrameBody::TDAT(TEXT::read(&mut readable, id)?),
        TDY_STR => FrameBody::TDLY(TEXT::read(&mut readable, id)?),
        TEN_STR => FrameBody::TENC(TEXT::read(&mut readable, id)?),
        TFT_STR => FrameBody::TFLT(TEXT::read(&mut readable, id)?),
        TIM_STR => FrameBody::TIME(TEXT::read(&mut readable, id)?),
        TKE_STR => FrameBody::TKEY(TEXT::read(&mut readable, id)?),
        TLA_STR => FrameBody::TLAN(TEXT::read(&mut readable, id)?),
        TLE_STR => FrameBody::TLEN(TEXT::read(&mut readable, id)?),
        TMT_STR => FrameBody::TMED(TEXT::read(&mut readable, id)?),
        TOA_STR => FrameBody::TMED(TEXT::read(&mut readable, id)?),
        TOF_STR => FrameBody::TOFN(TEXT::read(&mut readable, id)?),
        TOL_STR => FrameBody::TOLY(TEXT::read(&mut readable, id)?),
        TOR_STR => FrameBody::TORY(TEXT::read(&mut readable, id)?),
        TOT_STR => FrameBody::TOAL(TEXT::read(&mut readable, id)?),
        TP1_STR => FrameBody::TPE1(TEXT::read(&mut readable, id)?),
        TP2_STR => FrameBody::TPE2(TEXT::read(&mut readable, id)?),
        TP3_STR => FrameBody::TPE3(TEXT::read(&mut readable, id)?),
        TP4_STR => FrameBody::TPE4(TEXT::read(&mut readable, id)?),
        TPA_STR => FrameBody::TPOS(TEXT::read(&mut readable, id)?),
        TPB_STR => FrameBody::TPUB(TEXT::read(&mut readable, id)?),
        TRC_STR => FrameBody::TSRC(TEXT::read(&mut readable, id)?),
        TRD_STR => FrameBody::TRDA(TEXT::read(&mut readable, id)?),
        TRK_STR => FrameBody::TRCK(TEXT::read(&mut readable, id)?),
        TSI_STR => FrameBody::TSIZ(TEXT::read(&mut readable, id)?),
        TSS_STR => FrameBody::TSSE(TEXT::read(&mut readable, id)?),
        TT1_STR => FrameBody::TIT1(TEXT::read(&mut readable, id)?),
        TT2_STR => FrameBody::TIT2(TEXT::read(&mut readable, id)?),
        TT3_STR => FrameBody::TIT3(TEXT::read(&mut readable, id)?),
        TXT_STR => FrameBody::TEXT(TEXT::read(&mut readable, id)?),
        TYE_STR => FrameBody::TYER(TEXT::read(&mut readable, id)?),
        TXX_STR => FrameBody::TXXX(TXXX::read(&mut readable)?),
        UFI_STR => FrameBody::UFID(UFID::read(&mut readable)?),
        ULT_STR => FrameBody::USLT(USLT::read(&mut readable)?),
        WAF_STR => FrameBody::WOAF(LINK::read(&mut readable, version)?),
        WAR_STR => FrameBody::WOAR(LINK::read(&mut readable, version)?),
        WAS_STR => FrameBody::WOAS(LINK::read(&mut readable, version)?),
        WCM_STR => FrameBody::WCOM(LINK::read(&mut readable, version)?),
        WCP_STR => FrameBody::WCOP(LINK::read(&mut readable, version)?),
        WPB_STR => FrameBody::WPUB(LINK::read(&mut readable, version)?),
        WXX_STR => FrameBody::WXXX(WXXX::read(&mut readable)?),
        AENC_STR => FrameBody::AENC(AENC::read(&mut readable)?),
        APIC_STR => FrameBody::APIC(APIC::read(&mut readable)?),
        ASPI_STR => FrameBody::ASPI(ASPI::read(&mut readable)?),
        COMM_STR => FrameBody::COMM(COMM::read(&mut readable)?),
        COMR_STR => FrameBody::COMR(COMR::read(&mut readable)?),
        ENCR_STR => FrameBody::ENCR(ENCR::read(&mut readable)?),
        EQUA_STR => FrameBody::EQUA(EQUA::read(&mut readable)?),
        EQU2_STR => FrameBody::EQU2(EQU2::read(&mut readable)?),
        ETCO_STR => FrameBody::ETCO(ETCO::read(&mut readable)?),
        GEOB_STR => FrameBody::GEOB(GEOB::read(&mut readable)?),
        GRID_STR => FrameBody::GRID(GRID::read(&mut readable)?),
        IPLS_STR => FrameBody::IPLS(IPLS::read(&mut readable)?),
        LINK_STR => FrameBody::LINK(LINK::read(&mut readable, version)?),
        MCDI_STR => FrameBody::MCDI(MCDI::read(&mut readable)?),
        MLLT_STR => FrameBody::MLLT(MLLT::read(&mut readable)?),
        OWNE_STR => FrameBody::OWNE(OWNE::read(&mut readable)?),
        PRIV_STR => FrameBody::PRIV(PRIV::read(&mut readable)?),
        PCNT_STR => FrameBody::PCNT(PCNT::read(&mut readable)?),
        POPM_STR => FrameBody::POPM(POPM::read(&mut readable)?),
        POSS_STR => FrameBody::POSS(POSS::read(&mut readable)?),
        RBUF_STR => FrameBody::RBUF(RBUF::read(&mut readable)?),
        RVAD_STR => FrameBody::RVAD(RVA2::read(&mut readable)?),
        RVA2_STR => FrameBody::RVA2(RVA2::read(&mut readable)?),
        RVRB_STR => FrameBody::RVRB(RVRB::read(&mut readable)?),
        SEEK_STR => FrameBody::SEEK(SEEK::read(&mut readable)?),
        SIGN_STR => FrameBody::SIGN(SIGN::read(&mut readable)?),
        SYLT_STR => FrameBody::SYLT(SYLT::read(&mut readable)?),
        SYTC_STR => FrameBody::SYTC(SYTC::read(&mut readable)?),
        UFID_STR => FrameBody::UFID(UFID::read(&mut readable)?),
        USER_STR => FrameBody::USER(USER::read(&mut readable)?),
        USLT_STR => FrameBody::USLT(USLT::read(&mut readable)?),
        TALB_STR => FrameBody::TALB(TEXT::read(&mut readable, id)?),
        TBPM_STR => FrameBody::TBPM(TEXT::read(&mut readable, id)?),
        TCOM_STR => FrameBody::TCOM(TEXT::read(&mut readable, id)?),
        TCON_STR => FrameBody::TCON(TEXT::read(&mut readable, id)?),
        TCOP_STR => FrameBody::TCOP(TEXT::read(&mut readable, id)?),
        TDAT_STR => FrameBody::TDAT(TEXT::read(&mut readable, id)?),
        TDEN_STR => FrameBody::TDEN(TEXT::read(&mut readable, id)?),
        TDLY_STR => FrameBody::TDLY(TEXT::read(&mut readable, id)?),
        TDOR_STR => FrameBody::TDOR(TEXT::read(&mut readable, id)?),
        TDRC_STR => FrameBody::TDRC(TEXT::read(&mut readable, id)?),
        TDRL_STR => FrameBody::TDRL(TEXT::read(&mut readable, id)?),
        TDTG_STR => FrameBody::TDTG(TEXT::read(&mut readable, id)?),
        TENC_STR => FrameBody::TENC(TEXT::read(&mut readable, id)?),
        TEXT_STR => FrameBody::TEXT(TEXT::read(&mut readable, id)?),
        TIME_STR => FrameBody::TIME(TEXT::read(&mut readable, id)?),
        TFLT_STR => FrameBody::TFLT(TEXT::read(&mut readable, id)?),
        TIPL_STR => FrameBody::TIPL(TEXT::read(&mut readable, id)?),
        TIT1_STR => FrameBody::TIT1(TEXT::read(&mut readable, id)?),
        TIT2_STR => FrameBody::TIT2(TEXT::read(&mut readable, id)?),
        TIT3_STR => FrameBody::TIT3(TEXT::read(&mut readable, id)?),
        TKEY_STR => FrameBody::TKEY(TEXT::read(&mut readable, id)?),
        TLAN_STR => FrameBody::TLAN(TEXT::read(&mut readable, id)?),
        TLEN_STR => FrameBody::TLEN(TEXT::read(&mut readable, id)?),
        TMCL_STR => FrameBody::TMCL(TEXT::read(&mut readable, id)?),
        TMED_STR => FrameBody::TMED(TEXT::read(&mut readable, id)?),
        TMOO_STR => FrameBody::TMOO(TEXT::read(&mut readable, id)?),
        TOAL_STR => FrameBody::TOAL(TEXT::read(&mut readable, id)?),
        TOFN_STR => FrameBody::TOFN(TEXT::read(&mut readable, id)?),
        TOLY_STR => FrameBody::TOLY(TEXT::read(&mut readable, id)?),
        TOPE_STR => FrameBody::TOPE(TEXT::read(&mut readable, id)?),
        TORY_STR => FrameBody::TORY(TEXT::read(&mut readable, id)?),
        TOWN_STR => FrameBody::TOWN(TEXT::read(&mut readable, id)?),
        TPE1_STR => FrameBody::TPE1(TEXT::read(&mut readable, id)?),
        TPE2_STR => FrameBody::TPE2(TEXT::read(&mut readable, id)?),
        TPE3_STR => FrameBody::TPE3(TEXT::read(&mut readable, id)?),
        TPE4_STR => FrameBody::TPE4(TEXT::read(&mut readable, id)?),
        TPOS_STR => FrameBody::TPOS(TEXT::read(&mut readable, id)?),
        TPRO_STR => FrameBody::TPRO(TEXT::read(&mut readable, id)?),
        TPUB_STR => FrameBody::TPUB(TEXT::read(&mut readable, id)?),
        TRCK_STR => FrameBody::TRCK(TEXT::read(&mut readable, id)?),
        TRDA_STR => FrameBody::TRDA(TEXT::read(&mut readable, id)?),
        TRSN_STR => FrameBody::TRSN(TEXT::read(&mut readable, id)?),
        TSIZ_STR => FrameBody::TSIZ(TEXT::read(&mut readable, id)?),
        TRSO_STR => FrameBody::TRSO(TEXT::read(&mut readable, id)?),
        TSOA_STR => FrameBody::TSOA(TEXT::read(&mut readable, id)?),
        TSOP_STR => FrameBody::TSOP(TEXT::read(&mut readable, id)?),
        TSOT_STR => FrameBody::TSOT(TEXT::read(&mut readable, id)?),
        TSRC_STR => FrameBody::TSRC(TEXT::read(&mut readable, id)?),
        TSSE_STR => FrameBody::TSSE(TEXT::read(&mut readable, id)?),
        TYER_STR => FrameBody::TYER(TEXT::read(&mut readable, id)?),
        TSST_STR => FrameBody::TSST(TEXT::read(&mut readable, id)?),
        TXXX_STR => FrameBody::TXXX(TXXX::read(&mut readable)?),
        WCOM_STR => FrameBody::WCOM(LINK::read(&mut readable, version)?),
        WCOP_STR => FrameBody::WCOP(LINK::read(&mut readable, version)?),
        WOAF_STR => FrameBody::WOAF(LINK::read(&mut readable, version)?),
        WOAR_STR => FrameBody::WOAR(LINK::read(&mut readable, version)?),
        WOAS_STR => FrameBody::WOAS(LINK::read(&mut readable, version)?),
        WORS_STR => FrameBody::WORS(LINK::read(&mut readable, version)?),
        WPAY_STR => FrameBody::WPAY(LINK::read(&mut readable, version)?),
        WPUB_STR => FrameBody::WPUB(LINK::read(&mut readable, version)?),
        WXXX_STR => FrameBody::WXXX(WXXX::read(&mut readable)?),
        _ => {
            warn!("No frame id found!! '{}'", id);
            FrameBody::TEXT(TEXT::read(&mut readable, id)?)
        }
    };

    debug!("total read: {}", readable.total_read());

    Ok(frame_data)
}

fn write_frame_data(frame_data: &FrameBody,
                    version: u8)
                    -> result::Result<(&str, Vec<u8>), WriteError> {
    let mut writable = Cursor::new(vec![0u8; 0]).to_writable();

    let id = match frame_data {
        &FrameBody::BUF(ref frame) => {
            frame.write(&mut writable)?;
            id::BUF_STR
        }
        &FrameBody::CRM(ref frame) => {
            frame.write(&mut writable)?;
            id::CRM_STR
        }
        &FrameBody::PIC(ref frame) => {
            frame.write(&mut writable)?;
            id::PIC_STR
        }
        &FrameBody::AENC(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::CRA_STR
            } else {
                id::AENC_STR
            }
        }
        &FrameBody::APIC(ref frame) => {
            frame.write(&mut writable)?;
            id::APIC_STR
        }
        &FrameBody::ASPI(ref frame) => {
            frame.write(&mut writable)?;
            id::ASPI_STR
        }
        &FrameBody::COMM(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::COM_STR
            } else {
                id::COMM_STR
            }
        }
        &FrameBody::COMR(ref frame) => {
            frame.write(&mut writable)?;
            id::COMR_STR
        }
        &FrameBody::ENCR(ref frame) => {
            frame.write(&mut writable)?;
            id::ENCR_STR
        }
        &FrameBody::EQUA(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::EQU_STR
            } else {
                id::EQUA_STR
            }
        }
        &FrameBody::EQU2(ref frame) => {
            frame.write(&mut writable)?;
            id::EQU2_STR
        }
        &FrameBody::ETCO(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::ETC_STR
            } else {
                id::ETCO_STR
            }
        }
        &FrameBody::GEOB(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::GEO_STR
            } else {
                id::GEOB_STR
            }
        }
        &FrameBody::GRID(ref frame) => {
            frame.write(&mut writable)?;
            id::GRID_STR
        }
        &FrameBody::IPLS(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::IPL_STR
            } else {
                id::IPLS_STR
            }
        }
        &FrameBody::LINK(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::LNK_STR
            } else {
                id::LINK_STR
            }
        }
        &FrameBody::MCDI(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::MCI_STR
            } else {
                id::MCDI_STR
            }
        }
        &FrameBody::MLLT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::MLL_STR
            } else {
                id::MLLT_STR
            }
        }
        &FrameBody::OWNE(ref frame) => {
            frame.write(&mut writable)?;
            id::OWNE_STR
        }
        &FrameBody::PRIV(ref frame) => {
            frame.write(&mut writable)?;
            id::PRIV_STR
        }
        &FrameBody::PCNT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::CNT_STR
            } else {
                id::PCNT_STR
            }
        }
        &FrameBody::POPM(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::POP_STR
            } else {
                id::POPM_STR
            }
        }
        &FrameBody::POSS(ref frame) => {
            frame.write(&mut writable)?;
            id::POSS_STR
        }
        &FrameBody::RBUF(ref frame) => {
            frame.write(&mut writable)?;
            id::RBUF_STR
        }
        &FrameBody::RVAD(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::RVA_STR
            } else {
                id::RVAD_STR
            }
        }
        &FrameBody::RVA2(ref frame) => {
            frame.write(&mut writable)?;
            id::RVA2_STR
        }
        &FrameBody::RVRB(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::REV_STR
            } else {
                id::RVRB_STR
            }
        }
        &FrameBody::SEEK(ref frame) => {
            frame.write(&mut writable)?;
            id::SEEK_STR
        }
        &FrameBody::SIGN(ref frame) => {
            frame.write(&mut writable)?;
            id::SIGN_STR
        }
        &FrameBody::SYLT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::SLT_STR
            } else {
                id::SYLT_STR
            }
        }
        &FrameBody::SYTC(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::STC_STR
            } else {
                id::SYTC_STR
            }
        }
        &FrameBody::TALB(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TAL_STR
            } else {
                id::TALB_STR
            }
        }
        &FrameBody::TBPM(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TBP_STR
            } else {
                id::TBPM_STR
            }
        }
        &FrameBody::TCOM(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TCM_STR
            } else {
                id::TCOM_STR
            }
        }
        &FrameBody::TCON(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TCO_STR
            } else {
                id::TCON_STR
            }
        }
        &FrameBody::TCOP(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TCR_STR
            } else {
                id::TCOP_STR
            }
        }
        &FrameBody::TDAT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TDA_STR
            } else {
                id::TDAT_STR
            }
        }
        &FrameBody::TDEN(ref frame) => {
            frame.write(&mut writable)?;
            id::TDEN_STR
        }
        &FrameBody::TDLY(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TDY_STR
            } else {
                id::TDLY_STR
            }
        }
        &FrameBody::TDOR(ref frame) => {
            frame.write(&mut writable)?;
            id::TDOR_STR
        }
        &FrameBody::TDRC(ref frame) => {
            frame.write(&mut writable)?;
            id::TDRC_STR
        }
        &FrameBody::TDRL(ref frame) => {
            frame.write(&mut writable)?;
            id::TDRL_STR
        }
        &FrameBody::TDTG(ref frame) => {
            frame.write(&mut writable)?;
            id::TDTG_STR
        }
        &FrameBody::TENC(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TEN_STR
            } else {
                id::TENC_STR
            }
        }
        &FrameBody::TEXT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TXT_STR
            } else {
                id::TEXT_STR
            }
        }
        &FrameBody::TFLT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TFT_STR
            } else {
                id::TFLT_STR
            }
        }
        &FrameBody::TIME(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TIM_STR
            } else {
                id::TIME_STR
            }
        }
        &FrameBody::TIPL(ref frame) => {
            frame.write(&mut writable)?;
            id::TIPL_STR
        }
        &FrameBody::TIT1(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TT1_STR
            } else {
                id::TIT1_STR
            }
        }
        &FrameBody::TIT2(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TT2_STR
            } else {
                id::TIT2_STR
            }
        }
        &FrameBody::TIT3(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TT3_STR
            } else {
                id::TIT3_STR
            }
        }
        &FrameBody::TKEY(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TKE_STR
            } else {
                id::TKEY_STR
            }
        }
        &FrameBody::TLAN(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TLA_STR
            } else {
                id::TLAN_STR
            }
        }
        &FrameBody::TLEN(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TLE_STR
            } else {
                id::TLEN_STR
            }
        }
        &FrameBody::TMCL(ref frame) => {
            frame.write(&mut writable)?;
            id::TMCL_STR
        }
        &FrameBody::TMED(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TMT_STR
            } else {
                id::TMED_STR
            }
        }
        &FrameBody::TMOO(ref frame) => {
            frame.write(&mut writable)?;
            id::TMOO_STR
        }
        &FrameBody::TOAL(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TOT_STR
            } else {
                id::TOAL_STR
            }
        }
        &FrameBody::TOFN(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TOF_STR
            } else {
                id::TOFN_STR
            }
        }
        &FrameBody::TOLY(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TOL_STR
            } else {
                id::TOLY_STR
            }
        }
        &FrameBody::TOPE(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TOA_STR
            } else {
                id::TOPE_STR
            }
        }
        &FrameBody::TORY(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TOR_STR
            } else {
                id::TORY_STR
            }
        }
        &FrameBody::TOWN(ref frame) => {
            frame.write(&mut writable)?;
            id::TOWN_STR
        }
        &FrameBody::TPE1(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TP1_STR
            } else {
                id::TPE1_STR
            }
        }
        &FrameBody::TPE2(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TP2_STR
            } else {
                id::TPE2_STR
            }
        }
        &FrameBody::TPE3(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TP3_STR
            } else {
                id::TPE3_STR
            }
        }
        &FrameBody::TPE4(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TP4_STR
            } else {
                id::TPE4_STR
            }
        }
        &FrameBody::TPOS(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TPA_STR
            } else {
                id::TPOS_STR
            }
        }
        &FrameBody::TPRO(ref frame) => {
            frame.write(&mut writable)?;
            id::TPRO_STR
        }
        &FrameBody::TPUB(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TPB_STR
            } else {
                id::TPUB_STR
            }
        }
        &FrameBody::TRCK(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TRK_STR
            } else {
                id::TRCK_STR
            }
        }
        &FrameBody::TRDA(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TRD_STR
            } else {
                id::TRDA_STR
            }
        }
        &FrameBody::TRSN(ref frame) => {
            frame.write(&mut writable)?;
            id::TRSN_STR
        }
        &FrameBody::TRSO(ref frame) => {
            frame.write(&mut writable)?;
            id::TRSO_STR
        }
        &FrameBody::TSIZ(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TSI_STR
            } else {
                id::TSIZ_STR
            }
        }
        &FrameBody::TSOA(ref frame) => {
            frame.write(&mut writable)?;
            id::TSOA_STR
        }
        &FrameBody::TSOP(ref frame) => {
            frame.write(&mut writable)?;
            id::TSOP_STR
        }
        &FrameBody::TSOT(ref frame) => {
            frame.write(&mut writable)?;
            id::TSOT_STR
        }
        &FrameBody::TSRC(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TRC_STR
            } else {
                id::TSRC_STR
            }
        }
        &FrameBody::TSSE(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TSS_STR
            } else {
                id::TSSE_STR
            }
        }
        &FrameBody::TYER(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TYE_STR
            } else {
                id::TYER_STR
            }
        }
        &FrameBody::TSST(ref frame) => {
            frame.write(&mut writable)?;
            id::TSST_STR
        }
        &FrameBody::TXXX(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::TXX_STR
            } else {
                id::TXXX_STR
            }
        }
        &FrameBody::UFID(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::UFI_STR
            } else {
                id::UFID_STR
            }
        }
        &FrameBody::USER(ref frame) => {
            frame.write(&mut writable)?;
            id::USER_STR
        }
        &FrameBody::USLT(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::ULT_STR
            } else {
                id::USLT_STR
            }
        }
        &FrameBody::WCOM(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::WCM_STR
            } else {
                id::WCOM_STR
            }
        }
        &FrameBody::WCOP(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::WCP_STR
            } else {
                id::WCOP_STR
            }
        }
        &FrameBody::WOAF(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::WAF_STR
            } else {
                id::WOAF_STR
            }
        }
        &FrameBody::WOAR(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::WAR_STR
            } else {
                id::WOAR_STR
            }
        }
        &FrameBody::WOAS(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::WAS_STR
            } else {
                id::WOAS_STR
            }
        }
        &FrameBody::WORS(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WORS_STR
        }
        &FrameBody::WPAY(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WPAY_STR
        }
        &FrameBody::WPUB(ref frame) => {
            frame.write(&mut writable, version)?;
            if version == 2 {
                id::WPB_STR
            } else {
                id::WPUB_STR
            }
        }
        &FrameBody::WXXX(ref frame) => {
            frame.write(&mut writable)?;
            if version == 2 {
                id::WXX_STR
            } else {
                id::WXXX_STR
            }
        }
        _ => "",
    };

    let mut buf = Vec::new();
    writable.copy(&mut buf)?;

    Ok((id, buf))
}