pub extern crate regex;

use bytes;
use readable;
use readable::Readable;

use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, Error, ErrorKind, Result};
use std::iter::Iterator;
use std::vec::Vec;
use std::rc::Rc;

use frame::constants::{FrameData, FrameHeaderFlag};
use self::frames::FrameHeader;

type FrameReadable = Rc<RefCell<Readable<Cursor<Vec<u8>>>>>;

#[derive(Debug)]
enum Status {
    Head,
    ExtendedHeader(u32),
    Frame(FrameReadable),
    None
}


#[derive(Debug)]
pub enum Unit {
    Header(header::HeadFrame),
    // TODO not yet implemented
    ExtendedHeader(Vec<u8>),
    FrameV2(FrameHeader, FrameData),
    FrameV1(frames::FrameV1)
}

pub struct Metadata {
    pub version: u8,
    next: Status,
    readable: Readable<File>,
    frame_v1: Option<Vec<u8>>,
    file_len: u64
}

impl Metadata {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_len = metadata.len();
        let mut readable = readable::factory::from_file(file)?;

        Ok(Metadata {
            version: 0,
            next: Status::Head,
            readable: readable,
            frame_v1: None,
            file_len: file_len
        })
    }

    fn create_rc_readable(bytes: Vec<u8>) -> Result<FrameReadable> {
        Ok(Rc::new(RefCell::new(readable::factory::from_bytes(bytes)?)))
    }

    fn read_v1(&mut self) -> Result<Vec<u8>> {
        if self.file_len < 128 {
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid file length: {}", self.file_len)));
        }

        self.readable.skip((self.file_len - 128) as i64)?;
        let tag_id = self.readable.as_string(3)?;
        if tag_id != "TAG" {
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid v1 TAG: {}", tag_id)));
        }

        Ok(self.readable.all_bytes()?)
    }

    fn head(&mut self) -> Result<Unit> {
        // keep all the v1 info.
        self.frame_v1 = match self.read_v1() {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                trace!("{:?}", e);
                None
            }
        };
        self.readable.position(0)?;

        let head = header::Head::new(self.readable.as_bytes(10)?);
        let header = head.read()?;
        let next = if head.has_flag(header::Flag::ExtendedHeader) {
            Status::ExtendedHeader(header.size)
        } else if head.has_flag(header::Flag::Unsynchronisation) {
            match self.readable.as_bytes(header.size as usize) {
                Ok(mut frame_bytes) => {
                    bytes::to_synchronize(&mut frame_bytes);
                    Status::Frame(Self::create_rc_readable(frame_bytes)?)
                },
                _ => Status::Frame(Self::create_rc_readable(Vec::new())?)
            }
        } else {
            match self.readable.as_bytes(header.size as usize) {
                Ok(frame_bytes) => Status::Frame(Self::create_rc_readable(frame_bytes)?),
                _ => Status::Frame(Self::create_rc_readable(Vec::new())?)
            }
        };

        self.next = next;
        self.version = head.version;

        Ok(Unit::Header(header))
    }

    fn extended_head(&mut self, frame_size: u32) -> Result<Unit> {
        let bytes = self.readable.as_bytes(4)?;
        let size = match self.version {
            // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
            3 => bytes::to_u32(&bytes),
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => bytes::to_synchsafe(&bytes)
        };

        self.next = match self.readable.as_bytes(frame_size as usize) {
            Ok(frame_bytes) => Status::Frame(Self::create_rc_readable(frame_bytes)?),
            _ => Status::Frame(Self::create_rc_readable(Vec::new())?)
        };

        Ok(Unit::ExtendedHeader(self.readable.as_bytes(size as usize)?))
    }

    fn frame(&mut self, mut _readable: FrameReadable) -> Result<Unit> {
        let mut readable = _readable.borrow_mut();

        let has_frame_v2 = match readable.as_string(4) {
            Ok(id) => {
                readable.skip(-4);
                // TODO const
                // http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let matched = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}").unwrap().is_match(&id);
                debug!("Frame Id:{}, matched: {}", id, matched);
                matched
            },
            _ => false
        };

        if has_frame_v2 {
            // frame v2

            let id = match self.version {
                2 => readable.as_string(3)?,
                _ => readable.as_string(4)?
            };

            let head_bytes = match self.version {
                2 => readable.as_bytes(3)?,
                _ => readable.as_bytes(6)?
            };

            let size = match self.version {
                2 => bytes::to_u32(&head_bytes[0..3]),
                // v2.3 read as a regular big endian number.
                3 => bytes::to_u32(&head_bytes[0..4]),
                // v2.4 uses sync-safe frame sizes
                _ => bytes::to_synchsafe(&head_bytes[0..4])
            };

            let body_bytes = readable.as_bytes(size as usize)?;

            let (frame_header, frame_body) = frames::V2::new(id,
                                                             head_bytes,
                                                             body_bytes,
                                                             self.version).read()?;

            Ok(Unit::FrameV2(frame_header, frame_body))
        } else {
            self.next = Status::None;

            match self.frame_v1 {
                Some(ref bytes) => Ok(Unit::FrameV1(frames::V1::new(bytes[..].to_vec()).read()?)),
                None => Err(Error::new(ErrorKind::Other, "Frame v1"))
            }
        }
    }
}

