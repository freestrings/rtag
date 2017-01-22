pub extern crate regex;

use bytes;
use readable;
use readable::Readable;

use std::fs::File;
use std::io::Result;
use std::vec::Vec;
use std::iter::Iterator;

pub struct MetadataIterator {
    readable: Readable<File>,
    pub file_len: u64,
    next: Status,
    pub version: u8
}

impl MetadataIterator {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_len = metadata.len();
        let readable = readable::factory::from_file(file)?;

        Ok(MetadataIterator {
            readable: readable,
            file_len: file_len,
            next: Status::Head,
            version: 0
        })
    }

    fn head(&mut self) -> Result<Unit> {
        let bytes = self.readable.as_bytes(10)?;
        let flag = header::flag(&bytes);
        let version = header::version(&bytes);
        if header::has_flag(header::Flag::ExtendedHeader, flag, version) {
            self.next = Status::ExtendedHeader;
        } else {
            self.next = Status::Frame;
        }
        self.version = version;

        Ok(Unit::Header(header::Head::new(bytes)))
    }

    fn extended_head(&mut self) -> Result<Unit> {
        self.next = Status::Frame;

        let bytes = self.readable.as_bytes(4)?;
        let size = match self.version {
            // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
            3 => bytes::to_u32(&bytes),
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => bytes::to_synchsafe(&bytes),
        };

        Ok(Unit::ExtendedHeader(self.readable.as_bytes(size as usize)?))
    }

    fn frame(&mut self) -> Result<Unit> {
        let is_valid_id = match self.readable.as_string(4) {
            Ok(id) => {
                self.readable.skip(-4);
                // TODO const
                // http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let matched = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}").unwrap().is_match(&id);
                debug!("Frame Id:{}, matched: {}", id, matched);
                matched
            },
            _ => false
        };

        if is_valid_id {
            // frame v2
            if self.version == 2 {
                let id = self.readable.as_string(3)?;
                let head_bytes = self.readable.as_bytes(3)?;
                let size = bytes::to_u32(&head_bytes[0..3]);
                let body_bytes = self.readable.as_bytes(size as usize)?;
                Ok(Unit::FrameV2(frames::V2::new(id, head_bytes, body_bytes, self.version)))
            } else {
                let id = self.readable.as_string(4)?;
                let head_bytes = self.readable.as_bytes(6)?;
                let size = match self.version {
                    // v2.3 read as a regular big endian number.
                    3 => bytes::to_u32(&head_bytes[0..4]),
                    // v2.4 uses sync-safe frame sizes
                    _ => bytes::to_synchsafe(&head_bytes[0..4])
                };
                let body_bytes = self.readable.as_bytes(size as usize)?;

                Ok(Unit::FrameV2(frames::V2::new(id, head_bytes, body_bytes, self.version)))
            }
        } else {
            // frame v1

            if self.file_len < 128 as u64 {
                return Err(::std::io::Error::new(::std::io::ErrorKind::Other, "Bad v2 tag length"));
            }

            self.readable.position(0)?;
            self.readable.skip((self.file_len - 128 as u64) as i64)?;
            if self.readable.as_string(3)? != "TAG" {
                return Err(::std::io::Error::new(::std::io::ErrorKind::Other, "Ignored v1! '{}'"));
            }
            self.readable.skip(-3)?;
            let bytes = self.readable.all_bytes()?;
            self.next = Status::None;

            Ok(Unit::FrameV1(frames::V1::new(bytes)))
        }
    }
}

#[derive(Debug)]
enum Status {
    Head,
    ExtendedHeader,
    Frame,
    None
}

#[derive(Debug)]
pub enum Unit {
    Header(header::Head),
    // TODO not yet implemented
    ExtendedHeader(Vec<u8>),
    FrameV2(frames::V2),
    FrameV1(frames::V1)
}

pub mod header {
    use std::io::Result;
    use bytes;

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

    impl HeadFrame {
        pub fn has_flag(&self, flag: Flag) -> bool {
            self::has_flag(flag, self.flag, self.version)
        }
    }

    #[derive(Debug)]
    pub struct Head {
        bytes: Vec<u8>
    }

    // http://id3.org/id3v2.4.0-structure > 3.1 id3v2 Header
    impl Head {
        pub fn new(bytes: Vec<u8>) -> Self {
            Head {
                bytes: bytes
            }
        }

        pub fn read(&self) -> Result<HeadFrame> {
            let tag_id = String::from_utf8_lossy(&self.bytes[0..3]);
            if tag_id != "ID3" {
                return Err(::std::io::Error::new(::std::io::ErrorKind::Other,
                                                 format!("Bad v2 tag id: {}", tag_id)));
            }

            Ok(HeadFrame {
                version: self::version(&self.bytes),
                minor_version: self.bytes[4],
                flag: self::flag(&self.bytes),
                size: bytes::to_synchsafe(&self.bytes[6..10])
            })
        }
    }

