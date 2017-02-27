pub extern crate regex;
extern crate flate2;

use self::flate2::Compression;
use self::flate2::read::ZlibDecoder;
use self::flate2::write::ZlibEncoder;

use frame::*;
use frame::types::*;
use rw::{Readable, Writable};

use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Error, ErrorKind, Result, Read, Write};
use std::iter::Iterator;
use std::rc::Rc;
use std::vec::Vec;

type FrameReadable = Rc<RefCell<Box<Cursor<Vec<u8>>>>>;

///
/// Parsing results.
///
#[derive(Debug, Serialize, Deserialize)]
pub enum Unit {
    Header(Head),
    // TODO not yet implemented
    ExtendedHeader(Vec<u8>),
    FrameV2(FrameHeader, FrameBody),
    FrameV1(Frame1),
}

//
// Internal parsing state.
//
#[derive(Debug)]
enum Status {
    ExtendedHeader(Head),
    Frame(Head, FrameReadable),
    Frame1,
}

///
/// Read operation of Mp3 file.
///
pub trait ReadOp: Readable {
    ///
    /// it read header that is 10 byte length.
    ///
    fn head(&mut self) -> Result<Unit> {
        Ok(Unit::Header(Head::read(&mut self.to_readable(10)?, 0, "")?))
    }

    ///
    /// If tag version is 4, the size of extened header is calcurated as synchsize.
    ///
    fn ext_head(&mut self, head: &Head) -> Result<Unit> {
        let size = match head.version {
            3 => self.read_u32()?,
            _ => self.read_synchsafe()?,
        };

        Ok(Unit::ExtendedHeader(self.read_bytes(size as usize)?))
    }

    ///
    /// It return a Readable that have all the frame bytes.
    /// if flag of header unsynchronized, it recompute to synchronized byte.
    ///
    fn frame_bytes(&mut self, head: &Head) -> Result<Cursor<Vec<u8>>> {
        if head.has_flag(HeadFlag::Unsynchronisation) {
            Ok(Cursor::new(self.to_synchronize(head.size as usize)?))
        } else {
            self.to_readable(head.size as usize)
        }
    }

    ///
    /// read a version 2.x
    ///
    fn frame(&mut self, head: &Head, readable_wrap: FrameReadable) -> Result<Unit> {
        let mut readable = readable_wrap.borrow_mut();

        match head.version {
            2 => self.frame2(&mut readable),
            3 => self.frame3(&mut readable),
            _ => self.frame4(&mut readable),
        }
    }

    ///
    /// read a version 1
    ///
    fn frame1(&mut self, file_len: usize) -> Result<Unit> {

        //
        // Version 1 is 128 byte length totally.
        //
        if file_len < 128 {
            let err_msg = "Invalid frame1 length";
            warn!("{}", err_msg);
            return Err(Error::new(ErrorKind::Other, err_msg));
        }

        //
        // It is located in last of a file.
        //
        self.skip_bytes((file_len - 128) as isize)?;

        //
        // The name of tag id is "TAG".
        //
        if self.read_string(3)? != "TAG" {
            let _ = self.skip_bytes(-3);

            let err_msg = "Invalid frame1 id";
            warn!("{}", err_msg);
            return Err(Error::new(ErrorKind::Other, err_msg));
        }

        Ok(Unit::FrameV1(Frame1::read(&mut self.to_readable(125)?)?))
    }

    ///
    /// read a version 2.2
    ///
    fn frame2(&mut self, readable: &mut Cursor<Vec<u8>>) -> Result<Unit> {
        let frame_header = FrameHeaderV2::read(readable, 2, "")?;
        let size = frame_header.size as usize;

        let frame_body = match frame_header.has_flag(FrameHeaderFlag::Encryption) {
            true => FrameBody::SKIP(frame_header.id.to_owned(), readable.read_bytes(size)?),
            false => {
                read_framebody_with_id(frame_header.id.as_str(), 2, readable.to_readable(size)?)?
            }
        };

        Ok(Unit::FrameV2(FrameHeader::V22(frame_header), frame_body))
    }

