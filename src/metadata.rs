pub extern crate regex;
extern crate flate2;

use self::flate2::read::ZlibDecoder;

use errors::*;
use frame::*;
use frame::id::*;
use util;
use readable::{
    Readable,
    ReadableFactory
};
use writable::{
    Writable,
    WritableFactory
};

use std::cell::RefCell;
use std::error::Error;
use std::fs::{
    File,
    OpenOptions
};
use std::io::{
    Cursor,
    Read
};
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
    None
}

#[derive(Debug)]
pub enum Unit {
    Header(Head),
    // TODO not yet implemented
    ExtendedHeader(Vec<u8>),
    FrameV1(Frame1),
    FrameV2(FrameHeader, FrameData),
    Unknown(String)
}

pub struct MetadataReader {
    next: Status,
    file_len: u64
}

impl MetadataReader {
    pub fn new(path: &str) -> result::Result<Self, ParsingError> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len();
        let readable = file.to_readable();

        Ok(MetadataReader {
            next: Status::Head(Rc::new(RefCell::new(Box::new(readable)))),
            file_len: file_len
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
            },
            _ => false
        }
    }

    fn head(&mut self,
            readable_wrap: RefFileReader)
            -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();
        let head = Head::read(readable.to_readable(10)?)?;
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
            _ => readable.synchsafe()?
        };
        let extended_bytes = readable.bytes(size as usize)?;
        let head_size = head_wrap.borrow().size as usize;
        let frame_bytes = readable.bytes(head_size)?;
        let frame_readable = Cursor::new(frame_bytes).to_readable();
        let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

        self.next = Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap);

        Ok(Unit::ExtendedHeader(extended_bytes))
    }

    fn frame1(&mut self, readable: &mut Readable<File>)
              -> result::Result<Frame1, ParsingError> {
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

    // version 2.2
    fn frame2(&mut self,
              head: &Head,
              readable: &mut Readable<Cursor<Vec<u8>>>)
              -> result::Result<Unit, ParsingError> {
        let frame_header = FrameHeaderV2::read(readable)?;
        let frame_body = if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            FrameData::SKIP("Encrypted frame".to_string())
        } else {
            let frame_readable = readable.to_readable(frame_header.size as usize)?;
            frame_data(frame_header.id.as_str(), head.version, frame_readable)?
        };

        Ok(Unit::FrameV2(FrameHeader::V22(frame_header), frame_body))
    }

    // v2.3
    fn frame3(&mut self, head: &Head, readable: &mut Readable<Cursor<Vec<u8>>>)
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
            FrameData::SKIP("Encrypted frame".to_string())
        } else {
            let frame_readable = Cursor::new(body_bytes).to_readable();
            frame_data(frame_header.id.as_str(), head.version, frame_readable)?
        };

        Ok(Unit::FrameV2(FrameHeader::V23(frame_header), frame_body))
    }

    // v2.4
    fn frame4(&mut self,
              head: &Head,
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

            let _ = readable.u32()?;

            let real_frame = body_bytes.clone();
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&real_frame[..]);
            let _ = decoder.read_to_end(&mut out);

            body_bytes = out;
        }

        let frame_body = if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            FrameData::SKIP("Encrypted frame".to_string())
        } else {
            let frame_readable = Cursor::new(body_bytes).to_readable();
            frame_data(frame_header.id.as_str(), head.version, frame_readable)?
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
            2 => self.frame2(&head_wrap.borrow(), &mut frame_readable),
            3 => self.frame3(&head_wrap.borrow(), &mut frame_readable),
            4 => self.frame4(&head_wrap.borrow(), &mut frame_readable),
            _ => self.frame4(&head_wrap.borrow(), &mut frame_readable)
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
                _ => None
            }
        }

        fn extended_head(next: &Status) -> Option<(RefHead, RefFileReader)> {
            match next {
                &Status::ExtendedHeader(ref head, ref readable) =>
                    Some((head.clone(), readable.clone())),
                _ => None
            }
        }

        fn frame(next: &Status) -> Option<(RefHead, RefFileReader, RefByteReader)> {
            match next {
                &Status::Frame(ref head, ref readable, ref frame_readable) =>
                    Some((head.clone(), readable.clone(), frame_readable.clone())),
                _ => None
            }
        }

        let head = head(&self.next);
        let extended_header = extended_head(&self.next);
        let frame = frame(&self.next);

        match self.next {
            Status::Head(_) => match self.head(head.unwrap()) {
                Ok(data) => Some(data),
                Err(msg) => {
                    debug!("Stop on 'Head': {}", msg);
                    None
                }
            },
            Status::ExtendedHeader(_, _) => {
                let (head, readable) = extended_header.unwrap();
                match self.extended_head(head, readable) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        debug!("Stop on 'Extended Head': {}", msg);
                        None
                    }
                }
            },
            Status::Frame(_, _, _) => {
                let (head, readable, frame_readable) = frame.unwrap();
                match self.frame(head, readable, frame_readable) {
                    Ok(data) => {
                        Some(data)
                    },
                    Err(msg) => {
                        debug!("Ignored 'Frame': {}", msg);
                        Some(Unit::Unknown(msg.description().to_string()))
                    }
                }
            }
            _ => None
        }
    }
}