    pub fn version(bytes: &Vec<u8>) -> u8 {
        bytes[3]
    }

    pub fn flag(bytes: &Vec<u8>) -> u8 {
        bytes[5]
    }

    // ./references/id3v2.md#id3v2 Header
    pub fn has_flag(flag: Flag, flag_value: u8, version: u8) -> bool {
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
    use ::frame::FrameDefault;

    #[derive(Debug)]
    pub struct V1Frame {
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

        pub fn read(&self) -> Result<V1Frame> {
            let mut readable = ::readable::factory::from_byte(self.bytes.clone())?;

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

            Ok(V1Frame {
                title: title,
                artist: artist,
                album: album,
                year: year,
                comment: comment,
                track: track,
                genre: genre
            })
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

        pub fn read(&self) -> Result<FrameData> {
            let mut readable = if self.has_flag(FrameHeaderFlag::Compression) {
                debug!("{} is compressed", self.id);
                // skip 4 bytes that is decompressed size.
                let real_frame = self.body.clone().split_off(4);
                let mut decoder = ZlibDecoder::new(&real_frame[..]);
                let mut out = vec![];
                decoder.read_to_end(&mut out)?;
                ::readable::factory::from_byte(out)?
            } else {
                ::readable::factory::from_byte(self.body[..].to_vec())?
            };

            let id = self.id.as_str();

            let frame_data = match self.id.as_ref() {
                id::BUF_STR => FrameData::BUF(frame::BUF::read(&mut readable, id)?),
                id::CNT_STR => FrameData::PCNT(frame::PCNT::read(&mut readable, id)?),
                id::COM_STR => FrameData::COMM(frame::COMM::read(&mut readable, id)?),
                id::CRA_STR => FrameData::AENC(frame::AENC::read(&mut readable, id)?),
                id::CRM_STR => FrameData::CRM(frame::CRM::read(&mut readable, id)?),
                id::ETC_STR => FrameData::ETCO(frame::ETCO::read(&mut readable, id)?),
                id::EQU_STR => FrameData::EQUA(frame::EQUA::read(&mut readable, id)?),
                id::GEO_STR => FrameData::GEOB(frame::GEOB::read(&mut readable, id)?),
                id::IPL_STR => FrameData::IPLS(frame::IPLS::read(&mut readable, id)?),
                id::LNK_STR => FrameData::LINK(frame::LINK::read(&mut readable, id)?),
                id::MCI_STR => FrameData::MCDI(frame::MCDI::read(&mut readable, id)?),
                id::MLL_STR => FrameData::MLLT(frame::MLLT::read(&mut readable, id)?),
                id::PIC_STR => FrameData::PIC(frame::PIC::read(&mut readable, id)?),
                id::POP_STR => FrameData::POPM(frame::POPM::read(&mut readable, id)?),
                id::REV_STR => FrameData::RVRB(frame::RVRB::read(&mut readable, id)?),
                id::RVA_STR => FrameData::RVAD(frame::RVA2::read(&mut readable, id)?),
                id::SLT_STR => FrameData::SYLT(frame::SYLT::read(&mut readable, id)?),
                id::STC_STR => FrameData::SYTC(frame::SYTC::read(&mut readable, id)?),
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
                id::TXX_STR => FrameData::TXXX(frame::TXXX::read(&mut readable, id)?),
                id::TYE_STR => FrameData::TYER(frame::TEXT::read(&mut readable, id)?),
                id::UFI_STR => FrameData::UFID(frame::UFID::read(&mut readable, id)?),
                id::ULT_STR => FrameData::USLT(frame::USLT::read(&mut readable, id)?),
                id::WAF_STR => FrameData::WOAF(frame::LINK::read(&mut readable, id)?),
                id::WAR_STR => FrameData::WOAR(frame::LINK::read(&mut readable, id)?),
                id::WAS_STR => FrameData::WOAS(frame::LINK::read(&mut readable, id)?),
                id::WCM_STR => FrameData::WCOM(frame::LINK::read(&mut readable, id)?),
                id::WCP_STR => FrameData::WCOP(frame::LINK::read(&mut readable, id)?),
                id::WPB_STR => FrameData::WPUB(frame::LINK::read(&mut readable, id)?),
                id::WXX_STR => FrameData::WXXX(frame::WXXX::read(&mut readable, id)?),
                id::AENC_STR => FrameData::AENC(frame::AENC::read(&mut readable, id)?),
                id::APIC_STR => FrameData::APIC(frame::APIC::read(&mut readable, id)?),
                id::ASPI_STR => FrameData::ASPI(frame::ASPI::read(&mut readable, id)?),
                id::COMM_STR => FrameData::COMM(frame::COMM::read(&mut readable, id)?),
                id::COMR_STR => FrameData::COMR(frame::COMR::read(&mut readable, id)?),
                id::ENCR_STR => FrameData::ENCR(frame::ENCR::read(&mut readable, id)?),
                id::EQUA_STR => FrameData::EQUA(frame::EQUA::read(&mut readable, id)?),
                id::EQU2_STR => FrameData::EQU2(frame::EQU2::read(&mut readable, id)?),
                id::ETCO_STR => FrameData::ETCO(frame::ETCO::read(&mut readable, id)?),
                id::GEOB_STR => FrameData::GEOB(frame::GEOB::read(&mut readable, id)?),
                id::GRID_STR => FrameData::GRID(frame::GRID::read(&mut readable, id)?),
                id::IPLS_STR => FrameData::IPLS(frame::IPLS::read(&mut readable, id)?),
                id::LINK_STR => FrameData::LINK(frame::LINK::read(&mut readable, id)?),
                id::MCDI_STR => FrameData::MCDI(frame::MCDI::read(&mut readable, id)?),
                id::MLLT_STR => FrameData::MLLT(frame::MLLT::read(&mut readable, id)?),
                id::OWNE_STR => FrameData::OWNE(frame::OWNE::read(&mut readable, id)?),
                id::PRIV_STR => FrameData::PRIV(frame::PRIV::read(&mut readable, id)?),
                id::PCNT_STR => FrameData::PCNT(frame::PCNT::read(&mut readable, id)?),
                id::POPM_STR => FrameData::POPM(frame::POPM::read(&mut readable, id)?),
                id::POSS_STR => FrameData::POSS(frame::POSS::read(&mut readable, id)?),
                id::RBUF_STR => FrameData::RBUF(frame::RBUF::read(&mut readable, id)?),
                id::RVAD_STR => FrameData::RVAD(frame::RVA2::read(&mut readable, id)?),
                id::RVA2_STR => FrameData::RVA2(frame::RVA2::read(&mut readable, id)?),
                id::RVRB_STR => FrameData::RVRB(frame::RVRB::read(&mut readable, id)?),
                id::SEEK_STR => FrameData::SEEK(frame::SEEK::read(&mut readable, id)?),
                id::SIGN_STR => FrameData::SIGN(frame::SIGN::read(&mut readable, id)?),
                id::SYLT_STR => FrameData::SYLT(frame::SYLT::read(&mut readable, id)?),
                id::SYTC_STR => FrameData::SYTC(frame::SYTC::read(&mut readable, id)?),
                id::UFID_STR => FrameData::UFID(frame::UFID::read(&mut readable, id)?),
                id::USER_STR => FrameData::USER(frame::USER::read(&mut readable, id)?),
                id::USLT_STR => FrameData::USLT(frame::USLT::read(&mut readable, id)?),
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
                id::TXXX_STR => FrameData::TXXX(frame::TXXX::read(&mut readable, id)?),
                id::WCOM_STR => FrameData::WCOM(frame::LINK::read(&mut readable, id)?),
                id::WCOP_STR => FrameData::WCOP(frame::LINK::read(&mut readable, id)?),
                id::WOAF_STR => FrameData::WOAF(frame::LINK::read(&mut readable, id)?),
                id::WOAR_STR => FrameData::WOAR(frame::LINK::read(&mut readable, id)?),
                id::WOAS_STR => FrameData::WOAS(frame::LINK::read(&mut readable, id)?),
                id::WORS_STR => FrameData::WORS(frame::LINK::read(&mut readable, id)?),
                id::WPAY_STR => FrameData::WPAY(frame::LINK::read(&mut readable, id)?),
                id::WPUB_STR => FrameData::WPUB(frame::LINK::read(&mut readable, id)?),
                id::WXXX_STR => FrameData::WXXX(frame::WXXX::read(&mut readable, id)?),
                _ => {
                    warn!("No frame id found!! '{}'", id);
                    FrameData::TEXT(frame::TEXT::read(&mut readable, id)?)
                }
            };

            Ok(frame_data)
        }
    }
}

impl Iterator for MetadataIterator {
    type Item = Unit;

    fn next(&mut self) -> Option<(Self::Item)> {
        debug! ("next: {:?}", self.next);

        match self.next {
            Status::Head => match self.head() {
                Ok(data) => Some(data),
                _ => None
            },
            Status::ExtendedHeader => match self.extended_head() {
                Ok(data) => Some(data),
                _ => None
            },
            Status::Frame => match self.frame() {
                Ok(data) => Some(data),
                _ => None
            },
            _ => None
        }
    }
}