    ///
    /// read a version 2.3
    ///
    fn frame3(&mut self, readable: &mut Cursor<Vec<u8>>) -> Result<Unit> {

        let frame_header = FrameHeaderV3::read(readable, 3, "")?;

        let mut extra_size: u32 = 0;

        //
        // If the flag of group-identity is set, one byte follow the frame size.
        //
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            let _ = readable.read_u8()?;
            extra_size = extra_size + 1;
        }

        //
        // If the flag of encyrption is set, one byte follow the frame size.
        //
        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            let _ = readable.read_u8()?;
            extra_size = extra_size + 1;
        }

        //
        // If the flag of compression is set, four byte follow the frame size.
        //
        let body_bytes = match frame_header.has_flag(FrameHeaderFlag::Compression) {
            true => {
                debug!("compression");

                let _ = readable.read_u32()?;
                extra_size = extra_size + 4;

                let actual_size = frame_header.size - extra_size as u32;
                let body_bytes = readable.read_bytes(actual_size as usize)?;

                //
                // The compression frame is compressed using zlip.
                //
                let mut decoder = ZlibDecoder::new(&body_bytes[..]);

                let mut out = vec![];
                let _ = decoder.read_to_end(&mut out);

                out
            }
            false => {
                let actual_size = frame_header.size - extra_size as u32;
                readable.read_bytes(actual_size as usize)?
            }
        };


        //
        // If frame is encrypted, this frame can not read.
        //
        let frame_body = match frame_header.has_flag(FrameHeaderFlag::Encryption) {
            true => {
                debug!("encryption");
                FrameBody::SKIP(frame_header.id.to_owned(), body_bytes)
            }
            false => read_framebody_with_id(frame_header.id.as_str(), 3, Cursor::new(body_bytes))?,
        };

        Ok(Unit::FrameV2(FrameHeader::V23(frame_header), frame_body))
    }

    ///
    /// read a version 2.4
    ///
    fn frame4(&mut self, readable: &mut Cursor<Vec<u8>>) -> Result<Unit> {
        let frame_header = FrameHeaderV4::read(readable, 4, "")?;

        let mut extra_size: u32 = 0;

        //
        // If the flag of group-identity is set, one byte follow the frame size.
        //
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            let _ = readable.read_u8()?;
            extra_size = extra_size + 1;
        }

        //
        // If the flag of encyrption is set, one byte follow the frame size.
        //
        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            let _ = readable.read_u8()?;
            extra_size = extra_size + 1;
        }

        //
        // If the flag of data-length is set, one byte follow the frame size.
        //
        if frame_header.has_flag(FrameHeaderFlag::DataLength) {
            let _ = readable.read_u32()?;
            extra_size = extra_size + 4;
        }

        let actual_size = frame_header.size - extra_size as u32;
        let mut body_bytes = readable.read_bytes(actual_size as usize)?;

        //
        // If frame is unsynchronized, it re-build to synchronized byte.
        //
        if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
            debug!("'{}' is unsynchronised", frame_header.id);
            let bytes = body_bytes[..].to_vec();
            body_bytes = Cursor::new(bytes).to_synchronize(body_bytes.len())?;
        }

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("'{}' is compressed", frame_header.id);

            let real_frame = body_bytes.clone();
            let mut out = vec![];
            //
            // The compression frame is compressed using zlip.
            //
            let mut decoder = ZlibDecoder::new(&real_frame[..]);
            let _ = decoder.read_to_end(&mut out);
            body_bytes = out;
        }

        //
        // If frame is encrypted, this frame can not read.
        //
        let frame_body = match frame_header.has_flag(FrameHeaderFlag::Encryption) {
            true => FrameBody::SKIP(frame_header.id.to_owned(), body_bytes),
            false => read_framebody_with_id(frame_header.id.as_str(), 4, Cursor::new(body_bytes))?,
        };

        Ok(Unit::FrameV2(FrameHeader::V24(frame_header), frame_body))
    }
}

///
/// Apply 'ReadOf' to 'File'.
///
impl ReadOp for File {}

///
/// Mp3 metadata reader.
///
pub struct MetadataReader {
    next: Option<Status>,
    file: File,
}

impl MetadataReader {
    ///
    /// It create a new MetadataReader.
    /// @path: a file path.
    ///
    pub fn new(path: &str) -> Result<Self> {
        Ok(MetadataReader {
            next: None,
            file: File::open(path)?,
        })
    }

