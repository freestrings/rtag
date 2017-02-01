extern crate encoding;
pub extern crate regex;
extern crate flate2;

use self::flate2::read::ZlibDecoder;

use frame;
use frame::{
    FrameReaderDefault,
    FrameReaderIdAware,
    FrameReaderVesionAware
};
use frame::constants::{
    id,
    HeadFlag,
    FrameData,
    FrameHeaderFlag
};

use readable;
use readable::Readable;

use std::cell::RefCell;
use std::fs::File;
use std::io::{
    Cursor,
    Error,
    ErrorKind,
    Read,
    Result
};
use std::iter::Iterator;
use std::vec::Vec;
use std::rc::Rc;

const BIT7: u8 = 0x80;
const BIT6: u8 = 0x40;
const BIT5: u8 = 0x20;
const BIT4: u8 = 0x10;
const BIT3: u8 = 0x08;
const BIT2: u8 = 0x04;
const BIT1: u8 = 0x02;
const BIT0: u8 = 0x01;

mod util {
    use super::encoding::{
        Encoding,
        DecoderTrap
    };
    use super::encoding::all::ISO_8859_1;

    pub fn to_synchronize(bytes: &mut Vec<u8>) -> usize {
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

        to
    }

    #[allow(dead_code)]
    pub fn to_hex(bytes: &Vec<u8>) -> String {
        let strs: Vec<String> = bytes.iter()
                                     .map(|b| format!("{:02x}", b))
                                     .collect();
        strs.join(" ")
    }

    pub fn rtrim(bytes: &Vec<u8>) -> Vec<u8> {
        let mut idx = 0;
        for v in bytes.iter().rev() {
            if v > &32 { break; }
            idx = idx + 1;
        }
        let mut clone = bytes.clone();
        clone.split_off(bytes.len() - idx);
        clone
    }

