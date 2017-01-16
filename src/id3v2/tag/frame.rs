//MIT License
//
//Copyright (c) [2017] [Mark Han]
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

extern crate encoding;
extern crate regex;

use self::encoding::{Encoding, DecoderTrap};
use std::{borrow, collections, ops, io, vec};
use std::io::Result;

const AENC_STR: &'static str = "AENC";
const APIC_STR: &'static str = "APIC";
const ASPI_STR: &'static str = "ASPI";
const COMM_STR: &'static str = "COMM";
const COMR_STR: &'static str = "COMR";
const ENCR_STR: &'static str = "ENCR";
const EQU2_STR: &'static str = "EQU2";
const ETCO_STR: &'static str = "ETCO";
const GEOB_STR: &'static str = "GEOB";
const GRID_STR: &'static str = "GRID";
const LINK_STR: &'static str = "LINK";
const MCDI_STR: &'static str = "MCDI";
const MLLT_STR: &'static str = "MLLT";
const OWNE_STR: &'static str = "OWNE";
const PRIV_STR: &'static str = "PRIV";
const PCNT_STR: &'static str = "PCNT";
const POPM_STR: &'static str = "POPM";
const POSS_STR: &'static str = "POSS";
const RBUF_STR: &'static str = "RBUF";
const RVA2_STR: &'static str = "RVA2";
const RVRB_STR: &'static str = "RVRB";
const SEEK_STR: &'static str = "SEEK";
const SIGN_STR: &'static str = "SIGN";
const SYLT_STR: &'static str = "SYLT";
const SYTC_STR: &'static str = "SYTC";
const TALB_STR: &'static str = "TALB";
const TBPM_STR: &'static str = "TBPM";
const TCOM_STR: &'static str = "TCOM";
const TCON_STR: &'static str = "TCON";
const TCOP_STR: &'static str = "TCOP";
const TDEN_STR: &'static str = "TDEN";
const TDLY_STR: &'static str = "TDLY";
const TDOR_STR: &'static str = "TDOR";
const TDRC_STR: &'static str = "TDRC";
const TDRL_STR: &'static str = "TDRL";
const TDTG_STR: &'static str = "TDTG";
const TENC_STR: &'static str = "TENC";
const TEXT_STR: &'static str = "TEXT";
const TFLT_STR: &'static str = "TFLT";
const TIPL_STR: &'static str = "TIPL";
const TIT1_STR: &'static str = "TIT1";
const TIT2_STR: &'static str = "TIT2";
const TIT3_STR: &'static str = "TIT3";
const TKEY_STR: &'static str = "TKEY";
const TLAN_STR: &'static str = "TLAN";
const TLEN_STR: &'static str = "TLEN";
const TMCL_STR: &'static str = "TMCL";
const TMED_STR: &'static str = "TMED";
const TMOO_STR: &'static str = "TMOO";
const TOAL_STR: &'static str = "TOAL";
const TOFN_STR: &'static str = "TOFN";
const TOLY_STR: &'static str = "TOLY";
const TOPE_STR: &'static str = "TOPE";
const TOWN_STR: &'static str = "TOWN";
const TPE1_STR: &'static str = "TPE1";
const TPE2_STR: &'static str = "TPE2";
const TPE3_STR: &'static str = "TPE3";
const TPE4_STR: &'static str = "TPE4";
const TPOS_STR: &'static str = "TPOS";
const TPRO_STR: &'static str = "TPRO";
const TPUB_STR: &'static str = "TPUB";
const TRCK_STR: &'static str = "TRCK";
const TRSN_STR: &'static str = "TRSN";
const TRSO_STR: &'static str = "TRSO";
const TSOA_STR: &'static str = "TSOA";
const TSOP_STR: &'static str = "TSOP";
const TSOT_STR: &'static str = "TSOT";
const TSRC_STR: &'static str = "TSRC";
const TSSE_STR: &'static str = "TSSE";
const TSST_STR: &'static str = "TSST";
const TXXX_STR: &'static str = "TXXX";
const UFID_STR: &'static str = "UFID";
const USER_STR: &'static str = "USER";
const USLT_STR: &'static str = "USLT";
const WCOM_STR: &'static str = "WCOM";
const WCOP_STR: &'static str = "WCOP";
const WOAF_STR: &'static str = "WOAF";
const WOAR_STR: &'static str = "WOAR";
const WOAS_STR: &'static str = "WOAS";
const WORS_STR: &'static str = "WORS";
const WPAY_STR: &'static str = "WPAY";
const WPUB_STR: &'static str = "WPUB";
const WXXX_STR: &'static str = "WXXX";