    //
    // It decide next unit that follow a head unit.
    // - if extended header exist, next is extended header. if not, next is frames.
    //
    fn set_head_next(&mut self, header: &Unit) {
        if let &Unit::Header(ref head) = header {
            let head = head.clone();

            if head.has_flag(HeadFlag::ExtendedHeader) {
                self.next = Some(Status::ExtendedHeader(head));
                return;
            }

            self.next = match self.file.frame_bytes(&head) {
                Err(_) => None,
                Ok(readable) => {
                    let r = Rc::new(RefCell::new(Box::new(readable)));
                    Some(Status::Frame(head, r))
                }
            };
        } else {
            self.next = None;
        }
    }

    //
    // The next unit of a extend head is a frame.
    //
    fn set_ext_head_next(&mut self, head: &Head) {
        self.next = match self.file.frame_bytes(&head) {
            Err(_) => None,
            Ok(readable) => {
                let r = Rc::new(RefCell::new(Box::new(readable)));
                Some(Status::Frame(head.clone(), r))
            }
        };
    }

    //
    // If frame id exist, read next frame. if does not exist, read the frame1.
    //
    fn set_frame_next(&mut self, head: &Head, readable_wrap: FrameReadable) {
        let ref_readable = readable_wrap.clone();
        let mut readable = readable_wrap.borrow_mut();

        //
        // The rule of frame id.
        //
        let frame_exist = match readable.look_string(4) {
            Ok(id) => {
                //
                // http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let regex = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}").unwrap();
                let matched = regex.is_match(&id);
                debug!("Frame Id:'{}', reg matched: {}", id, matched);

                matched
            }
            _ => false,
        };

        if frame_exist {
            self.next = Some(Status::Frame(head.clone(), ref_readable));
        } else {
            self.next = Some(Status::Frame1);
        }
    }
}

pub struct MetadataWriter<'a> {
    //
    // file path
    //
    path: &'a str,
}

impl<'a> MetadataWriter<'a> {
    pub fn new(path: &'a str) -> Result<Self> {
        Ok(MetadataWriter { path: path })
    }

    /// clean_write: it determin if rewrite all to version 4 or not. if it 'true', it rewrite to version 4.
    /// and in 2.2 'CRM', 'PIC'. in 2.3 'EQUA', 'IPLS', 'RVAD', 'TDAT', 'TIME', 'TORY', 'TRDA', 'TSIZ',
    /// 'TYER' frames are ignored.
    ///
    /// if it is false, it write with given 'units' parameter.
    /// but it checks version. all of the unit must have to same version.
    ///
    /// if both 'head' are not given, a 'head' will be created with version 4.
    pub fn write(&self, mut units: Vec<Unit>, clean_write: bool) -> Result<()> {
        if clean_write {
            units = self.fix_units(&units)?;
        } else {
            self.check_version(&units)?;
        }

        let (has_frame1, head_len, all_bytes) = self.to_bytes(units)?;
        let (orig_head_len, file_len) = self.metadata_length()?;

        let mut writable = OpenOptions::new().read(true)
            .write(true)
            .open(self.path)?;

        let head_diff_len = orig_head_len as i32 - head_len as i32;

        //
        // when new metadata size is shorter than original size.
        //
        if head_diff_len > 0 && file_len > head_diff_len as u64 {
            writable.unshift(head_diff_len as usize)?;

            let len = file_len - head_diff_len as u64;
            OpenOptions::new().write(true).open(self.path)?.set_len(len)?;
        }
        //
        // when new metadata size is larger than original size.
        //
        else if head_diff_len < 0 && file_len > head_diff_len.abs() as u64 {
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

    ///
    /// @return tuple. (origin header size, origin file size)
    ///
    fn metadata_length(&self) -> Result<(u32, u64)> {

        let i = MetadataReader::new(self.path)
            ?
            .filter(|m| match m {
                &Unit::Header(_) => true,
                _ => false,
            })
            .map(|unit| match unit {
                Unit::Header(head) => head.size,
                _ => 0,
            })
            .collect::<Vec<_>>();

        let header_length = if i.len() > 0 { i[0] } else { 0 };
        let file_len = File::open(self.path)?.metadata()?.len();

        Ok((header_length, file_len))
    }

    ///
    /// It checks that all the unit have the same version.
    ///
    fn check_version(&self, units: &Vec<Unit>) -> Result<()> {

        let head_unit = units.iter().find(|unit| match unit {
            &&Unit::Header(_) => true,
            _ => false,
        });

        let head_version = match head_unit {
            Some(&Unit::Header(ref head)) => head.version,
            _ => 4,
        };

        let err = Err(Error::new(ErrorKind::InvalidData, "exist different version of 'Unit'"));

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

    ///
    /// it rewite all the frames to version 4 and it removes frame version 1
    ///
    pub fn fix_units(&self, units: &Vec<Unit>) -> Result<Vec<Unit>> {
        let ret = units.iter().fold(Vec::new(), |mut vec, unit| {
            match unit {
                &Unit::Header(ref head) => {
                    let mut new_head = head.clone();
                    new_head.version = 4;
                    new_head.minor_version = 0;
                    vec.push(Unit::Header(new_head));
                }
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
                                    let body_id = framebody_to_id(&frame_body, 2);
                                    FrameHeaderV4 {
                                        id: frame2_to_frame4(body_id),
                                        size: 0,
                                        status_flag: 0,
                                        encoding_flag: 0,
                                    }
                                }
                                &FrameHeader::V23(ref header) => {
                                    let mut new_header = FrameHeaderV4 {
                                        id: header.id.to_owned(),
                                        size: 0,
                                        status_flag: 0,
                                        encoding_flag: 0,
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
                                }
                                &FrameHeader::V24(ref header) => header.clone(),

                            };
                            vec.push(Unit::FrameV2(FrameHeader::V24(new_frame_header),
                                                   frame_body.clone()));
                        }

                    }
                }
                _ => (),
            }

            vec
        });