pub trait MetaFrame<T> {
    fn read(&self) -> Result<T>;
}

impl Iterator for Metadata {
    type Item = Unit;

    fn next(&mut self) -> Option<(Self::Item)> {
        match self.next {
            Status::Head => debug!("next: Head"),
            Status::ExtendedHeader(_) => debug!("next: ExtendedHeader"),
            Status::Frame(_) => debug!("next: Frame"),
            Status::None => debug!("next: None"),
        };

        let mut frame_readable = if let Status::Frame(ref rc) = self.next {
            Some(rc.clone())
        } else {
            None
        };

        match self.next {
            Status::Head => match self.head() {
                Ok(data) => Some(data),
                Err(msg) => {
                    trace!("Head ignored: {}", msg);
                    None
                }
            },
            Status::ExtendedHeader(frame_size) => match self.extended_head(frame_size) {
                Ok(data) => Some(data),
                Err(msg) => {
                    trace!("Extended head ignored: {}", msg);
                    None
                }
            },
            Status::Frame(_) => {
                match self.frame(frame_readable.unwrap()) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        trace!("Frame ignored: {}", msg);
                        None
                    }
                }
            },
            _ => None
        }
    }
}

pub mod header {
    use std::io::Result;

    use bytes;
    use super::MetaFrame;

    #[derive(Debug, PartialEq)]
    pub enum Flag {
        Unsynchronisation,
        Compression,
        ExtendedHeader,
        ExperimentalIndicator,
        FooterPresent
    }

    #[derive(Debug)]
    pub struct HeadFrame {
        pub version: u8,
        pub minor_version: u8,
        pub flag: u8,
        pub size: u32
    }

    // ./id3v2_summary.md/id3v2.md#id3v2 Header
    fn has_flag(flag: Flag, flag_value: u8, version: u8) -> bool {
        if version == 3 {
            match flag {
                Flag::Unsynchronisation => flag_value & bytes::BIT7 != 0,
                Flag::ExtendedHeader => flag_value & bytes::BIT6 != 0,
                Flag::ExperimentalIndicator => flag_value & bytes::BIT5 != 0,
                _ => false
            }
        } else if version == 4 {
            match flag {
                Flag::Unsynchronisation => flag_value & bytes::BIT7 != 0,
                Flag::ExtendedHeader => flag_value & bytes::BIT6 != 0,
                Flag::ExperimentalIndicator => flag_value & bytes::BIT5 != 0,
                Flag::FooterPresent => flag_value & bytes::BIT4 != 0,
                _ => false
            }
        } else if version == 2 {
            match flag {
                Flag::Unsynchronisation => flag_value & bytes::BIT7 != 0,
                Flag::Compression => flag_value & bytes::BIT6 != 0,
                _ => false
            }
        } else {
            warn!("Header.has_flag=> Unknown version!");
            false
        }
    }

    impl HeadFrame {
        pub fn has_flag(&self, flag: Flag) -> bool {
            self::has_flag(flag, self.flag, self.version)
        }
    }

    #[derive(Debug)]
    pub struct Head {
        bytes: Vec<u8>,
        pub flag: u8,
        pub version: u8
    }

    // http://id3.org/id3v2.4.0-structure > 3.1 id3v2 Header
    impl Head {
        pub fn new(bytes: Vec<u8>) -> Self {
            let flag = bytes[5];
            let version = bytes[3];
            Head {
                bytes: bytes,
                flag: flag,
                version: version
            }
        }

        pub fn has_flag(&self, flag: Flag) -> bool {
            self::has_flag(flag, self.flag, self.version)
        }
    }

    impl MetaFrame<HeadFrame> for Head {
        fn read(&self) -> Result<HeadFrame> {
            let tag_id = String::from_utf8_lossy(&self.bytes[0..3]);
            if tag_id != "ID3" {
                return Err(::std::io::Error::new(::std::io::ErrorKind::Other,
                                                 format!("Bad v2 tag id: {}", tag_id)));
            }

            Ok(HeadFrame {
                version: self.version,
                minor_version: self.bytes[4],
                flag: self.flag,
                size: bytes::to_synchsafe(&self.bytes[6..10])
            })
        }
    }
}

