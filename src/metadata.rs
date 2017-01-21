extern crate regex;

use bytes;
use readable;
use readable::Readable;

use std::fs::File;
use std::io::Result;
use std::vec::Vec;
use std::iter::Iterator;

pub struct MetadataIterator {
    readable: Readable<File>,
    file_len: u64,
    next: Status,
    version: u8
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

    fn head(&mut self) -> Option<Unit> {
        match self.readable.as_bytes(10) {
            Ok(bytes) => {
                if header::has_flag(header::Flag::ExtendedHeader,
                                    header::version(&bytes),
                                    header::flag(&bytes)) {
                    self.next = Status::ExtendedHeader;
                } else {
                    self.next = Status::Frame;
                }

                Some(Unit::Header(bytes))
            },
            _ => None
        }
    }

    fn extended_head(&mut self) -> Option<Unit> {
        match self.readable.as_bytes(4) {
            Ok(bytes) => {
                let size = match self.version {
                    // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
                    3 => bytes::to_u32(&bytes),
                    // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
                    _ => bytes::to_synchsafe(&bytes),
                };

                match self.readable.as_bytes(size as usize) {
                    Ok(bytes) => {
                        self.next = Status::Frame;
                        Some(Unit::ExtendedHeader(bytes))
                    },
                    _ => {
                        None
                    }
                }
            },
            _ => None
        }
    }