        Ok(ret)
    }

    ///
    /// It transform the Head to byte array.
    ///
    pub fn head(&self, head: Head) -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![0u8; 0]);

        head.write(&mut writable, 0)?;

        let mut buf = Vec::new();
        let _ = writable.copy(&mut buf);

        Ok(buf)
    }

    ///
    /// It transform the Frame1 to byte array.
    ///
    pub fn frame1(&self, frame1: Frame1) -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![0u8; 0]);
        frame1.write(&mut writable)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    ///
    /// It transform the Frame2 to byte array.
    ///
    pub fn frame2(&self,
                  frame_header: &mut FrameHeaderV2,
                  frame_body: FrameBody)
                  -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![0u8; 0]);

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            match frame_body {
                FrameBody::OBJECT(_) => {}
                _ => {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Encrypted frame must be FrameBody::OBJECT."));
                }
            };
        }

        let (id, bytes) = framebody_as_bytes(&frame_body, 2)?;

        frame_header.id = id.to_string();
        frame_header.size = bytes.len() as u32;
        frame_header.write(&mut writable, 2)?;
        writable.write(&bytes)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    ///
    /// It transform the Frame3 to byte array.
    ///
    pub fn frame3(&self,
                  frame_header: &mut FrameHeaderV3,
                  frame_body: FrameBody)
                  -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![]);

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            match frame_body {
                FrameBody::OBJECT(object) => {
                    frame_header.size = object.data.len() as u32;
                    let _ = frame_header.write(&mut writable, 3);
                    let _ = writable.write(&object.data);

                    let mut buf = Vec::new();
                    writable.copy(&mut buf)?;

                    return Ok(buf);
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Encrypted frame must be FrameBody::OBJECT."));
                }
            };
        }

        let (id, mut bytes) = framebody_as_bytes(&frame_body, 3)?;

        frame_header.id = id.to_string();

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
            let _ = encoder.write(&bytes);
            bytes = encoder.finish()?;
            frame_header.size = bytes.len() as u32;
        } else {
            frame_header.size = bytes.len() as u32;
        }

        frame_header.write(&mut writable, 3)?;
        writable.write(&bytes)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    ///
    /// It transform the Frame4 to byte array.
    ///
    pub fn frame4(&self,
                  frame_header: &mut FrameHeaderV4,
                  frame_body: FrameBody)
                  -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![]);

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            match frame_body {
                FrameBody::OBJECT(object) => {
                    frame_header.size = object.data.len() as u32;
                    let _ = frame_header.write(&mut writable, 4);
                    let _ = writable.write(&object.data);

                    let mut buf = Vec::new();
                    writable.copy(&mut buf)?;

                    return Ok(buf);
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Encrypted frame must be FrameBody::OBJECT."));
                }
            }
        }

        let (id, mut bytes) = framebody_as_bytes(&frame_body, 4)?;

        frame_header.id = id.to_string();
        frame_header.size = bytes.len() as u32;

        if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
            debug!("write {} unsynchronization", id);

            let len = bytes.len();
            bytes = Cursor::new(bytes).to_unsynchronize(len)?;
            frame_header.size = bytes.len() as u32
        }

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("write {} compression", id);

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
            let _ = encoder.write(&bytes);
            bytes = encoder.finish()?;
            frame_header.size = bytes.len() as u32
        }

        frame_header.write(&mut writable, 4)?;
        writable.write(&bytes)?;

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    pub fn frames(&self, frames: Vec<(FrameHeader, FrameBody)>) -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![]);
        for frame in frames {
            let _ = writable.write(&self.frame(frame)?);
        }

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok(buf)
    }

    ///
    /// It transform the Frame 2.x to byte array.
    ///
    pub fn frame(&self, frame: (FrameHeader, FrameBody)) -> Result<Vec<u8>> {
        let mut writable = Cursor::new(vec![]);

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

    ///
    /// It transform all the units to byte array.
    /// if Unit::Header is not given, create new one as version 4.
    ///
    pub fn to_bytes(&self, units: Vec<Unit>) -> Result<(bool, u32, Vec<u8>)> {
        let mut writable = Cursor::new(vec![]);

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

        let mut head = match head_wrap {
            None => {
                Head {
                    tag_id: "ID3".to_string(),
                    version: 4,
                    minor_version: 0,
                    flag: 0,
                    size: 0,
                }
            }
            _ => head_wrap.unwrap(),
        };

        let mut frame_bytes = self.frames(frames)?;

        //
        // Head level Unsynchronisation.
        //
        if head.has_flag(HeadFlag::Unsynchronisation) {
            debug!("head unsynchronisation");

            let len = frame_bytes.len();
            frame_bytes = Cursor::new(frame_bytes).to_unsynchronize(len)?;
        };

        head.size = frame_bytes.len() as u32;

        let head_size = head.size;

        writable.write(&self.head(head)?)?;
        writable.write(&frame_bytes)?;

        let has_frame1 = match frame1_wrap {
            None => false,
            Some(frame1) => {
                writable.write(&self.frame1(frame1)?)?;
                true
            }
        };

        let mut buf = Vec::new();
        writable.copy(&mut buf)?;

        Ok((has_frame1, head_size, buf))
    }
}