pub mod frames {
    extern crate encoding;
    extern crate flate2;

    use self::flate2::read::ZlibDecoder;

    use self::encoding::{Encoding, DecoderTrap};

    use std::vec::Vec;
    use std::io::{Read, Result};
    use bytes;
    use ::frame;
    use ::frame::constants::{id, FrameHeaderFlag, FrameData};
    use super::MetaFrame;
    use ::frame::{FrameReaderDefault, FrameReaderIdAware, FrameReaderVesionAware};

    #[derive(Debug)]
    pub struct FrameV1 {
        pub title: String,
        pub artist: String,
        pub album: String,
        pub year: String,
        pub comment: String,
        pub track: String,
        pub genre: String
    }

    #[derive(Debug)]
    pub struct V1 {
        bytes: Vec<u8>
    }

    impl V1 {
        pub fn new(bytes: Vec<u8>) -> Self {
            V1 {
                bytes: bytes
            }
        }

        fn rtrim(bytes: &Vec<u8>) -> Vec<u8> {
            let mut idx = 0;
            for v in bytes.iter().rev() {
                if v > &32 { break; }
                idx = idx + 1;
            }
            let mut clone = bytes.clone();
            clone.split_off(bytes.len() - idx);
            clone
        }

        fn to_string_with_rtrim(bytes: &Vec<u8>) -> String {
            let cloned = Self::rtrim(bytes);
            match encoding::all::ISO_8859_1.decode(&cloned, encoding::DecoderTrap::Strict) {
                Ok(value) => value.to_string(),
                _ => "".to_string()
            }
        }
    }