#[derive(Debug)]
pub enum FrameData {
    AENC(AENC),
    APIC(APIC),
    ASPI(ASPI),
    COMM(COMM),
    COMR(COMR),
    ENCR(ENCR),
    EQU2(EQU2),
    ETCO(ETCO),
    GEOB(GEOB),
    GRID(GRID),
    LINK(LINK),
    MCDI(MCDI),
    MLLT(MLLT),
    OWNE(OWNE),
    PRIV(PRIV),
    PCNT(PCNT),
    POPM(POPM),
    POSS(POSS),
    RBUF(RBUF),
    RVA2(RVA2),
    RVRB(RVRB),
    SEEK(SEEK),
    SIGN(SIGN),
    SYLT(SYLT),
    SYTC(SYTC),
    TALB(TEXT),
    TBPM(TEXT),
    TCOM(TEXT),
    TCON(TEXT),
    TCOP(TEXT),
    TDEN(TEXT),
    TDLY(TEXT),
    TDOR(TEXT),
    TDRC(TEXT),
    TDRL(TEXT),
    TDTG(TEXT),
    TENC(TEXT),
    TEXT(TEXT),
    TFLT(TEXT),
    TIPL(TEXT),
    TIT1(TEXT),
    TIT2(TEXT),
    TIT3(TEXT),
    TKEY(TEXT),
    TLAN(TEXT),
    TLEN(TEXT),
    TMCL(TEXT),
    TMED(TEXT),
    TMOO(TEXT),
    TOAL(TEXT),
    TOFN(TEXT),
    TOLY(TEXT),
    TOPE(TEXT),
    TOWN(TEXT),
    TPE1(TEXT),
    TPE2(TEXT),
    TPE3(TEXT),
    TPE4(TEXT),
    TPOS(TEXT),
    TPRO(TEXT),
    TPUB(TEXT),
    TRCK(TEXT),
    TRSO(TEXT),
    TSOA(TEXT),
    TSOP(TEXT),
    TSOT(TEXT),
    TSRC(TEXT),
    TSSE(TEXT),
    TSST(TEXT),
    TXXX(TEXT),
    UFID(UFID),
    USER(USER),
    USLT(USLT),
    WXXX(WXXX)
}

#[derive(Debug)]
enum PictureType {
    Other,
    FileIcon,
    OtherFileIcon,
    CoverFront,
    CoverBack,
    LeafletPage,
    Media,
    LeadArtist,
    Artist,
    Conductor,
    Band,
    Composer,
    Lyricist,
    RecordingLocation,
    DuringRecording,
    DuringPerformance,
    MovieScreenCapture,
    BrightColouredFish,
    Illustration,
    BandLogotype,
    PublisherLogoType
}

#[derive(Debug)]
enum ReceivedAs {
    Other,
    StandardCDAlbum,
    CompressedAudioOnCD,
    FileOverInternet,
    StreamOverInternet,
    AsNoteSheets,
    AsNoteSheetsInBook,
    MusicOnMedia,
    NonMusicalMerchandise
}

#[derive(Debug)]
enum ContentType {
    Other,
    Lyrics,
    TextTranscription,
    MovementName,
    Events,
    Chord,
    Trivia,
    UrlsToWebpages,
    UrlsToImages
}

#[derive(Debug)]
enum TimeStampFormat {
    MpecFrames,
    Milliseconds
}

enum FrameHeaderFlag {
    TagAlter,
    FileAlter,
    ReadOnly,
    Compression,
    Encryption,
    GroupIdentity,
    //2.4 only
    Unsynchronisation,
    //2.4 only
    DataLength
}