///
/// MetadataReader implement a Iterator.
///
/// because instead of loading all the metadata information,
/// it support to read Unit step by step and there is convenient methods
/// in Iterator like filter, map.
///
impl Iterator for MetadataReader {
    type Item = Unit;

    fn next(&mut self) -> Option<Self::Item> {

        let next = self.next.take();

        if next.is_none() {
            return match self.file.head() {
                Err(_) => None,
                Ok(header) => {
                    self.set_head_next(&header);
                    Some(header)
                }
            };
        }

        match next.unwrap() {
            Status::ExtendedHeader(ref head) => {
                match self.file.ext_head(head) {
                    Err(_) => None,
                    Ok(ext_head) => {
                        self.set_ext_head_next(head);
                        Some(ext_head)
                    }
                }
            }

            Status::Frame(ref head, ref readable) => {
                match self.file.frame(head, readable.clone()) {
                    Err(_) => None,
                    Ok(frame) => {
                        self.set_frame_next(head, readable.clone());
                        Some(frame)
                    }
                }
            }

            Status::Frame1 => {
                match self.file.metadata() {
                    Err(_) => None,
                    Ok(metadata) => {
                        match self.file.frame1(metadata.len() as usize) {
                            Err(_) => None,
                            Ok(frame1) => {
                                self.next = None;
                                Some(frame1)
                            }
                        }
                    }
                }
            }
        }

    }
}