    pub fn string_and_rtrim(bytes: &Vec<u8>) -> String {
        let cloned = rtrim(bytes);
        match ISO_8859_1.decode(&cloned, DecoderTrap::Strict) {
            Ok(value) => value.to_string(),
            _ => "".to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Head {
    pub flag: u8,
    pub version: u8,
    pub minor_version: u8,
    pub size: u32
}

// http://id3.org/id3v2.4.0-structure > 3.1 id3v2 Header
impl Head {
    pub fn new(mut readable: Readable<Cursor<Vec<u8>>>) -> Result<Self> {
        let tag_id = readable.string(3)?;
        let version = readable.u8()?;
        let minor_version = readable.u8()?;
        let flag = readable.u8()?;
        let size = readable.synchsafe()?;

        if tag_id != "ID3" {
            return Err(Error::new(ErrorKind::Other,
                                  format!("Bad v2 tag id: {}", tag_id)));
        }

        Ok(Head {
            version: version,
            minor_version: minor_version,
            flag: flag,
            size: size
        })
    }

    // ./id3v2_summary.md/id3v2.md#id3v2 Header
    //
    // Head level 'Unsynchronisation' does not work on "./test-resources/v2.4-unsync.mp3".
    pub fn has_flag(&self, flag: HeadFlag) -> bool {
        match self.version {
            2 => match flag {
                HeadFlag::Unsynchronisation => self.flag & BIT7 != 0,
                HeadFlag::Compression => self.flag & BIT6 != 0,
                _ => false
            },
            3 => match flag {
                HeadFlag::Unsynchronisation => self.flag & BIT7 != 0,
                HeadFlag::ExtendedHeader => self.flag & BIT6 != 0,
                HeadFlag::ExperimentalIndicator => self.flag & BIT5 != 0,
                _ => false
            },
            4 => match flag {
                //
                // HeadFlag::Unsynchronisation => self.flag & super::BIT7 != 0,
                HeadFlag::ExtendedHeader => self.flag & BIT6 != 0,
                HeadFlag::ExperimentalIndicator => self.flag & BIT5 != 0,
                HeadFlag::FooterPresent => self.flag & BIT4 != 0,
                _ => false
            },
            _ => {
                warn!("Header.has_flag=> Unknown version!");
                false
            }
        }
    }
}


#[derive(Debug)]
pub struct Frame1 {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: String,
    pub genre: String
}

impl Frame1 {
    pub fn new(readable: &mut Readable<Cursor<Vec<u8>>>) -> Result<Self> {
        readable.skip(3)?;

        // offset 3
        let title = util::string_and_rtrim(&readable.bytes(30)?);
        // offset 33
        let artist = util::string_and_rtrim(&readable.bytes(30)?);
        // offset 63
        let album = util::string_and_rtrim(&readable.bytes(30)?);
        // offset 93
        let year = util::string_and_rtrim(&readable.bytes(4)?);
        // goto track marker offset
        readable.skip(28)?;
        // offset 125
        let track_marker = readable.u8()?;
        // offset 126
        let _track = readable.u8()? & 0xff;
        // offset 127
        let genre = (readable.u8()? & 0xff).to_string();
        // goto comment offset
        readable.skip(-31)?;

        let (comment, track) = if track_marker != 0 {
            (
                util::string_and_rtrim(&readable.bytes(30)?),
                String::new()
            )
        } else {
            (
                util::string_and_rtrim(&readable.bytes(28)?),
                if _track == 0 {
                    String::new()
                } else {
                    _track.to_string()
                }
            )
        };

        Ok(Frame1 {
            title: title,
            artist: artist,
            album: album,
            year: year,
            comment: comment,
            track: track,
            genre: genre
        })
    }
}

#[derive(Debug)]
pub struct FrameHeader {
    id: String,
    version: u8,
    status_flag: u8,
    encoding_flag: u8
}

impl FrameHeader {
    pub fn new(id: String, version: u8, status_flag: u8, encoding_flag: u8) -> Self {
        FrameHeader {
            id: id,
            version: version,
            status_flag: status_flag,
            encoding_flag: encoding_flag
        }
    }

    // There is no flag for 2.2 frame.
    // http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
    pub fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        if self.version < 3 {
            return false;
        }

        match self.version {
            3 => match flag {
                FrameHeaderFlag::TagAlter => self.status_flag & BIT7 != 0,
                FrameHeaderFlag::FileAlter => self.status_flag & BIT6 != 0,
                FrameHeaderFlag::ReadOnly => self.status_flag & BIT5 != 0,
                FrameHeaderFlag::Compression => self.encoding_flag & BIT7 != 0,
                FrameHeaderFlag::Encryption => self.encoding_flag & BIT6 != 0,
                FrameHeaderFlag::GroupIdentity => self.encoding_flag & BIT5 != 0,
                _ => false
            },
            4 => match flag {
                FrameHeaderFlag::TagAlter => self.status_flag & BIT6 != 0,
                FrameHeaderFlag::FileAlter => self.status_flag & BIT5 != 0,
                FrameHeaderFlag::ReadOnly => self.status_flag & BIT4 != 0,
                FrameHeaderFlag::GroupIdentity => self.encoding_flag & BIT6 != 0,
                FrameHeaderFlag::Compression => self.encoding_flag & BIT3 != 0,
                FrameHeaderFlag::Encryption => self.encoding_flag & BIT2 != 0,
                FrameHeaderFlag::Unsynchronisation => self.encoding_flag & BIT1 != 0,
                FrameHeaderFlag::DataLength => self.encoding_flag & BIT0 != 0
            },
            _ => false
        }
    }
}

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
    FrameV2(FrameHeader, FrameData)
}

pub struct Metadata {
    next: Status,
    file_len: u64
}

impl Metadata {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_len = metadata.len();
        let readable = readable::factory::from_file(file)?;

        Ok(Metadata {
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

    fn head(&mut self, readable_wrap: RefFileReader) -> Result<Unit> {
        let mut readable = readable_wrap.borrow_mut();

        let head = Head::new(readable.to_readable(10)?)?;
        debug!("{:?}", head);

        let is_extended = head.has_flag(HeadFlag::ExtendedHeader);
        let head_wrap = Rc::new(RefCell::new(Box::new(head.clone())));

        let next = if is_extended {
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
            let frame_readable = readable::factory::from_bytes(frame_bytes)?;
            let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

            Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap)
        };

        self.next = next;

        Ok(Unit::Header(head))
    }

    // optional unit
    fn extended_head(&mut self,
                     head_wrap: RefHead,
                     readable_wrap: RefFileReader) -> Result<Unit> {
        let mut readable = readable_wrap.borrow_mut();

        let size = match head_wrap.borrow().version {
            //
            // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
            3 => readable.u32()?,
            //
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => readable.synchsafe()?
        };
        let extended_bytes = readable.bytes(size as usize)?;
        let head_size = head_wrap.borrow().size as usize;
        let frame_bytes = readable.bytes(head_size)?;
        let frame_readable = readable::factory::from_bytes(frame_bytes)?;
        let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

        self.next = Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap);

        Ok(Unit::ExtendedHeader(extended_bytes))
    }