    impl MetaFrame<FrameV1> for V1 {
        fn read(&self) -> Result<FrameV1> {
            let mut readable = ::readable::factory::from_bytes(self.bytes.clone())?;

            // skip id
            readable.skip(3)?;

            // offset 3
            let title = Self::to_string_with_rtrim(&readable.as_bytes(30)?);
            // offset 33
            let artist = Self::to_string_with_rtrim(&readable.as_bytes(30)?);
            // offset 63
            let album = Self::to_string_with_rtrim(&readable.as_bytes(30)?);
            // offset 93
            let year = Self::to_string_with_rtrim(&readable.as_bytes(4)?);
            // goto track marker offset
            readable.skip(28)?;
            // offset 125
            let track_marker = readable.as_bytes(1)?[0];
            // offset 126
            let _track = readable.as_bytes(1)?[0] & 0xff;
            // offset 127
            let genre = (readable.as_bytes(1)?[0] & 0xff).to_string();
            // goto comment offset
            readable.skip(-31)?;

            let (comment, track) = if track_marker != 0 {
                (
                    Self::to_string_with_rtrim(&readable.as_bytes(30)?),
                    String::new()
                )
            } else {
                (
                    Self::to_string_with_rtrim(&readable.as_bytes(28)?),
                    if _track == 0 { String::new() } else { _track.to_string() }
                )
            };

            Ok(FrameV1 {
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
        header: Vec<u8>,
        version: u8
    }

    impl FrameHeader {
        pub fn new(header: Vec<u8>, version: u8) -> Self {
            FrameHeader {
                header: header,
                version: version
            }
        }
        // There is no flag for 2.2 frame.
        // http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
        pub fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
            if self.version < 3 {
                return false;
            }

            let status_flag = self.header[4];
            let encoding_flag = self.header[5];
            match self.version {
                3 => match flag {
                    FrameHeaderFlag::TagAlter => status_flag & bytes::BIT7 != 0,
                    FrameHeaderFlag::FileAlter => status_flag & bytes::BIT6 != 0,
                    FrameHeaderFlag::ReadOnly => status_flag & bytes::BIT5 != 0,
                    FrameHeaderFlag::Compression => encoding_flag & bytes::BIT7 != 0,
                    FrameHeaderFlag::Encryption => encoding_flag & bytes::BIT6 != 0,
                    FrameHeaderFlag::GroupIdentity => encoding_flag & bytes::BIT5 != 0,
                    _ => false
                },
                4 => match flag {
                    FrameHeaderFlag::TagAlter => status_flag & bytes::BIT6 != 0,
                    FrameHeaderFlag::FileAlter => status_flag & bytes::BIT5 != 0,
                    FrameHeaderFlag::ReadOnly => status_flag & bytes::BIT4 != 0,
                    FrameHeaderFlag::GroupIdentity => encoding_flag & bytes::BIT6 != 0,
                    FrameHeaderFlag::Compression => encoding_flag & bytes::BIT3 != 0,
                    FrameHeaderFlag::Encryption => encoding_flag & bytes::BIT2 != 0,
                    FrameHeaderFlag::Unsynchronisation => encoding_flag & bytes::BIT1 != 0,
                    FrameHeaderFlag::DataLength => encoding_flag & bytes::BIT0 != 0
                },
                _ => false
            }
        }
    }

    #[derive(Debug)]
    pub struct V2 {
        pub id: String,
        header: Vec<u8>,
        body: Vec<u8>,
        version: u8
    }

    impl V2 {
        pub fn new(id: String, header: Vec<u8>, body: Vec<u8>, version: u8) -> Self {
            V2 {
                id: id,
                header: header,
                body: body,
                version: version
            }
        }
    }

    impl MetaFrame<(FrameHeader, FrameData)> for V2 {
        fn read(&self) -> Result<(FrameHeader, FrameData)> {
            let frame_header = FrameHeader::new(self.header[..].to_vec(), self.version);

            let mut readable = if frame_header.has_flag(FrameHeaderFlag::Compression) {
                debug!("{} is compressed", self.id);
                // skip 4 bytes that is decompressed size.
                let real_frame = self.body.clone().split_off(4);
                let mut decoder = ZlibDecoder::new(&real_frame[..]);
                let mut out = vec![];
                decoder.read_to_end(&mut out)?;

                ::readable::factory::from_bytes(out)?
            } else if frame_header.has_flag(FrameHeaderFlag::Encryption) {
                return Ok((frame_header, FrameData::SKIP("Encrypted frame".to_string())));
            } else {
                ::readable::factory::from_bytes(self.body[..].to_vec())?
            };

            let id = self.id.as_str();

            let frame_data = match self.id.as_ref() {
                id::BUF_STR => FrameData::BUF(frame::BUF::read(&mut readable)?),
                id::CNT_STR => FrameData::PCNT(frame::PCNT::read(&mut readable)?),
                id::COM_STR => FrameData::COMM(frame::COMM::read(&mut readable)?),
                id::CRA_STR => FrameData::AENC(frame::AENC::read(&mut readable)?),
                id::CRM_STR => FrameData::CRM(frame::CRM::read(&mut readable)?),
                id::ETC_STR => FrameData::ETCO(frame::ETCO::read(&mut readable)?),
                id::EQU_STR => FrameData::EQUA(frame::EQUA::read(&mut readable)?),
                id::GEO_STR => FrameData::GEOB(frame::GEOB::read(&mut readable)?),
                id::IPL_STR => FrameData::IPLS(frame::IPLS::read(&mut readable)?),
                id::LNK_STR => FrameData::LINK(frame::LINK::read(&mut readable, self.version)?),
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
                id::WAF_STR => FrameData::WOAF(frame::LINK::read(&mut readable, self.version)?),
                id::WAR_STR => FrameData::WOAR(frame::LINK::read(&mut readable, self.version)?),
                id::WAS_STR => FrameData::WOAS(frame::LINK::read(&mut readable, self.version)?),
                id::WCM_STR => FrameData::WCOM(frame::LINK::read(&mut readable, self.version)?),
                id::WCP_STR => FrameData::WCOP(frame::LINK::read(&mut readable, self.version)?),
                id::WPB_STR => FrameData::WPUB(frame::LINK::read(&mut readable, self.version)?),
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
                id::LINK_STR => FrameData::LINK(frame::LINK::read(&mut readable, self.version)?),
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
                id::WCOM_STR => FrameData::WCOM(frame::LINK::read(&mut readable, self.version)?),
                id::WCOP_STR => FrameData::WCOP(frame::LINK::read(&mut readable, self.version)?),
                id::WOAF_STR => FrameData::WOAF(frame::LINK::read(&mut readable, self.version)?),
                id::WOAR_STR => FrameData::WOAR(frame::LINK::read(&mut readable, self.version)?),
                id::WOAS_STR => FrameData::WOAS(frame::LINK::read(&mut readable, self.version)?),
                id::WORS_STR => FrameData::WORS(frame::LINK::read(&mut readable, self.version)?),
                id::WPAY_STR => FrameData::WPAY(frame::LINK::read(&mut readable, self.version)?),
                id::WPUB_STR => FrameData::WPUB(frame::LINK::read(&mut readable, self.version)?),
                id::WXXX_STR => FrameData::WXXX(frame::WXXX::read(&mut readable)?),
                _ => {
                    warn!("No frame id found!! '{}'", id);
                    FrameData::TEXT(frame::TEXT::read(&mut readable, id)?)
                }
            };

            Ok((frame_header, frame_data))
        }
    }
}