pub struct MetadataWriter<'a> {
    path: &'a str
}

impl<'a> MetadataWriter<'a> {
    pub fn new(path: &'a str) -> result::Result<Self, WriteError> {
        Ok(MetadataWriter {
            path: path
        })
    }

    pub fn head_to_bytes(&self, head: &Head) -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));
        head.write(&mut writable)?;

        let mut buf = Vec::new();
        writable.as_mut().read_to_end(&mut buf)?;

        Ok(buf)
    }

    pub fn frame_to_bytes(&self, frames: Vec<(FrameHeader, FrameData)>) -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));
        for frame in frames {
            let (mut frame_header, frame_data) = frame;

            match frame_header {
                FrameHeader::V22(ref mut frame_header) => {
                    let (id, bytes) = write_frame_data(&frame_data, 2)?;
                    frame_header.id = id.to_string();
                    frame_header.size = bytes.len() as u32;
                    frame_header.write(&mut writable)?;
                    writable.write(&bytes)?;
                },
                FrameHeader::V23(ref mut frame_header) => {
                    let (id, bytes) = write_frame_data(&frame_data, 3)?;
                    frame_header.id = id.to_string();
                    frame_header.size = bytes.len() as u32;
                    frame_header.write(&mut writable)?;
                    writable.write(&bytes)?;
                },
                FrameHeader::V24(ref mut frame_header) => {
                    let (id, bytes) = write_frame_data(&frame_data, 4)?;
                    frame_header.id = id.to_string();
                    frame_header.size = if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
                        util::to_unsynchronize(&bytes).len() as u32
                    } else {
                        bytes.len() as u32
                    };
                    frame_header.write(&mut writable)?;
                    writable.write(&bytes)?;
                },
            };
        }

        let mut buf = Vec::new();
        writable.as_mut().read_to_end(&mut buf)?;

        Ok(buf)
    }

    pub fn frame1_to_bytes(&self, frame1: Frame1) -> result::Result<Vec<u8>, WriteError> {
        let mut writable = Writable::new(Cursor::new(vec![]));
        frame1.write(&mut writable)?;

        let mut buf = Vec::new();
        writable.as_mut().read_to_end(&mut buf)?;

        Ok(buf)
    }

    pub fn write(&mut self, units: Vec<Unit>) -> result::Result<(), WriteError> {
        let mut head_wrap = None;
        let mut frame1_wrap = None;
        let mut frames = Vec::new();
        for unit in units {
            match unit {
                Unit::Header(head) => head_wrap = Some(head),
                Unit::FrameV1(frame) => frame1_wrap = Some(frame),
                Unit::FrameV2(frame_header, frame_data) =>
                    frames.push((frame_header, frame_data)),
                _ => ()
            }
        };

        let mut head = if head_wrap.is_none() {
            Head { version: 4, minor_version: 0, flag: 0, size: 0 }
        } else {
            head_wrap.unwrap()
        };

        let frame_bytes = if head.has_flag(HeadFlag::Unsynchronisation) {
            util::to_unsynchronize(&self.frame_to_bytes(frames)?)
        } else {
            self.frame_to_bytes(frames)?
        };
        head.size = frame_bytes.len() as u32;

        let head_bytes = self.head_to_bytes(&head)?;
        let frame1_bytes = if let Some(frame1) = frame1_wrap {
            self.frame1_to_bytes(frame1)?
        } else {
            vec![0u8; 0]
        };

        let file_len = self.adjust_metadata_size(&head)?;

        let mut writable = OpenOptions::new().write(true).open(self.path)?.to_writable();
        writable.write(&head_bytes)?;
        writable.write(&frame_bytes)?;

        if frame1_bytes.len() > 0 {
            writable.position(file_len as usize - 128)?;
            writable.write(&frame1_bytes)?;
        }

        Ok(())
    }

    fn adjust_metadata_size(&self, head: &Head) -> result::Result<u64, WriteError> {
        let metadata_length = self.metadata_length();
        let file = OpenOptions::new().read(true).write(true).open(self.path)?;
        let file_len = file.metadata()?.len();

        let mut writable = file.to_writable();

        if metadata_length > head.size {
            let unshift_size = (metadata_length - head.size) as u64;

            writable.unshift(unshift_size as usize)?;

            let len = file_len - unshift_size;
            OpenOptions::new().write(true).open(self.path)?.set_len(len)?;

            Ok(len)
        } else {
            writable.shift((head.size - metadata_length) as usize)?;

            Ok(file_len)
        }
    }

    fn metadata_length(&self) -> u32 {
        match self::MetadataReader::new(self.path) {
            Ok(meta) => {
                for m in meta {
                    match m {
                        Unit::Header(header) => return header.size,
                        _ => ()
                    }
                }
                0
            },
            _ => 0
        }
    }
}