fn to_picture_type(t: u8) -> PictureType {
    match t {
        0x00 => PictureType::Other,
        0x01 => PictureType::FileIcon,
        0x02 => PictureType::OtherFileIcon,
        0x03 => PictureType::CoverFront,
        0x04 => PictureType::CoverBack,
        0x05 => PictureType::LeafletPage,
        0x06 => PictureType::Media,
        0x07 => PictureType::LeadArtist,
        0x08 => PictureType::Artist,
        0x09 => PictureType::Conductor,
        0x0A => PictureType::Band,
        0x0B => PictureType::Composer,
        0x0C => PictureType::Lyricist,
        0x0D => PictureType::RecordingLocation,
        0x0E => PictureType::DuringRecording,
        0x0F => PictureType::DuringPerformance,
        0x10 => PictureType::MovieScreenCapture,
        0x11 => PictureType::BrightColouredFish,
        0x12 => PictureType::Illustration,
        0x13 => PictureType::BandLogotype,
        0x14 => PictureType::PublisherLogoType,
        _ => PictureType::Other
    }
}

fn encoded_text(text_encoding: &::id3v2::bytes::TextEncoding, readable: &mut Readable) -> Result<(usize, String)> {
    Ok(match text_encoding {
        &::id3v2::bytes::TextEncoding::ISO8859_1 | &::id3v2::bytes::TextEncoding::UTF8 => readable.read_non_utf16_string()?,
        _ => readable.read_utf16_string()?
    })
}

type Readable = ::readable::Readable<io::Cursor<vec::Vec<u8>>>;

trait FrameDataBase {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData>;
}

//Audio encryption
#[derive(Debug)]
struct AENC {
    owner_identifier: String,
    preview_start: u16,
    preview_end: u16,
    encryption_info: vec::Vec<u8>
}

impl FrameDataBase for AENC {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData> {
        let (read, id) = readable.read_non_utf16_string()?;
        Ok(self::FrameData::AENC(AENC {
            owner_identifier: id,
            preview_start: ::id3v2::bytes::to_u16(&readable.as_bytes(2)?),
            preview_end: ::id3v2::bytes::to_u16(&readable.as_bytes(2)?),
            encryption_info: readable.all_bytes()?
        }))
    }
}

//Attached picture
#[derive(Debug)]
struct APIC {
    text_encoding: ::id3v2::bytes::TextEncoding,
    mime_type: String,
    picture_type: PictureType,
    description: String,
    picture_data: vec::Vec<u8>
}

impl FrameDataBase for APIC {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (read, mine_type) = readable.read_non_utf16_string()?;
        let picture_type = to_picture_type(readable.as_bytes(1)?[0]);
        let (_, description) = encoded_text(&text_encoding, readable)?;
        let picture_data = readable.all_bytes()?;
        Ok(self::FrameData::APIC(APIC {
            text_encoding: text_encoding,
            mime_type: mine_type,
            picture_type: picture_type,
            description: description,
            picture_data: picture_data
        }))
    }
}

// Audio seek point index
#[derive(Debug)]
struct ASPI {
    indexed_data_start: u32,
    indexed_data_length: u32,
    number_of_index_points: u16,
    bit_per_index_point: u8,
    fraction_at_index: u8
}

// Comments
#[derive(Debug)]
struct COMM {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    short_description: String,
    actual_text: String
}

impl COMM {

    pub fn get_text_encoding(&self) -> &::id3v2::bytes::TextEncoding {
        &self.text_encoding
    }

    pub fn get_language(&self) -> &str {
        self.language.as_str()
    }

    pub fn get_short_description(&self) -> &str {
        self.short_description.as_str()
    }

    pub fn get_actual_text(&self) -> &str {
        self.actual_text.as_str()
    }
}

impl FrameDataBase for COMM {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, short_description) = encoded_text(&text_encoding, readable)?;
        let actual_text = readable.all_string()?;
        Ok(self::FrameData::COMM(COMM {
            text_encoding: text_encoding,
            language: language,
            short_description: short_description,
            actual_text: actual_text
        }))
    }
}

// Commercial frame
#[derive(Debug)]
struct COMR {
    text_encoding: ::id3v2::bytes::TextEncoding,
    price_string: String,
    // 8 bit long
    valid_util: String,
    contat_url: String,
    received_as: ReceivedAs,
    name_of_seller: String,
    description: String,
    picture_mime_type: String,
    seller_logo: vec::Vec<u8>
}

// Encryption method registration
#[derive(Debug)]
struct ENCR {
    owner_identifier: String,
    method_symbol: u8,
    encryption_data: vec::Vec<u8>
}

// Equalisation (2)
#[derive(Debug)]
struct EQU2 {
    interpolation_method: u8,
    identification: String
}