    fn frame(&mut self) -> Option<Unit> {
        // http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
        fn is_valid_frame_id(id: &str) -> bool {
            // TODO const
            let reg = regex::Regex::new(r"^[A-Z][A-Z0-9]{3}$").unwrap();
            reg.is_match(id)
        }

        let is_valid_id = match self.readable.as_string(4) {
            Ok(id) => {
                // rewind
                self.readable.skip(-4);
                let matched = is_valid_frame_id(&id);
                debug!("Frame Id:{}, matched: {}", id, matched);
                matched
            },
            _ => false
        };

        if is_valid_id {
            match self.readable.as_bytes(10) {
                Ok(head_bytes) => {
                    let size = match self.version {
                        3 => bytes::to_u32(&head_bytes[4..8]),
                        _ => bytes::to_synchsafe(&head_bytes[4..8])
                    };

                    match self.readable.as_bytes(size as usize) {
                        Ok(body_bytes) => Some(Unit::FrameV2(head_bytes, body_bytes)),
                        _ => None
                    }
                },
                _ => None
            }
        } else {
            if self.file_len < 128 as u64 {
                return None;
            }

            if let Ok(tag_id) = self.readable.as_string(3) {
                if tag_id != "TAG" {
                    debug!("Ignored v1! {}", tag_id);
                    return None
                }
            }

            match self.readable.position(0) {
                Ok(_) => match self.readable.skip((self.file_len - 128 as u64) as i64) {
                    Ok(_) => match self.readable.all_bytes() {
                        Ok(bytes) => {
                            self.next = Status::None;
                            Some(Unit::FrameV1(bytes))
                        },
                        _ => None
                    },
                    _ => None
                },
                _ => None
            }
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
    Header(Vec<u8>),
    ExtendedHeader(Vec<u8>),
    FrameV2(Vec<u8>, Vec<u8>),
    FrameV1(Vec<u8>)
}

pub mod header {
    use std::io::Result;
    use bytes;

    #[derive(Debug, PartialEq)]
    pub enum Flag {
        Unsynchronisation,
        ExtendedHeader,
        ExperimentalIndicator,
        FooterPresent
    }

    pub struct HeadFrame {
        version: u8,
        minor_version: u8,
        flag: u8,
        size: u32
    }

    impl HeadFrame {
        pub fn get_version(&self) -> u8 {
            self.version
        }

        pub fn get_minor_version(&self) -> u8 {
            self.minor_version
        }

        pub fn has_flag(&self, flag: Flag) -> bool {
            self::has_flag(flag, self.flag, self.version)
        }

        pub fn get_size(&self) -> u32 {
            self.size
        }
    }

    pub struct Head {
        bytes: Vec<u8>
    }

    impl Head {
        pub fn new(bytes: Vec<u8>) -> Self {
            Head {
                bytes: bytes
            }
        }

        pub fn read(&self) -> Result<HeadFrame> {
            let tag_id = String::from_utf8_lossy(&self.bytes[0..3]);
            if tag_id != "ID3" {
                return Err(::std::io::Error::new(::std::io::ErrorKind::Other, format!("Bad v2 tag id: {}", tag_id)));
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

    // see references/id3v2.md#id3v2 Header
    pub fn has_flag(flag: Flag, flag_value: u8, version: u8) -> bool {
        if version == 3 {
            match flag {
                Flag::Unsynchronisation => flag_value & 0x01 << 7 != 0,
                Flag::ExtendedHeader => flag_value & 0x01 << 6 != 0,
                Flag::ExperimentalIndicator => flag_value & 0x01 << 5 != 0,
                _ => false
            }
        } else if version == 4 {
            match flag {
                Flag::Unsynchronisation => flag_value & 0x01 << 7 != 0,
                Flag::ExtendedHeader => flag_value & 0x01 << 6 != 0,
                Flag::ExperimentalIndicator => flag_value & 0x01 << 5 != 0,
                Flag::FooterPresent => flag_value & 0x01 << 4 != 0
            }
        } else {
            warn!("Header.has_flag=> Unknown version!");
            false
        }
    }
}

pub mod frames {
    use std::vec::Vec;
    use std::io::Result;
    use ::frame;
    use ::frame::constants::{id, FrameHeaderFlag, FrameData};
    use ::frame::FrameDataBase;

    #[derive(Debug)]
    pub struct V1Frame {
        title: String,
        artist: String,
        album: String,
        year: String,
        comment: String,
        track: String,
        genre: String
    }

    impl V1Frame {
        pub fn get_title(&self) -> &str {
            self.title.as_ref()
        }

        pub fn get_artist(&self) -> &str {
            self.artist.as_ref()
        }

        pub fn get_album(&self) -> &str {
            self.album.as_ref()
        }

        pub fn get_year(&self) -> &str {
            self.year.as_ref()
        }

        pub fn get_comment(&self) -> &str {
            self.comment.as_ref()
        }

        pub fn get_track(&self) -> &str {
            self.track.as_ref()
        }

        pub fn get_genre(&self) -> &str {
            self.genre.as_ref()
        }
    }

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
            readable.as_bytes(3)?;

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
            let value = String::from_utf8_lossy(&cloned).into_owned();
            value
        }
    }

    pub struct V2 {
        id: String,
        header: Vec<u8>,
        body: Vec<u8>
    }

    impl V2 {
        pub fn new(header: Vec<u8>, body: Vec<u8>) -> Self {
            V2 {
                id: String::from_utf8_lossy(&header[0..4]).into_owned(),
                header: header,
                body: body
            }
        }

        pub fn get_id(&self) -> &str {
            &self.id.as_str()
        }

        // @see http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
        pub fn has_flag(&self, flag: FrameHeaderFlag, version: u8) -> bool {
            let status_flag = self.header[8] & 0x01;
            let encoding_flag = self.header[9] & 0x01;
            match version {
                3 => match flag {
                    FrameHeaderFlag::TagAlter => status_flag << 7 != 0,
                    FrameHeaderFlag::FileAlter => status_flag << 6 != 0,
                    FrameHeaderFlag::ReadOnly => status_flag << 5 != 0,
                    FrameHeaderFlag::Compression => encoding_flag << 7 != 0,
                    FrameHeaderFlag::Encryption => encoding_flag << 6 != 0,
                    FrameHeaderFlag::GroupIdentity => encoding_flag << 5 != 0,
                    _ => false
                },
                4 => match flag {
                    FrameHeaderFlag::TagAlter => status_flag << 6 != 0,
                    FrameHeaderFlag::FileAlter => status_flag << 5 != 0,
                    FrameHeaderFlag::ReadOnly => status_flag << 4 != 0,
                    FrameHeaderFlag::GroupIdentity => encoding_flag << 6 != 0,
                    FrameHeaderFlag::Compression => encoding_flag << 3 != 0,
                    FrameHeaderFlag::Encryption => encoding_flag << 2 != 0,
                    FrameHeaderFlag::Unsynchronisation => encoding_flag << 1 != 0,
                    FrameHeaderFlag::DataLength => encoding_flag != 0
                },
                _ => false
            }
        }

        pub fn read(&self) -> Result<FrameData> {
            let mut readable = ::readable::factory::from_byte(self.body.clone())?;
            let frame_data = match self.get_id() {
                id::AENC_STR => FrameData::AENC(frame::AENC::to_framedata(&mut readable, self.get_id())?),
                id::APIC_STR => FrameData::APIC(frame::APIC::to_framedata(&mut readable, self.get_id())?),
                id::ASPI_STR => FrameData::ASPI(frame::ASPI::to_framedata(&mut readable, self.get_id())?),
                id::COMM_STR => FrameData::COMM(frame::COMM::to_framedata(&mut readable, self.get_id())?),
                id::COMR_STR => FrameData::COMR(frame::COMR::to_framedata(&mut readable, self.get_id())?),
                id::ENCR_STR => FrameData::ENCR(frame::ENCR::to_framedata(&mut readable, self.get_id())?),
                id::EQUA_STR => FrameData::EQUA(frame::EQUA::to_framedata(&mut readable, self.get_id())?),
                id::EQU2_STR => FrameData::EQU2(frame::EQU2::to_framedata(&mut readable, self.get_id())?),
                id::ETCO_STR => FrameData::ETCO(frame::ETCO::to_framedata(&mut readable, self.get_id())?),
                id::GEOB_STR => FrameData::GEOB(frame::GEOB::to_framedata(&mut readable, self.get_id())?),
                id::GRID_STR => FrameData::GRID(frame::GRID::to_framedata(&mut readable, self.get_id())?),
                id::IPLS_STR => FrameData::IPLS(frame::IPLS::to_framedata(&mut readable, self.get_id())?),
                id::LINK_STR => FrameData::LINK(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::MCDI_STR => FrameData::MCDI(frame::MCDI::to_framedata(&mut readable, self.get_id())?),
                id::MLLT_STR => FrameData::MLLT(frame::MLLT::to_framedata(&mut readable, self.get_id())?),
                id::OWNE_STR => FrameData::OWNE(frame::OWNE::to_framedata(&mut readable, self.get_id())?),
                id::PRIV_STR => FrameData::PRIV(frame::PRIV::to_framedata(&mut readable, self.get_id())?),
                id::PCNT_STR => FrameData::PCNT(frame::PCNT::to_framedata(&mut readable, self.get_id())?),
                id::POPM_STR => FrameData::POPM(frame::POPM::to_framedata(&mut readable, self.get_id())?),
                id::POSS_STR => FrameData::POSS(frame::POSS::to_framedata(&mut readable, self.get_id())?),
                id::RBUF_STR => FrameData::RBUF(frame::RBUF::to_framedata(&mut readable, self.get_id())?),
                id::RVAD_STR => FrameData::RVAD(frame::RVA2::to_framedata(&mut readable, self.get_id())?),
                id::RVA2_STR => FrameData::RVA2(frame::RVA2::to_framedata(&mut readable, self.get_id())?),
                id::RVRB_STR => FrameData::RVRB(frame::RVRB::to_framedata(&mut readable, self.get_id())?),
                id::SEEK_STR => FrameData::SEEK(frame::SEEK::to_framedata(&mut readable, self.get_id())?),
                id::SIGN_STR => FrameData::SIGN(frame::SIGN::to_framedata(&mut readable, self.get_id())?),
                id::SYLT_STR => FrameData::SYLT(frame::SYLT::to_framedata(&mut readable, self.get_id())?),
                id::SYTC_STR => FrameData::SYTC(frame::SYTC::to_framedata(&mut readable, self.get_id())?),
                id::UFID_STR => FrameData::UFID(frame::UFID::to_framedata(&mut readable, self.get_id())?),
                id::USER_STR => FrameData::USER(frame::USER::to_framedata(&mut readable, self.get_id())?),
                id::USLT_STR => FrameData::USLT(frame::USLT::to_framedata(&mut readable, self.get_id())?),
                id::TALB_STR => FrameData::TALB(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TBPM_STR => FrameData::TBPM(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TCOM_STR => FrameData::TCOM(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TCON_STR => FrameData::TCON(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TCOP_STR => FrameData::TCOP(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDAT_STR => FrameData::TDAT(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDEN_STR => FrameData::TDEN(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDLY_STR => FrameData::TDLY(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDOR_STR => FrameData::TDOR(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDRC_STR => FrameData::TDRC(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDRL_STR => FrameData::TDRL(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TDTG_STR => FrameData::TDTG(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TENC_STR => FrameData::TENC(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TEXT_STR => FrameData::TEXT(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TIME_STR => FrameData::TIME(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TFLT_STR => FrameData::TFLT(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TIPL_STR => FrameData::TIPL(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TIT1_STR => FrameData::TIT1(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TIT2_STR => FrameData::TIT2(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TIT3_STR => FrameData::TIT3(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TKEY_STR => FrameData::TKEY(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TLAN_STR => FrameData::TLAN(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TLEN_STR => FrameData::TLEN(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TMCL_STR => FrameData::TMCL(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TMED_STR => FrameData::TMED(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TMOO_STR => FrameData::TMOO(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TOAL_STR => FrameData::TOAL(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TOFN_STR => FrameData::TOFN(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TOLY_STR => FrameData::TOLY(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TOPE_STR => FrameData::TOPE(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TORY_STR => FrameData::TORY(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TOWN_STR => FrameData::TOWN(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPE1_STR => FrameData::TPE1(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPE2_STR => FrameData::TPE2(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPE3_STR => FrameData::TPE3(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPE4_STR => FrameData::TPE4(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPOS_STR => FrameData::TPOS(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPRO_STR => FrameData::TPRO(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TPUB_STR => FrameData::TPUB(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TRCK_STR => FrameData::TRCK(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TRDA_STR => FrameData::TRDA(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TRSN_STR => FrameData::TRSN(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSIZ_STR => FrameData::TSIZ(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TRSO_STR => FrameData::TRSO(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSOA_STR => FrameData::TSOA(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSOP_STR => FrameData::TSOP(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSOT_STR => FrameData::TSOT(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSRC_STR => FrameData::TSRC(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSSE_STR => FrameData::TSSE(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TYER_STR => FrameData::TYER(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TSST_STR => FrameData::TSST(frame::TEXT::to_framedata(&mut readable, self.get_id())?),
                id::TXXX_STR => FrameData::TXXX(frame::TXXX::to_framedata(&mut readable, self.get_id())?),
                id::WCOM_STR => FrameData::WCOM(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WCOP_STR => FrameData::WCOP(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WOAF_STR => FrameData::WOAF(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WOAR_STR => FrameData::WOAR(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WOAS_STR => FrameData::WOAS(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WORS_STR => FrameData::WORS(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WPAY_STR => FrameData::WPAY(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WPUB_STR => FrameData::WPUB(frame::LINK::to_framedata(&mut readable, self.get_id())?),
                id::WXXX_STR => FrameData::WXXX(frame::WXXX::to_framedata(&mut readable, self.get_id())?),
                _ => FrameData::TEXT(frame::TEXT::to_framedata(&mut readable, self.get_id())?)
            };

            Ok(frame_data)
        }
    }
}

impl Iterator for MetadataIterator {
    type Item = Unit;

    fn next(&mut self) -> Option<(Self::Item)> {
        debug! ("{:?}", self.next);

        match self.next {
            Status::Head => self.head(),
            Status::ExtendedHeader => self.extended_head(),
            Status::Frame => self.frame(),
            _ => None
        }
    }
}
