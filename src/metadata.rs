pub extern crate regex;
extern crate flate2;

use self::flate2::Compression;
use self::flate2::read::ZlibDecoder;
use self::flate2::write::ZlibEncoder;

use errors::*;
use frame::*;
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
            util::id_to_frame_body(frame_header.id.as_str(), 2, frame_readable)?
        };

        Ok(Unit::FrameV2(FrameHeader::V22(frame_header), frame_body))
    }

    pub fn frame3(&mut self,
                  readable: &mut Readable<Cursor<Vec<u8>>>)
                  -> result::Result<Unit, ParsingError> {
        let frame_header = FrameHeaderV3::read(readable)?;

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
            util::id_to_frame_body(frame_header.id.as_str(), 3, frame_readable)?
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
            util::id_to_frame_body(frame_header.id.as_str(), 4, frame_readable)?
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
                  frame_body: FrameBody)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            if let FrameBody::OBJECT(_) = frame_body {
                //
            } else {
                return Err(WriteError::BadInput("Encrypted frame must be FrameBody::OBJECT."
                    .to_string()));
            }
        }

        let (id, bytes) = util::frame_body_as_bytes(&frame_body, 2)?;
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
                  frame_body: FrameBody)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            if let FrameBody::OBJECT(object) = frame_body {
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

        let (id, mut bytes) = util::frame_body_as_bytes(&frame_body, 3)?;
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
                  frame_body: FrameBody)
                  -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            if let FrameBody::OBJECT(object) = frame_body {
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

        let (id, mut bytes) = util::frame_body_as_bytes(&frame_body, 4)?;

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

        let (mut frame_header, frame_body) = frame;

        match frame_header {
            FrameHeader::V22(ref mut frame_header) => {
                let bytes = &self.frame2(frame_header, frame_body)?;
                debug!("write frame2: {}, {}", frame_header.id, bytes.len());
                writable.write(bytes)?;
            }
            FrameHeader::V23(ref mut frame_header) => {
                let bytes = &self.frame3(frame_header, frame_body)?;
                debug!("write frame3: {}, {}", frame_header.id, bytes.len());
                writable.write(bytes)?;
            }
            FrameHeader::V24(ref mut frame_header) => {
                let bytes = &self.frame4(frame_header, frame_body)?;
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
                Unit::FrameV2(frame_header, frame_body) => frames.push((frame_header, frame_body)),
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

    fn check_version(&self, units: &Vec<Unit>) -> result::Result<(), WriteError> {
        let head_version = if let Some(&Unit::Header(ref head)) =
            units.iter().find(|unit| match unit {
                &&Unit::Header(_) => true,
                _ => false,
            }) {
            head.version
        } else {
            4
        };

        let err = Err(WriteError::BadInput("exist different version of 'Unit'".to_string()));

        for unit in units {
            match unit {
                &Unit::FrameV2(FrameHeader::V22(_), _) if head_version != 2 => {
                    return err;
                }
                &Unit::FrameV2(FrameHeader::V23(_), _) if head_version != 3 => {
                    return err;
                }
                &Unit::FrameV2(FrameHeader::V24(_), _) if head_version != 4 => {
                    return err;
                }
                _ => (),
            }
        }

        Ok(())
    }

    /// it rewite all the frames to version 4 and it removes frame version 1
    pub fn fix_units(&self, units: &Vec<Unit>) -> result::Result<(Vec<Unit>), WriteError> {
        let ret = units.iter().fold(Vec::new(), |mut vec, unit| {
            match unit {
                &Unit::Header(ref head) => {
                    let mut new_head = head.clone();
                    new_head.version = 4;
                    new_head.minor_version = 0;
                    vec.push(Unit::Header(new_head));
                },
                &Unit::FrameV2(ref frame_header, ref frame_body) => {
                    match frame_body {
                        &FrameBody::CRM(_) => (),
                        &FrameBody::PIC(_) => (),
                        &FrameBody::EQUA(_) => (),
                        &FrameBody::IPLS(_) => (),
                        &FrameBody::RVAD(_) => (),
                        &FrameBody::TDAT(_) => (),
                        &FrameBody::TIME(_) => (),
                        &FrameBody::TORY(_) => (),
                        &FrameBody::TRDA(_) => (),
                        &FrameBody::TSIZ(_) => (),
                        &FrameBody::TYER(_) => (),
                        _ => {
                            let new_frame_header = match frame_header {
                                &FrameHeader::V22(_) => {
                                    let body_id = util::frame_body_to_id(&frame_body, 2);
                                    FrameHeaderV4 {
                                        id: util::ID_V2_V4.get(body_id).unwrap().to_string(),
                                        size: 0, 
                                        status_flag: 0, 
                                        encoding_flag: 0
                                    }
                                },
                                &FrameHeader::V23(ref header) => {
                                    let mut new_header = FrameHeaderV4 {
                                        id: header.id.to_owned(),
                                        size: 0, 
                                        status_flag: 0, 
                                        encoding_flag: 0
                                    };

                                    if header.has_flag(FrameHeaderFlag::TagAlter) {
                                        new_header.set_flag(FrameHeaderFlag::TagAlter);
                                    } 
                                    if header.has_flag(FrameHeaderFlag::FileAlter) {
                                        new_header.set_flag(FrameHeaderFlag::FileAlter);
                                    }
                                    if header.has_flag(FrameHeaderFlag::ReadOnly) {
                                        new_header.set_flag(FrameHeaderFlag::ReadOnly);
                                    }
                                    if header.has_flag(FrameHeaderFlag::Compression) {
                                        new_header.set_flag(FrameHeaderFlag::Compression);
                                    }
                                    if header.has_flag(FrameHeaderFlag::Encryption) {
                                        new_header.set_flag(FrameHeaderFlag::Encryption);
                                    }
                                    if header.has_flag(FrameHeaderFlag::GroupIdentity) {
                                        new_header.set_flag(FrameHeaderFlag::GroupIdentity);
                                    }

                                    new_header
                                },
                                &FrameHeader::V24(ref header) => header.clone()

                            };
                            vec.push(Unit::FrameV2(FrameHeader::V24(new_frame_header), frame_body.clone()));
                        }

                    }
                },
                _ => (),
            }

            vec
        });


        Ok((ret))
    }

    /// clean_write: it determin if rewrite all to version 4 or not. if it 'true', it rewrite to version 4. 
    /// and in 2.2 'CRM', 'PIC'. in 2.3 'EQUA', 'IPLS', 'RVAD', 'TDAT', 'TIME', 'TORY', 'TRDA', 'TSIZ', 
    /// 'TYER' frames are ignored.
    ///
    /// if it is false, it write with given 'units' parameter.
    /// but it checks version. all of the unit must have to same version.
    ///
    /// if both 'head' are not given, a 'head' will be created with version 4.
    pub fn write(&self, mut units: Vec<Unit>, clean_write: bool) -> result::Result<(), WriteError> {

        if clean_write {
            units = self.fix_units(&units)?;
        } else {
            self.check_version(&units)?;
        }

        let (has_frame1, head_len, all_bytes) = self.to_bytes(units)?;
        let (orig_head_len, file_len) = self.metadata_length()?;

        let mut writable = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.path)?
            .to_writable();

        let head_diff_len = orig_head_len as i32 - head_len as i32;

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