// Event timing codes
#[derive(Debug)]
struct ETCO {
    time_stamp_format: TimeStampFormat
}

// General encapsulated object
#[derive(Debug)]
struct GEOB {
    text_encoding: ::id3v2::bytes::TextEncoding,
    mine_type: String,
    filename: String,
    content_description: String,
    encapsulation_object: vec::Vec<u8>
}

// Group identification registration
#[derive(Debug)]
struct GRID {
    owner_identifier: String,
    group_symbol: u8,
    group_dependent_data: vec::Vec<u8>
}

// Linked information
#[derive(Debug)]
struct LINK {
    frame_identifier: u32,
    url: String,
    additional_data: String
}

impl FrameDataBase for LINK {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData> {
        let frame_id = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let (_, url) = readable.read_non_utf16_string()?;
        let additional_data = readable.all_string()?;
        Ok(self::FrameData::LINK(LINK {
            frame_identifier: frame_id,
            url: url,
            additional_data: additional_data
        }))
    }
}

// Music CD identifier
#[derive(Debug)]
struct MCDI {
    cd_toc: vec::Vec<u8>
}

// MPEG location lookup table
// TODO
#[derive(Debug)]
struct MLLT {
    data: vec::Vec<u8>
}

// Ownership frame
#[derive(Debug)]
struct OWNE {
    text_encoding: ::id3v2::bytes::TextEncoding,
    price_paid: String,
    // 8 bit long
    date_of_purch: String,
    seller: String
}

// Private frame
#[derive(Debug)]
struct PRIV {
    owner_identifier: String,
    private_data: vec::Vec<u8>
}

// Play counter
#[derive(Debug)]
struct PCNT {
    counter: u32
}

// Popularimeter
#[derive(Debug)]
struct POPM {
    email_to_user: String,
    rating: u8,
    counter: u32
}

// Position synchronisation frame
#[derive(Debug)]
struct POSS {
    time_stamp_format: TimeStampFormat,
    position: u8
}

// Recommended buffer size
#[derive(Debug)]
struct RBUF {
    buffer_size: u32,
    embedded_info_flag: u8,
    offset_to_next_tag: u32
}

// Relative volume adjustment (2)
#[derive(Debug)]
struct RVA2 {
    identification: String
}

// Reverb
#[derive(Debug)]
struct RVRB {
    reverb_left: u16,
    reverb_right: u16,
    reverb_bounce_left: u8,
    reverb_bounce_right: u8,
    reverb_feedback_left_to_left: u8,
    reverb_feedback_left_to_right: u8,
    reverb_feedback_right_to_right: u8,
    reverb_feedback_right_to_left: u8,
    premix_left_to_right: u8,
    premix_right_to_left: u8
}

// Seek frame
#[derive(Debug)]
struct SEEK {
    next_tag: String
}

// Signature frame
#[derive(Debug)]
struct SIGN {
    group_symbol: u8,
    signature: vec::Vec<u8>
}

// Synchronised lyric/text
#[derive(Debug)]
struct SYLT {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    time_stamp_format: TimeStampFormat,
    content_type: ContentType,
    content_descriptor: String
}

// Synchronised tempo codes
#[derive(Debug)]
struct SYTC {
    time_stamp_format: TimeStampFormat,
    tempo_data: vec::Vec<u8>
}

// Unique file identifier
#[derive(Debug)]
struct UFID {
    owner_identifier: String,
    identifier: vec::Vec<u8>
}

// Terms of use
#[derive(Debug)]
struct USER {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    actual_text: String
}

// Unsynchronised lyric/text transcription
#[derive(Debug)]
struct USLT {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    content_descriptor: String,
    lyrics: String
}

#[derive(Debug)]
struct TEXT {
    text_encoding: ::id3v2::bytes::TextEncoding,
    text: String
}

impl TEXT {

    pub fn get_text_encoding(&self) -> &::id3v2::bytes::TextEncoding {
        &self.text_encoding
    }

    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }
}

impl FrameDataBase for TEXT {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let data = readable.all_bytes()?;
        let text = match text_encoding {
            ::id3v2::bytes::TextEncoding::ISO8859_1 => encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?,

            ::id3v2::bytes::TextEncoding::UTF16LE => encoding::all::UTF_16LE.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?,

            ::id3v2::bytes::TextEncoding::UTF16BE => encoding::all::UTF_16BE.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?,

            ::id3v2::bytes::TextEncoding::UTF8 => encoding::all::UTF_8.decode(&data, encoding::DecoderTrap::Strict)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?
        };