    fn frame1(&mut self, readable: &mut Readable<File>) -> Result<Frame1> {
        if self.file_len < 128 {
            return Err(Error::new(ErrorKind::InvalidInput,
                                  format!("Invalid file length: {}", self.file_len)));
        }

        readable.skip((self.file_len - 128) as i64)?;

        let tag_id = readable.string(3)?;
        if tag_id != "TAG" {
            return Err(Error::new(ErrorKind::InvalidInput,
                                  format!("Invalid v1 TAG: {}", tag_id)));
        }

        Frame1::new(&mut readable::factory::from_bytes(readable.all_bytes()?)?)
    }

    // version 2.2
    fn frame2(&mut self,
              head: &Head,
              readable: &mut Readable<Cursor<Vec<u8>>>) -> Result<Unit> {
        let id = readable.string(3)?;
        let size = readable.u24()?;
        let frame_header = FrameHeader::new(id.to_string(), head.version, 0, 0);
        let frame_readable = readable.to_readable(size as usize)?;
        let frame_body = frame_data(id.as_str(), head.version, &frame_header, frame_readable)?;

        Ok(Unit::FrameV2(frame_header, frame_body))
    }

    // v2.3
    fn frame3(&mut self, head: &Head, readable: &mut Readable<Cursor<Vec<u8>>>) -> Result<Unit> {
        let id = readable.string(4)?;
        let size = readable.u32()?;
        let status_flag = readable.u8()?;
        let encoding_flag = readable.u8()?;
        let frame_header = FrameHeader::new(id.to_string(), head.version, status_flag, encoding_flag);

        let mut extra_size: u32 = 0;
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            readable.u8()?;
            extra_size = extra_size + 1;
        }

        let body_bytes = if frame_header.has_flag(FrameHeaderFlag::Compression) {
            readable.u32()?;
            extra_size = extra_size + 4;

            let actual_size = size - extra_size as u32;
            let body_bytes = readable.bytes(actual_size as usize)?;
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&body_bytes[..]);

            let _ = decoder.read_to_end(&mut out);

            out
        } else {
            let actual_size = size - extra_size as u32;
            readable.bytes(actual_size as usize)?
        };

        let frame_readable = readable::factory::from_bytes(body_bytes)?;
        let frame_body = frame_data(id.as_str(), head.version, &frame_header, frame_readable)?;

        Ok(Unit::FrameV2(frame_header, frame_body))
    }

    // v2.4
    fn frame4(&mut self,
              head: &Head,
              readable: &mut Readable<Cursor<Vec<u8>>>) -> Result<Unit> {
        let id = readable.string(4)?;
        let size = readable.synchsafe()?;
        let status_flag = readable.u8()?;
        let encoding_flag = readable.u8()?;
        let frame_header = FrameHeader::new(id.to_string(), head.version, status_flag, encoding_flag);


        let mut extra_size: u32 = 0;
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::DataLength) {
            readable.u32()?;
            extra_size = extra_size + 4;
        }

        let actual_size = size - extra_size as u32;
        let mut body_bytes = readable.bytes(actual_size as usize)?;

        if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
            debug!("'{}' is unsynchronised", id);

            let mut out = body_bytes[..].to_vec();
            let sync_size = util::to_synchronize(&mut out);

            //cut to synchrosized size
            out.split_off(sync_size);

            body_bytes = out;
        }

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("'{}' is compressed", id);

            readable.u32()?;

            let real_frame = body_bytes.clone();
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&real_frame[..]);
            let _ = decoder.read_to_end(&mut out);

            body_bytes = out;
        }

        let frame_readable = readable::factory::from_bytes(body_bytes)?;
        let frame_body = frame_data(id.as_str(), head.version, &frame_header, frame_readable)?;

        Ok(Unit::FrameV2(frame_header, frame_body))
    }

    fn frame(&mut self,
             head_wrap: RefHead,
             readable_wrap: RefFileReader,
             frame_readable_wrap: RefByteReader) -> Result<Unit> {
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

impl Iterator for Metadata {
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
                    trace!("Head ignored: {}", msg);
                    None
                }
            },
            Status::ExtendedHeader(_, _) => {
                let (head, readable) = extended_header.unwrap();
                match self.extended_head(head, readable) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        trace!("Extended head ignored: {}", msg);
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
                        trace!("Frame ignored: {}", msg);
                        None
                    }
                }
            }
            _ => None
        }
    }
}