fn frame_data(id: &str,
              version: u8,
              mut readable: Readable<Cursor<Vec<u8>>>)
              -> result::Result<FrameData, ParsingError> {
    let frame_data = match id.as_ref() {
        BUF_STR => FrameData::BUF(BUF::read(&mut readable)?),
        CNT_STR => FrameData::PCNT(PCNT::read(&mut readable)?),
        COM_STR => FrameData::COMM(COMM::read(&mut readable)?),
        CRA_STR => FrameData::AENC(AENC::read(&mut readable)?),
        CRM_STR => FrameData::CRM(CRM::read(&mut readable)?),
        ETC_STR => FrameData::ETCO(ETCO::read(&mut readable)?),
        EQU_STR => FrameData::EQUA(EQUA::read(&mut readable)?),
        GEO_STR => FrameData::GEOB(GEOB::read(&mut readable)?),
        IPL_STR => FrameData::IPLS(IPLS::read(&mut readable)?),
        LNK_STR => FrameData::LINK(LINK::read(&mut readable, version)?),
        MCI_STR => FrameData::MCDI(MCDI::read(&mut readable)?),
        MLL_STR => FrameData::MLLT(MLLT::read(&mut readable)?),
        PIC_STR => FrameData::PIC(PIC::read(&mut readable)?),
        POP_STR => FrameData::POPM(POPM::read(&mut readable)?),
        REV_STR => FrameData::RVRB(RVRB::read(&mut readable)?),
        RVA_STR => FrameData::RVAD(RVA2::read(&mut readable)?),
        SLT_STR => FrameData::SYLT(SYLT::read(&mut readable)?),
        STC_STR => FrameData::SYTC(SYTC::read(&mut readable)?),
        TAL_STR => FrameData::TALB(TEXT::read(&mut readable, id)?),
        TBP_STR => FrameData::TBPM(TEXT::read(&mut readable, id)?),
        TCM_STR => FrameData::TCOM(TEXT::read(&mut readable, id)?),
        TCO_STR => FrameData::TCON(TEXT::read(&mut readable, id)?),
        TCR_STR => FrameData::TCOP(TEXT::read(&mut readable, id)?),
        TDA_STR => FrameData::TDAT(TEXT::read(&mut readable, id)?),
        TDY_STR => FrameData::TDLY(TEXT::read(&mut readable, id)?),
        TEN_STR => FrameData::TENC(TEXT::read(&mut readable, id)?),
        TFT_STR => FrameData::TFLT(TEXT::read(&mut readable, id)?),
        TIM_STR => FrameData::TIME(TEXT::read(&mut readable, id)?),
        TKE_STR => FrameData::TKEY(TEXT::read(&mut readable, id)?),
        TLA_STR => FrameData::TLAN(TEXT::read(&mut readable, id)?),
        TLE_STR => FrameData::TLEN(TEXT::read(&mut readable, id)?),
        TMT_STR => FrameData::TMED(TEXT::read(&mut readable, id)?),
        TOA_STR => FrameData::TMED(TEXT::read(&mut readable, id)?),
        TOF_STR => FrameData::TOFN(TEXT::read(&mut readable, id)?),
        TOL_STR => FrameData::TOLY(TEXT::read(&mut readable, id)?),
        TOR_STR => FrameData::TORY(TEXT::read(&mut readable, id)?),
        TOT_STR => FrameData::TOAL(TEXT::read(&mut readable, id)?),
        TP1_STR => FrameData::TPE1(TEXT::read(&mut readable, id)?),
        TP2_STR => FrameData::TPE2(TEXT::read(&mut readable, id)?),
        TP3_STR => FrameData::TPE3(TEXT::read(&mut readable, id)?),
        TP4_STR => FrameData::TPE4(TEXT::read(&mut readable, id)?),
        TPA_STR => FrameData::TPOS(TEXT::read(&mut readable, id)?),
        TPB_STR => FrameData::TPUB(TEXT::read(&mut readable, id)?),
        TRC_STR => FrameData::TSRC(TEXT::read(&mut readable, id)?),
        TRD_STR => FrameData::TRDA(TEXT::read(&mut readable, id)?),
        TRK_STR => FrameData::TRCK(TEXT::read(&mut readable, id)?),
        TSI_STR => FrameData::TSIZ(TEXT::read(&mut readable, id)?),
        TSS_STR => FrameData::TSSE(TEXT::read(&mut readable, id)?),
        TT1_STR => FrameData::TIT1(TEXT::read(&mut readable, id)?),
        TT2_STR => FrameData::TIT2(TEXT::read(&mut readable, id)?),
        TT3_STR => FrameData::TIT3(TEXT::read(&mut readable, id)?),
        TXT_STR => FrameData::TEXT(TEXT::read(&mut readable, id)?),
        TYE_STR => FrameData::TYER(TEXT::read(&mut readable, id)?),
        TXX_STR => FrameData::TXXX(TXXX::read(&mut readable)?),
        UFI_STR => FrameData::UFID(UFID::read(&mut readable)?),
        ULT_STR => FrameData::USLT(USLT::read(&mut readable)?),
        WAF_STR => FrameData::WOAF(LINK::read(&mut readable, version)?),
        WAR_STR => FrameData::WOAR(LINK::read(&mut readable, version)?),
        WAS_STR => FrameData::WOAS(LINK::read(&mut readable, version)?),
        WCM_STR => FrameData::WCOM(LINK::read(&mut readable, version)?),
        WCP_STR => FrameData::WCOP(LINK::read(&mut readable, version)?),
        WPB_STR => FrameData::WPUB(LINK::read(&mut readable, version)?),
        WXX_STR => FrameData::WXXX(WXXX::read(&mut readable)?),
        AENC_STR => FrameData::AENC(AENC::read(&mut readable)?),
        APIC_STR => FrameData::APIC(APIC::read(&mut readable)?),
        ASPI_STR => FrameData::ASPI(ASPI::read(&mut readable)?),
        COMM_STR => FrameData::COMM(COMM::read(&mut readable)?),
        COMR_STR => FrameData::COMR(COMR::read(&mut readable)?),
        ENCR_STR => FrameData::ENCR(ENCR::read(&mut readable)?),
        EQUA_STR => FrameData::EQUA(EQUA::read(&mut readable)?),
        EQU2_STR => FrameData::EQU2(EQU2::read(&mut readable)?),
        ETCO_STR => FrameData::ETCO(ETCO::read(&mut readable)?),
        GEOB_STR => FrameData::GEOB(GEOB::read(&mut readable)?),
        GRID_STR => FrameData::GRID(GRID::read(&mut readable)?),
        IPLS_STR => FrameData::IPLS(IPLS::read(&mut readable)?),
        LINK_STR => FrameData::LINK(LINK::read(&mut readable, version)?),
        MCDI_STR => FrameData::MCDI(MCDI::read(&mut readable)?),
        MLLT_STR => FrameData::MLLT(MLLT::read(&mut readable)?),
        OWNE_STR => FrameData::OWNE(OWNE::read(&mut readable)?),
        PRIV_STR => FrameData::PRIV(PRIV::read(&mut readable)?),
        PCNT_STR => FrameData::PCNT(PCNT::read(&mut readable)?),
        POPM_STR => FrameData::POPM(POPM::read(&mut readable)?),
        POSS_STR => FrameData::POSS(POSS::read(&mut readable)?),
        RBUF_STR => FrameData::RBUF(RBUF::read(&mut readable)?),
        RVAD_STR => FrameData::RVAD(RVA2::read(&mut readable)?),
        RVA2_STR => FrameData::RVA2(RVA2::read(&mut readable)?),
        RVRB_STR => FrameData::RVRB(RVRB::read(&mut readable)?),
        SEEK_STR => FrameData::SEEK(SEEK::read(&mut readable)?),
        SIGN_STR => FrameData::SIGN(SIGN::read(&mut readable)?),
        SYLT_STR => FrameData::SYLT(SYLT::read(&mut readable)?),
        SYTC_STR => FrameData::SYTC(SYTC::read(&mut readable)?),
        UFID_STR => FrameData::UFID(UFID::read(&mut readable)?),
        USER_STR => FrameData::USER(USER::read(&mut readable)?),
        USLT_STR => FrameData::USLT(USLT::read(&mut readable)?),
        TALB_STR => FrameData::TALB(TEXT::read(&mut readable, id)?),
        TBPM_STR => FrameData::TBPM(TEXT::read(&mut readable, id)?),
        TCOM_STR => FrameData::TCOM(TEXT::read(&mut readable, id)?),
        TCON_STR => FrameData::TCON(TEXT::read(&mut readable, id)?),
        TCOP_STR => FrameData::TCOP(TEXT::read(&mut readable, id)?),
        TDAT_STR => FrameData::TDAT(TEXT::read(&mut readable, id)?),
        TDEN_STR => FrameData::TDEN(TEXT::read(&mut readable, id)?),
        TDLY_STR => FrameData::TDLY(TEXT::read(&mut readable, id)?),
        TDOR_STR => FrameData::TDOR(TEXT::read(&mut readable, id)?),
        TDRC_STR => FrameData::TDRC(TEXT::read(&mut readable, id)?),
        TDRL_STR => FrameData::TDRL(TEXT::read(&mut readable, id)?),
        TDTG_STR => FrameData::TDTG(TEXT::read(&mut readable, id)?),
        TENC_STR => FrameData::TENC(TEXT::read(&mut readable, id)?),
        TEXT_STR => FrameData::TEXT(TEXT::read(&mut readable, id)?),
        TIME_STR => FrameData::TIME(TEXT::read(&mut readable, id)?),
        TFLT_STR => FrameData::TFLT(TEXT::read(&mut readable, id)?),
        TIPL_STR => FrameData::TIPL(TEXT::read(&mut readable, id)?),
        TIT1_STR => FrameData::TIT1(TEXT::read(&mut readable, id)?),
        TIT2_STR => FrameData::TIT2(TEXT::read(&mut readable, id)?),
        TIT3_STR => FrameData::TIT3(TEXT::read(&mut readable, id)?),
        TKEY_STR => FrameData::TKEY(TEXT::read(&mut readable, id)?),
        TLAN_STR => FrameData::TLAN(TEXT::read(&mut readable, id)?),
        TLEN_STR => FrameData::TLEN(TEXT::read(&mut readable, id)?),
        TMCL_STR => FrameData::TMCL(TEXT::read(&mut readable, id)?),
        TMED_STR => FrameData::TMED(TEXT::read(&mut readable, id)?),
        TMOO_STR => FrameData::TMOO(TEXT::read(&mut readable, id)?),
        TOAL_STR => FrameData::TOAL(TEXT::read(&mut readable, id)?),
        TOFN_STR => FrameData::TOFN(TEXT::read(&mut readable, id)?),
        TOLY_STR => FrameData::TOLY(TEXT::read(&mut readable, id)?),
        TOPE_STR => FrameData::TOPE(TEXT::read(&mut readable, id)?),
        TORY_STR => FrameData::TORY(TEXT::read(&mut readable, id)?),
        TOWN_STR => FrameData::TOWN(TEXT::read(&mut readable, id)?),
        TPE1_STR => FrameData::TPE1(TEXT::read(&mut readable, id)?),
        TPE2_STR => FrameData::TPE2(TEXT::read(&mut readable, id)?),
        TPE3_STR => FrameData::TPE3(TEXT::read(&mut readable, id)?),
        TPE4_STR => FrameData::TPE4(TEXT::read(&mut readable, id)?),
        TPOS_STR => FrameData::TPOS(TEXT::read(&mut readable, id)?),
        TPRO_STR => FrameData::TPRO(TEXT::read(&mut readable, id)?),
        TPUB_STR => FrameData::TPUB(TEXT::read(&mut readable, id)?),
        TRCK_STR => FrameData::TRCK(TEXT::read(&mut readable, id)?),
        TRDA_STR => FrameData::TRDA(TEXT::read(&mut readable, id)?),
        TRSN_STR => FrameData::TRSN(TEXT::read(&mut readable, id)?),
        TSIZ_STR => FrameData::TSIZ(TEXT::read(&mut readable, id)?),
        TRSO_STR => FrameData::TRSO(TEXT::read(&mut readable, id)?),
        TSOA_STR => FrameData::TSOA(TEXT::read(&mut readable, id)?),
        TSOP_STR => FrameData::TSOP(TEXT::read(&mut readable, id)?),
        TSOT_STR => FrameData::TSOT(TEXT::read(&mut readable, id)?),
        TSRC_STR => FrameData::TSRC(TEXT::read(&mut readable, id)?),
        TSSE_STR => FrameData::TSSE(TEXT::read(&mut readable, id)?),
        TYER_STR => FrameData::TYER(TEXT::read(&mut readable, id)?),
        TSST_STR => FrameData::TSST(TEXT::read(&mut readable, id)?),
        TXXX_STR => FrameData::TXXX(TXXX::read(&mut readable)?),
        WCOM_STR => FrameData::WCOM(LINK::read(&mut readable, version)?),
        WCOP_STR => FrameData::WCOP(LINK::read(&mut readable, version)?),
        WOAF_STR => FrameData::WOAF(LINK::read(&mut readable, version)?),
        WOAR_STR => FrameData::WOAR(LINK::read(&mut readable, version)?),
        WOAS_STR => FrameData::WOAS(LINK::read(&mut readable, version)?),
        WORS_STR => FrameData::WORS(LINK::read(&mut readable, version)?),
        WPAY_STR => FrameData::WPAY(LINK::read(&mut readable, version)?),
        WPUB_STR => FrameData::WPUB(LINK::read(&mut readable, version)?),
        WXXX_STR => FrameData::WXXX(WXXX::read(&mut readable)?),
        _ => {
            warn!("No frame id found!! '{}'", id);
            FrameData::TEXT(TEXT::read(&mut readable, id)?)
        }
    };

    Ok(frame_data)
}