        Ok(self::FrameData::TEXT(TEXT {
            text_encoding: text_encoding,
            text: text
        }))
    }
}

// User defined URL link frame
#[derive(Debug)]
struct WXXX {
    text_encoding: ::id3v2::bytes::TextEncoding,
    description: String,
    url: String
}

impl FrameDataBase for WXXX {
    fn to_framedata(readable: &mut Readable) -> Result<FrameData> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = encoded_text(&text_encoding, readable)?;
        let url = readable.all_string()?;
        Ok(self::FrameData::WXXX(WXXX {
            text_encoding: text_encoding,
            description: description,
            url: url
        }))
    }
}

pub struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8
}

impl Frame {
    fn _frame_id(bytes: &vec::Vec<u8>) -> String {
        String::from_utf8_lossy(&bytes[0..4]).into_owned()
    }

    fn _frame_size(bytes: &vec::Vec<u8>, tag_version: u8) -> u32 {
        match tag_version {
            3 => ::id3v2::bytes::to_u32(&bytes[4..8]),
            _ => ::id3v2::bytes::to_synchsafe(&bytes[4..8])
        }
    }

    pub fn has_next_frame<T>(readable: &mut ::readable::Readable<T>) -> bool
        where T: io::Read + io::Seek {
        // read frame id 4 bytes
        match readable.as_string(4) {
            Ok(id) => {
                // rewind
                readable.skip(-4);
                // @see http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let re = regex::Regex::new(r"^[A-Z][A-Z0-9]{3}$").unwrap();
                let matched = re.is_match(&id);
                debug!("Frame.has_next_frame=> Frame Id:{}, matched: {}", id, matched);
                matched
            },
            Err(_) => {
                debug!("Frame.has_next_frame=> Fail");
                false
            }
        }
    }

    pub fn new<T>(readable: &mut ::readable::Readable<T>, tag_version: u8) -> Result<Frame>
        where T: io::Read + io::Seek {
        // head 10 bytes
        let header_bytes = readable.as_bytes(10)?;
        let id = Self::_frame_id(&header_bytes);
        let frame_size = Self::_frame_size(&header_bytes, tag_version);
        let body_bytes = readable.as_bytes(frame_size as usize)?;

        debug!("Frame.new=> frame size: {}", frame_size);
        if frame_size == 0 {
            warn!("Frame.new: frame size is 0!");
        }

        Ok(Frame {
            id: id,
            size: frame_size,
            data: body_bytes,
            // status_flag offset is 8
            status_flag: header_bytes[8],
            // encoding_flag offset is 9
            encoding_flag: header_bytes[9]
        })
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    // @see http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
    pub fn has_flag(&self, flag: FrameHeaderFlag, major_version: u8) -> bool {
        if major_version == 3 {
            match flag {
                FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 7 != 0,
                FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 5 != 0,
                FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 7 != 0,
                FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 5 != 0,
                _ => false
            }
        } else if major_version == 4 {
            match flag {
                FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 5 != 0,
                FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 4 != 0,
                FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 6 != 0,
                FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 3 != 0,
                FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 2 != 0,
                FrameHeaderFlag::Unsynchronisation => self.encoding_flag & 0x01 << 1 != 0,
                FrameHeaderFlag::DataLength => self.encoding_flag & 0x01 != 0
            }
        } else {
            warn!("Frame.has_flag=> Unknown version!");
            false
        }
    }

    // @see http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
    pub fn get_data(&self) -> Result<FrameData> {
        let mut readable = ::readable::factory::from_byte(self.data.clone())?;
        if let Some('T') = self.id.chars().next() {
            return TEXT::to_framedata(&mut readable);
        }

        if let Some('W') = self.id.chars().next() {
            if self.id != WXXX_STR {
                return LINK::to_framedata(&mut readable);
            }
        }

        match self.id.as_ref() {
            self::AENC_STR => AENC::to_framedata(&mut readable),
            self::APIC_STR => APIC::to_framedata(&mut readable),
            self::COMM_STR => COMM::to_framedata(&mut readable),
            self::WXXX_STR => WXXX::to_framedata(&mut readable),
            _ => self::TEXT::to_framedata(&mut readable)
        }
    }
}