fn frame_data(id: &str,
              version: u8,
              frame_header: &FrameHeader,
              mut readable: Readable<Cursor<Vec<u8>>>) -> Result<FrameData> {
    if frame_header.has_flag(FrameHeaderFlag::Encryption) {
        return Ok(FrameData::SKIP("Encrypted frame".to_string()));
    };

    let frame_data = match id.as_ref() {
        id::BUF_STR => FrameData::BUF(frame::BUF::read(&mut readable)?),
        id::CNT_STR => FrameData::PCNT(frame::PCNT::read(&mut readable)?),
        id::COM_STR => FrameData::COMM(frame::COMM::read(&mut readable)?),
        id::CRA_STR => FrameData::AENC(frame::AENC::read(&mut readable)?),
        id::CRM_STR => FrameData::CRM(frame::CRM::read(&mut readable)?),
        id::ETC_STR => FrameData::ETCO(frame::ETCO::read(&mut readable)?),
        id::EQU_STR => FrameData::EQUA(frame::EQUA::read(&mut readable)?),
        id::GEO_STR => FrameData::GEOB(frame::GEOB::read(&mut readable)?),
        id::IPL_STR => FrameData::IPLS(frame::IPLS::read(&mut readable)?),
        id::LNK_STR => FrameData::LINK(frame::LINK::read(&mut readable, version)?),
        id::MCI_STR => FrameData::MCDI(frame::MCDI::read(&mut readable)?),
        id::MLL_STR => FrameData::MLLT(frame::MLLT::read(&mut readable)?),
        id::PIC_STR => FrameData::PIC(frame::PIC::read(&mut readable)?),
        id::POP_STR => FrameData::POPM(frame::POPM::read(&mut readable)?),
        id::REV_STR => FrameData::RVRB(frame::RVRB::read(&mut readable)?),
        id::RVA_STR => FrameData::RVAD(frame::RVA2::read(&mut readable)?),
        id::SLT_STR => FrameData::SYLT(frame::SYLT::read(&mut readable)?),
        id::STC_STR => FrameData::SYTC(frame::SYTC::read(&mut readable)?),
        id::TAL_STR => FrameData::TALB(frame::TEXT::read(&mut readable, id)?),
        id::TBP_STR => FrameData::TBPM(frame::TEXT::read(&mut readable, id)?),
        id::TCM_STR => FrameData::TCOM(frame::TEXT::read(&mut readable, id)?),
        id::TCO_STR => FrameData::TCON(frame::TEXT::read(&mut readable, id)?),
        id::TCR_STR => FrameData::TCOP(frame::TEXT::read(&mut readable, id)?),
        id::TDA_STR => FrameData::TDAT(frame::TEXT::read(&mut readable, id)?),
        id::TDY_STR => FrameData::TDLY(frame::TEXT::read(&mut readable, id)?),
        id::TEN_STR => FrameData::TENC(frame::TEXT::read(&mut readable, id)?),
        id::TFT_STR => FrameData::TFLT(frame::TEXT::read(&mut readable, id)?),
        id::TIM_STR => FrameData::TIME(frame::TEXT::read(&mut readable, id)?),
        id::TKE_STR => FrameData::TKEY(frame::TEXT::read(&mut readable, id)?),
        id::TLA_STR => FrameData::TLAN(frame::TEXT::read(&mut readable, id)?),
        id::TLE_STR => FrameData::TLEN(frame::TEXT::read(&mut readable, id)?),
        id::TMT_STR => FrameData::TMED(frame::TEXT::read(&mut readable, id)?),
        id::TOA_STR => FrameData::TMED(frame::TEXT::read(&mut readable, id)?),
        id::TOF_STR => FrameData::TOFN(frame::TEXT::read(&mut readable, id)?),
        id::TOL_STR => FrameData::TOLY(frame::TEXT::read(&mut readable, id)?),
        id::TOR_STR => FrameData::TORY(frame::TEXT::read(&mut readable, id)?),
        id::TOT_STR => FrameData::TOAL(frame::TEXT::read(&mut readable, id)?),
        id::TP1_STR => FrameData::TPE1(frame::TEXT::read(&mut readable, id)?),
        id::TP2_STR => FrameData::TPE2(frame::TEXT::read(&mut readable, id)?),
        id::TP3_STR => FrameData::TPE3(frame::TEXT::read(&mut readable, id)?),
        id::TP4_STR => FrameData::TPE4(frame::TEXT::read(&mut readable, id)?),
        id::TPA_STR => FrameData::TPOS(frame::TEXT::read(&mut readable, id)?),
        id::TPB_STR => FrameData::TPUB(frame::TEXT::read(&mut readable, id)?),
        id::TRC_STR => FrameData::TSRC(frame::TEXT::read(&mut readable, id)?),
        id::TRD_STR => FrameData::TRDA(frame::TEXT::read(&mut readable, id)?),
        id::TRK_STR => FrameData::TRCK(frame::TEXT::read(&mut readable, id)?),
        id::TSI_STR => FrameData::TSIZ(frame::TEXT::read(&mut readable, id)?),
        id::TSS_STR => FrameData::TSSE(frame::TEXT::read(&mut readable, id)?),
        id::TT1_STR => FrameData::TIT1(frame::TEXT::read(&mut readable, id)?),
        id::TT2_STR => FrameData::TIT2(frame::TEXT::read(&mut readable, id)?),
        id::TT3_STR => FrameData::TIT3(frame::TEXT::read(&mut readable, id)?),
        id::TXT_STR => FrameData::TEXT(frame::TEXT::read(&mut readable, id)?),
        id::TXX_STR => FrameData::TXXX(frame::TXXX::read(&mut readable)?),
        id::TYE_STR => FrameData::TYER(frame::TEXT::read(&mut readable, id)?),
        id::UFI_STR => FrameData::UFID(frame::UFID::read(&mut readable)?),
        id::ULT_STR => FrameData::USLT(frame::USLT::read(&mut readable)?),
        id::WAF_STR => FrameData::WOAF(frame::LINK::read(&mut readable, version)?),
        id::WAR_STR => FrameData::WOAR(frame::LINK::read(&mut readable, version)?),
        id::WAS_STR => FrameData::WOAS(frame::LINK::read(&mut readable, version)?),
        id::WCM_STR => FrameData::WCOM(frame::LINK::read(&mut readable, version)?),
        id::WCP_STR => FrameData::WCOP(frame::LINK::read(&mut readable, version)?),
        id::WPB_STR => FrameData::WPUB(frame::LINK::read(&mut readable, version)?),
        id::WXX_STR => FrameData::WXXX(frame::WXXX::read(&mut readable)?),
        id::AENC_STR => FrameData::AENC(frame::AENC::read(&mut readable)?),
        id::APIC_STR => FrameData::APIC(frame::APIC::read(&mut readable)?),
        id::ASPI_STR => FrameData::ASPI(frame::ASPI::read(&mut readable)?),
        id::COMM_STR => FrameData::COMM(frame::COMM::read(&mut readable)?),
        id::COMR_STR => FrameData::COMR(frame::COMR::read(&mut readable)?),
        id::ENCR_STR => FrameData::ENCR(frame::ENCR::read(&mut readable)?),
        id::EQUA_STR => FrameData::EQUA(frame::EQUA::read(&mut readable)?),
        id::EQU2_STR => FrameData::EQU2(frame::EQU2::read(&mut readable)?),
        id::ETCO_STR => FrameData::ETCO(frame::ETCO::read(&mut readable)?),
        id::GEOB_STR => FrameData::GEOB(frame::GEOB::read(&mut readable)?),
        id::GRID_STR => FrameData::GRID(frame::GRID::read(&mut readable)?),
        id::IPLS_STR => FrameData::IPLS(frame::IPLS::read(&mut readable)?),
        id::LINK_STR => FrameData::LINK(frame::LINK::read(&mut readable, version)?),
        id::MCDI_STR => FrameData::MCDI(frame::MCDI::read(&mut readable)?),
        id::MLLT_STR => FrameData::MLLT(frame::MLLT::read(&mut readable)?),
        id::OWNE_STR => FrameData::OWNE(frame::OWNE::read(&mut readable)?),
        id::PRIV_STR => FrameData::PRIV(frame::PRIV::read(&mut readable)?),
        id::PCNT_STR => FrameData::PCNT(frame::PCNT::read(&mut readable)?),
        id::POPM_STR => FrameData::POPM(frame::POPM::read(&mut readable)?),
        id::POSS_STR => FrameData::POSS(frame::POSS::read(&mut readable)?),
        id::RBUF_STR => FrameData::RBUF(frame::RBUF::read(&mut readable)?),
        id::RVAD_STR => FrameData::RVAD(frame::RVA2::read(&mut readable)?),
        id::RVA2_STR => FrameData::RVA2(frame::RVA2::read(&mut readable)?),
        id::RVRB_STR => FrameData::RVRB(frame::RVRB::read(&mut readable)?),
        id::SEEK_STR => FrameData::SEEK(frame::SEEK::read(&mut readable)?),
        id::SIGN_STR => FrameData::SIGN(frame::SIGN::read(&mut readable)?),
        id::SYLT_STR => FrameData::SYLT(frame::SYLT::read(&mut readable)?),
        id::SYTC_STR => FrameData::SYTC(frame::SYTC::read(&mut readable)?),
        id::UFID_STR => FrameData::UFID(frame::UFID::read(&mut readable)?),
        id::USER_STR => FrameData::USER(frame::USER::read(&mut readable)?),
        id::USLT_STR => FrameData::USLT(frame::USLT::read(&mut readable)?),
        id::TALB_STR => FrameData::TALB(frame::TEXT::read(&mut readable, id)?),
        id::TBPM_STR => FrameData::TBPM(frame::TEXT::read(&mut readable, id)?),
        id::TCOM_STR => FrameData::TCOM(frame::TEXT::read(&mut readable, id)?),
        id::TCON_STR => FrameData::TCON(frame::TEXT::read(&mut readable, id)?),
        id::TCOP_STR => FrameData::TCOP(frame::TEXT::read(&mut readable, id)?),
        id::TDAT_STR => FrameData::TDAT(frame::TEXT::read(&mut readable, id)?),
        id::TDEN_STR => FrameData::TDEN(frame::TEXT::read(&mut readable, id)?),
        id::TDLY_STR => FrameData::TDLY(frame::TEXT::read(&mut readable, id)?),
        id::TDOR_STR => FrameData::TDOR(frame::TEXT::read(&mut readable, id)?),
        id::TDRC_STR => FrameData::TDRC(frame::TEXT::read(&mut readable, id)?),
        id::TDRL_STR => FrameData::TDRL(frame::TEXT::read(&mut readable, id)?),
        id::TDTG_STR => FrameData::TDTG(frame::TEXT::read(&mut readable, id)?),
        id::TENC_STR => FrameData::TENC(frame::TEXT::read(&mut readable, id)?),
        id::TEXT_STR => FrameData::TEXT(frame::TEXT::read(&mut readable, id)?),
        id::TIME_STR => FrameData::TIME(frame::TEXT::read(&mut readable, id)?),
        id::TFLT_STR => FrameData::TFLT(frame::TEXT::read(&mut readable, id)?),
        id::TIPL_STR => FrameData::TIPL(frame::TEXT::read(&mut readable, id)?),
        id::TIT1_STR => FrameData::TIT1(frame::TEXT::read(&mut readable, id)?),
        id::TIT2_STR => FrameData::TIT2(frame::TEXT::read(&mut readable, id)?),
        id::TIT3_STR => FrameData::TIT3(frame::TEXT::read(&mut readable, id)?),
        id::TKEY_STR => FrameData::TKEY(frame::TEXT::read(&mut readable, id)?),
        id::TLAN_STR => FrameData::TLAN(frame::TEXT::read(&mut readable, id)?),
        id::TLEN_STR => FrameData::TLEN(frame::TEXT::read(&mut readable, id)?),
        id::TMCL_STR => FrameData::TMCL(frame::TEXT::read(&mut readable, id)?),
        id::TMED_STR => FrameData::TMED(frame::TEXT::read(&mut readable, id)?),
        id::TMOO_STR => FrameData::TMOO(frame::TEXT::read(&mut readable, id)?),
        id::TOAL_STR => FrameData::TOAL(frame::TEXT::read(&mut readable, id)?),
        id::TOFN_STR => FrameData::TOFN(frame::TEXT::read(&mut readable, id)?),
        id::TOLY_STR => FrameData::TOLY(frame::TEXT::read(&mut readable, id)?),
        id::TOPE_STR => FrameData::TOPE(frame::TEXT::read(&mut readable, id)?),
        id::TORY_STR => FrameData::TORY(frame::TEXT::read(&mut readable, id)?),
        id::TOWN_STR => FrameData::TOWN(frame::TEXT::read(&mut readable, id)?),
        id::TPE1_STR => FrameData::TPE1(frame::TEXT::read(&mut readable, id)?),
        id::TPE2_STR => FrameData::TPE2(frame::TEXT::read(&mut readable, id)?),
        id::TPE3_STR => FrameData::TPE3(frame::TEXT::read(&mut readable, id)?),
        id::TPE4_STR => FrameData::TPE4(frame::TEXT::read(&mut readable, id)?),
        id::TPOS_STR => FrameData::TPOS(frame::TEXT::read(&mut readable, id)?),
        id::TPRO_STR => FrameData::TPRO(frame::TEXT::read(&mut readable, id)?),
        id::TPUB_STR => FrameData::TPUB(frame::TEXT::read(&mut readable, id)?),
        id::TRCK_STR => FrameData::TRCK(frame::TEXT::read(&mut readable, id)?),
        id::TRDA_STR => FrameData::TRDA(frame::TEXT::read(&mut readable, id)?),
        id::TRSN_STR => FrameData::TRSN(frame::TEXT::read(&mut readable, id)?),
        id::TSIZ_STR => FrameData::TSIZ(frame::TEXT::read(&mut readable, id)?),
        id::TRSO_STR => FrameData::TRSO(frame::TEXT::read(&mut readable, id)?),
        id::TSOA_STR => FrameData::TSOA(frame::TEXT::read(&mut readable, id)?),
        id::TSOP_STR => FrameData::TSOP(frame::TEXT::read(&mut readable, id)?),
        id::TSOT_STR => FrameData::TSOT(frame::TEXT::read(&mut readable, id)?),
        id::TSRC_STR => FrameData::TSRC(frame::TEXT::read(&mut readable, id)?),
        id::TSSE_STR => FrameData::TSSE(frame::TEXT::read(&mut readable, id)?),
        id::TYER_STR => FrameData::TYER(frame::TEXT::read(&mut readable, id)?),
        id::TSST_STR => FrameData::TSST(frame::TEXT::read(&mut readable, id)?),
        id::TXXX_STR => FrameData::TXXX(frame::TXXX::read(&mut readable)?),
        id::WCOM_STR => FrameData::WCOM(frame::LINK::read(&mut readable, version)?),
        id::WCOP_STR => FrameData::WCOP(frame::LINK::read(&mut readable, version)?),
        id::WOAF_STR => FrameData::WOAF(frame::LINK::read(&mut readable, version)?),
        id::WOAR_STR => FrameData::WOAR(frame::LINK::read(&mut readable, version)?),
        id::WOAS_STR => FrameData::WOAS(frame::LINK::read(&mut readable, version)?),
        id::WORS_STR => FrameData::WORS(frame::LINK::read(&mut readable, version)?),
        id::WPAY_STR => FrameData::WPAY(frame::LINK::read(&mut readable, version)?),
        id::WPUB_STR => FrameData::WPUB(frame::LINK::read(&mut readable, version)?),
        id::WXXX_STR => FrameData::WXXX(frame::WXXX::read(&mut readable)?),
        _ => {
            warn!("No frame id found!! '{}'", id);
            FrameData::TEXT(frame::TEXT::read(&mut readable, id)?)
        }
    };

    Ok(frame_data)
}