fn write_frame_data(frame_data: &FrameData, version: u8)
                    -> result::Result<(&str, Vec<u8>), WriteError> {
    let mut writable = Cursor::new(vec![0u8; 0]).to_writable();

    let id = match frame_data {
        &FrameData::BUF(ref frame) => {
            frame.write(&mut writable)?;
            id::BUF_STR
        },
        &FrameData::CRM(ref frame) => {
            frame.write(&mut writable)?;
            id::CRM_STR
        },
        &FrameData::PIC(ref frame) => {
            frame.write(&mut writable)?;
            id::PIC_STR
        },
        &FrameData::AENC(ref frame) => {
            frame.write(&mut writable)?;
            id::AENC_STR
        },
        &FrameData::APIC(ref frame) => {
            frame.write(&mut writable)?;
            id::APIC_STR
        },
        &FrameData::ASPI(ref frame) => {
            frame.write(&mut writable)?;
            id::ASPI_STR
        },
        &FrameData::COMM(ref frame) => {
            frame.write(&mut writable)?;
            id::COMM_STR
        },
        &FrameData::COMR(ref frame) => {
            frame.write(&mut writable)?;
            id::COMR_STR
        },
        &FrameData::ENCR(ref frame) => {
            frame.write(&mut writable)?;
            id::ENCR_STR
        },
        &FrameData::EQUA(ref frame) => {
            frame.write(&mut writable)?;
            id::EQUA_STR
        },
        &FrameData::EQU2(ref frame) => {
            frame.write(&mut writable)?;
            id::EQU2_STR
        },
        &FrameData::ETCO(ref frame) => {
            frame.write(&mut writable)?;
            id::ETCO_STR
        },
        &FrameData::GEOB(ref frame) => {
            frame.write(&mut writable)?;
            id::GEOB_STR
        },
        &FrameData::GRID(ref frame) => {
            frame.write(&mut writable)?;
            id::GRID_STR
        },
        &FrameData::IPLS(ref frame) => {
            frame.write(&mut writable)?;
            id::IPLS_STR
        },
        &FrameData::LINK(ref frame) => {
            frame.write(&mut writable, version)?;
            id::LINK_STR
        },
        &FrameData::MCDI(ref frame) => {
            frame.write(&mut writable)?;
            id::MCDI_STR
        },
        &FrameData::MLLT(ref frame) => {
            frame.write(&mut writable)?;
            id::MLLT_STR
        },
        &FrameData::OWNE(ref frame) => {
            frame.write(&mut writable)?;
            id::OWNE_STR
        },
        &FrameData::PRIV(ref frame) => {
            frame.write(&mut writable)?;
            id::PRIV_STR
        },
        &FrameData::PCNT(ref frame) => {
            frame.write(&mut writable)?;
            id::PCNT_STR
        },
        &FrameData::POPM(ref frame) => {
            frame.write(&mut writable)?;
            id::POPM_STR
        },
        &FrameData::POSS(ref frame) => {
            frame.write(&mut writable)?;
            id::POSS_STR
        },
        &FrameData::RBUF(ref frame) => {
            frame.write(&mut writable)?;
            id::RBUF_STR
        },
        &FrameData::RVAD(ref frame) => {
            frame.write(&mut writable)?;
            id::RVAD_STR
        },
        &FrameData::RVA2(ref frame) => {
            frame.write(&mut writable)?;
            id::RVA2_STR
        },
        &FrameData::RVRB(ref frame) => {
            frame.write(&mut writable)?;
            id::RVRB_STR
        },
        &FrameData::SEEK(ref frame) => {
            frame.write(&mut writable)?;
            id::SEEK_STR
        },
        &FrameData::SIGN(ref frame) => {
            frame.write(&mut writable)?;
            id::SIGN_STR
        },
        &FrameData::SYLT(ref frame) => {
            frame.write(&mut writable)?;
            id::SYLT_STR
        },
        &FrameData::SYTC(ref frame) => {
            frame.write(&mut writable)?;
            id::SYTC_STR
        },
        &FrameData::TALB(ref frame) => {
            frame.write(&mut writable)?;
            id::TALB_STR
        },
        &FrameData::TBPM(ref frame) => {
            frame.write(&mut writable)?;
            id::TBPM_STR
        },
        &FrameData::TCOM(ref frame) => {
            frame.write(&mut writable)?;
            id::TCOM_STR
        },
        &FrameData::TCON(ref frame) => {
            frame.write(&mut writable)?;
            id::TCON_STR
        },
        &FrameData::TCOP(ref frame) => {
            frame.write(&mut writable)?;
            id::TCOP_STR
        },
        &FrameData::TDAT(ref frame) => {
            frame.write(&mut writable)?;
            id::TDAT_STR
        },
        &FrameData::TDEN(ref frame) => {
            frame.write(&mut writable)?;
            id::TDEN_STR
        },
        &FrameData::TDLY(ref frame) => {
            frame.write(&mut writable)?;
            id::TDLY_STR
        },
        &FrameData::TDOR(ref frame) => {
            frame.write(&mut writable)?;
            id::TDOR_STR
        },
        &FrameData::TDRC(ref frame) => {
            frame.write(&mut writable)?;
            id::TDRC_STR
        },
        &FrameData::TDRL(ref frame) => {
            frame.write(&mut writable)?;
            id::TDRL_STR
        },
        &FrameData::TDTG(ref frame) => {
            frame.write(&mut writable)?;
            id::TDTG_STR
        },
        &FrameData::TENC(ref frame) => {
            frame.write(&mut writable)?;
            id::TENC_STR
        },
        &FrameData::TEXT(ref frame) => {
            frame.write(&mut writable)?;
            id::TEXT_STR
        },
        &FrameData::TFLT(ref frame) => {
            frame.write(&mut writable)?;
            id::TFLT_STR
        },
        &FrameData::TIME(ref frame) => {
            frame.write(&mut writable)?;
            id::TIME_STR
        },
        &FrameData::TIPL(ref frame) => {
            frame.write(&mut writable)?;
            id::TIPL_STR
        },
        &FrameData::TIT1(ref frame) => {
            frame.write(&mut writable)?;
            id::TIT1_STR
        },
        &FrameData::TIT2(ref frame) => {
            frame.write(&mut writable)?;
            id::TIT2_STR
        },
        &FrameData::TIT3(ref frame) => {
            frame.write(&mut writable)?;
            id::TIT3_STR
        },
        &FrameData::TKEY(ref frame) => {
            frame.write(&mut writable)?;
            id::TKEY_STR
        },
        &FrameData::TLAN(ref frame) => {
            frame.write(&mut writable)?;
            id::TLAN_STR
        },
        &FrameData::TLEN(ref frame) => {
            frame.write(&mut writable)?;
            id::TLEN_STR
        },
        &FrameData::TMCL(ref frame) => {
            frame.write(&mut writable)?;
            id::TMCL_STR
        },
        &FrameData::TMED(ref frame) => {
            frame.write(&mut writable)?;
            id::TMED_STR
        },
        &FrameData::TMOO(ref frame) => {
            frame.write(&mut writable)?;
            id::TMOO_STR
        },
        &FrameData::TOAL(ref frame) => {
            frame.write(&mut writable)?;
            id::TOAL_STR
        },
        &FrameData::TOFN(ref frame) => {
            frame.write(&mut writable)?;
            id::TOFN_STR
        },
        &FrameData::TOLY(ref frame) => {
            frame.write(&mut writable)?;
            id::TOLY_STR
        },
        &FrameData::TOPE(ref frame) => {
            frame.write(&mut writable)?;
            id::TOPE_STR
        },
        &FrameData::TORY(ref frame) => {
            frame.write(&mut writable)?;
            id::TORY_STR
        },
        &FrameData::TOWN(ref frame) => {
            frame.write(&mut writable)?;
            id::TOWN_STR
        },
        &FrameData::TPE1(ref frame) => {
            frame.write(&mut writable)?;
            id::TPE1_STR
        },
        &FrameData::TPE2(ref frame) => {
            frame.write(&mut writable)?;
            id::TPE2_STR
        },
        &FrameData::TPE3(ref frame) => {
            frame.write(&mut writable)?;
            id::TPE3_STR
        },
        &FrameData::TPE4(ref frame) => {
            frame.write(&mut writable)?;
            id::TPE4_STR
        },
        &FrameData::TPOS(ref frame) => {
            frame.write(&mut writable)?;
            id::TPOS_STR
        },
        &FrameData::TPRO(ref frame) => {
            frame.write(&mut writable)?;
            id::TPRO_STR
        },
        &FrameData::TPUB(ref frame) => {
            frame.write(&mut writable)?;
            id::TPUB_STR
        },
        &FrameData::TRCK(ref frame) => {
            frame.write(&mut writable)?;
            id::TRCK_STR
        },
        &FrameData::TRDA(ref frame) => {
            frame.write(&mut writable)?;
            id::TRDA_STR
        },
        &FrameData::TRSN(ref frame) => {
            frame.write(&mut writable)?;
            id::TRSN_STR
        },
        &FrameData::TRSO(ref frame) => {
            frame.write(&mut writable)?;
            id::TRSO_STR
        },
        &FrameData::TSIZ(ref frame) => {
            frame.write(&mut writable)?;
            id::TSIZ_STR
        },
        &FrameData::TSOA(ref frame) => {
            frame.write(&mut writable)?;
            id::TSOA_STR
        },
        &FrameData::TSOP(ref frame) => {
            frame.write(&mut writable)?;
            id::TSOP_STR
        },
        &FrameData::TSOT(ref frame) => {
            frame.write(&mut writable)?;
            id::TSOT_STR
        },
        &FrameData::TSRC(ref frame) => {
            frame.write(&mut writable)?;
            id::TSRC_STR
        },
        &FrameData::TSSE(ref frame) => {
            frame.write(&mut writable)?;
            id::TSSE_STR
        },
        &FrameData::TYER(ref frame) => {
            frame.write(&mut writable)?;
            id::TYER_STR
        },
        &FrameData::TSST(ref frame) => {
            frame.write(&mut writable)?;
            id::TSST_STR
        },
        &FrameData::TXXX(ref frame) => {
            frame.write(&mut writable)?;
            id::TXXX_STR
        },
        &FrameData::UFID(ref frame) => {
            frame.write(&mut writable)?;
            id::UFID_STR
        },
        &FrameData::USER(ref frame) => {
            frame.write(&mut writable)?;
            id::USER_STR
        },
        &FrameData::USLT(ref frame) => {
            frame.write(&mut writable)?;
            id::USLT_STR
        },
        &FrameData::WCOM(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WCOM_STR
        },
        &FrameData::WCOP(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WCOP_STR
        },
        &FrameData::WOAF(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WOAF_STR
        },
        &FrameData::WOAR(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WOAR_STR
        },
        &FrameData::WOAS(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WOAS_STR
        },
        &FrameData::WORS(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WORS_STR
        },
        &FrameData::WPAY(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WPAY_STR
        },
        &FrameData::WPUB(ref frame) => {
            frame.write(&mut writable, version)?;
            id::WPUB_STR
        },
        &FrameData::WXXX(ref frame) => {
            frame.write(&mut writable)?;
            id::WXXX_STR
        },
        _ => ""
    };

    let mut buf = Vec::new();
    writable.as_mut().read_to_end(&mut buf)?;

    Ok((id, buf))
}