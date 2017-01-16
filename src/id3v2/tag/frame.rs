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

fn to_picture_type(t: u8) -> ::id3v2::tag::frame_constants::PictureType {
    match t {
        0x00 => ::id3v2::tag::frame_constants::PictureType::Other,
        0x01 => ::id3v2::tag::frame_constants::PictureType::FileIcon,
        0x02 => ::id3v2::tag::frame_constants::PictureType::OtherFileIcon,
        0x03 => ::id3v2::tag::frame_constants::PictureType::CoverFront,
        0x04 => ::id3v2::tag::frame_constants::PictureType::CoverBack,
        0x05 => ::id3v2::tag::frame_constants::PictureType::LeafletPage,
        0x06 => ::id3v2::tag::frame_constants::PictureType::Media,
        0x07 => ::id3v2::tag::frame_constants::PictureType::LeadArtist,
        0x08 => ::id3v2::tag::frame_constants::PictureType::Artist,
        0x09 => ::id3v2::tag::frame_constants::PictureType::Conductor,
        0x0A => ::id3v2::tag::frame_constants::PictureType::Band,
        0x0B => ::id3v2::tag::frame_constants::PictureType::Composer,
        0x0C => ::id3v2::tag::frame_constants::PictureType::Lyricist,
        0x0D => ::id3v2::tag::frame_constants::PictureType::RecordingLocation,
        0x0E => ::id3v2::tag::frame_constants::PictureType::DuringRecording,
        0x0F => ::id3v2::tag::frame_constants::PictureType::DuringPerformance,
        0x10 => ::id3v2::tag::frame_constants::PictureType::MovieScreenCapture,
        0x11 => ::id3v2::tag::frame_constants::PictureType::BrightColouredFish,
        0x12 => ::id3v2::tag::frame_constants::PictureType::Illustration,
        0x13 => ::id3v2::tag::frame_constants::PictureType::BandLogotype,
        0x14 => ::id3v2::tag::frame_constants::PictureType::PublisherLogoType,
        _ => ::id3v2::tag::frame_constants::PictureType::Other
    }
}

fn encoded_text(text_encoding: &::id3v2::bytes::TextEncoding, readable: &mut Readable) -> Result<(usize, String)> {
    Ok(match text_encoding {
        &::id3v2::bytes::TextEncoding::ISO8859_1 | &::id3v2::bytes::TextEncoding::UTF8 => readable.read_non_utf16_string()?,
        _ => readable.read_utf16_string()?
    })
}

type Readable = ::readable::Readable<io::Cursor<vec::Vec<u8>>>;

trait FrameDataBase<T> {
    fn to_framedata(readable: &mut Readable) -> Result<T>;
}

//Audio encryption
#[derive(Debug)]
pub struct AENC {
    owner_identifier: String,
    preview_start: u16,
    preview_end: u16,
    encryption_info: vec::Vec<u8>
}

impl FrameDataBase<AENC> for AENC {
    fn to_framedata(readable: &mut Readable) -> Result<AENC> {
        let (read, id) = readable.read_non_utf16_string()?;
        Ok(AENC {
            owner_identifier: id,
            preview_start: ::id3v2::bytes::to_u16(&readable.as_bytes(2)?),
            preview_end: ::id3v2::bytes::to_u16(&readable.as_bytes(2)?),
            encryption_info: readable.all_bytes()?
        })
    }
}

//Attached picture
#[derive(Debug)]
pub struct APIC {
    text_encoding: ::id3v2::bytes::TextEncoding,
    mime_type: String,
    picture_type: ::id3v2::tag::frame_constants::PictureType,
    description: String,
    picture_data: vec::Vec<u8>
}

impl FrameDataBase<APIC> for APIC {
    fn to_framedata(readable: &mut Readable) -> Result<APIC> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (read, mine_type) = readable.read_non_utf16_string()?;
        let picture_type = to_picture_type(readable.as_bytes(1)?[0]);
        let (_, description) = encoded_text(&text_encoding, readable)?;
        let picture_data = readable.all_bytes()?;
        Ok(APIC {
            text_encoding: text_encoding,
            mime_type: mine_type,
            picture_type: picture_type,
            description: description,
            picture_data: picture_data
        })
    }
}

// Audio seek point index
#[derive(Debug)]
pub struct ASPI {
    indexed_data_start: u32,
    indexed_data_length: u32,
    number_of_index_points: u16,
    bit_per_index_point: u8,
    fraction_at_index: u8
}

impl ASPI {
    pub fn get_indexed_data_start(&self) -> u32 {
        self.indexed_data_start
    }

    pub fn get_indexed_data_length(&self) -> u32 {
        self.indexed_data_length
    }

    pub fn get_number_of_index_points(&self) -> u16 {
        self.number_of_index_points
    }

    pub fn get_bit_per_index_point(&self) -> u8 {
        self.bit_per_index_point
    }

    pub fn get_fraction_at_index(&self) -> u8 {
        self.fraction_at_index
    }
}

impl FrameDataBase<ASPI> for ASPI {
    fn to_framedata(readable: &mut Readable) -> Result<ASPI> {
        let indexed_data_start = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let indexed_data_length = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let number_of_index_points = ::id3v2::bytes::to_u16(&readable.as_bytes(2)?);
        let bit_per_index_point = readable.as_bytes(1)?[0];
        let fraction_at_index = readable.as_bytes(1)?[0];

        Ok(ASPI {
            indexed_data_start: indexed_data_start,
            indexed_data_length: indexed_data_length,
            number_of_index_points: number_of_index_points,
            bit_per_index_point: bit_per_index_point,
            fraction_at_index: fraction_at_index
        })
    }
}

// Comments
#[derive(Debug)]
pub struct COMM {
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

impl FrameDataBase<COMM> for COMM {
    fn to_framedata(readable: &mut Readable) -> Result<COMM> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, short_description) = encoded_text(&text_encoding, readable)?;
        let actual_text = readable.all_string()?;
        Ok(COMM {
            text_encoding: text_encoding,
            language: language,
            short_description: short_description,
            actual_text: actual_text
        })
    }
}

// Commercial frame
#[derive(Debug)]
pub struct COMR {
    text_encoding: ::id3v2::bytes::TextEncoding,
    price_string: String,
    // 8 bit long
    valid_util: String,
    contat_url: String,
    received_as: ::id3v2::tag::frame_constants::ReceivedAs,
    name_of_seller: String,
    description: String,
    picture_mime_type: String,
    seller_logo: vec::Vec<u8>
}

// Encryption method registration
#[derive(Debug)]
pub struct ENCR {
    owner_identifier: String,
    method_symbol: u8,
    encryption_data: vec::Vec<u8>
}

// Equalisation (2)
#[derive(Debug)]
pub struct EQU2 {
    interpolation_method: u8,
    identification: String
}

// Event timing codes
#[derive(Debug)]
pub struct ETCO {
    time_stamp_format: ::id3v2::tag::frame_constants::TimeStampFormat
}

// General encapsulated object
#[derive(Debug)]
pub struct GEOB {
    text_encoding: ::id3v2::bytes::TextEncoding,
    mine_type: String,
    filename: String,
    content_description: String,
    encapsulation_object: vec::Vec<u8>
}

// Group identification registration
#[derive(Debug)]
pub struct GRID {
    owner_identifier: String,
    group_symbol: u8,
    group_dependent_data: vec::Vec<u8>
}

// Linked information
#[derive(Debug)]
pub struct LINK {
    frame_identifier: u32,
    url: String,
    additional_data: String
}

impl FrameDataBase<LINK> for LINK {
    fn to_framedata(readable: &mut Readable) -> Result<LINK> {
        let frame_id = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let (_, url) = readable.read_non_utf16_string()?;
        let additional_data = readable.all_string()?;
        Ok(LINK {
            frame_identifier: frame_id,
            url: url,
            additional_data: additional_data
        })
    }
}

// Music CD identifier
#[derive(Debug)]
pub struct MCDI {
    cd_toc: vec::Vec<u8>
}

// MPEG location lookup table
// TODO
#[derive(Debug)]
pub struct MLLT {
    data: vec::Vec<u8>
}

// Ownership frame
#[derive(Debug)]
pub struct OWNE {
    text_encoding: ::id3v2::bytes::TextEncoding,
    price_paid: String,
    // 8 bit long
    date_of_purch: String,
    seller: String
}

// Private frame
#[derive(Debug)]
pub struct PRIV {
    owner_identifier: String,
    private_data: vec::Vec<u8>
}

// Play counter
#[derive(Debug)]
pub struct PCNT {
    counter: u32
}

// Popularimeter
#[derive(Debug)]
pub struct POPM {
    email_to_user: String,
    rating: u8,
    counter: u32
}

// Position synchronisation frame
#[derive(Debug)]
pub struct POSS {
    time_stamp_format: ::id3v2::tag::frame_constants::TimeStampFormat,
    position: u8
}

// Recommended buffer size
#[derive(Debug)]
pub struct RBUF {
    buffer_size: u32,
    embedded_info_flag: u8,
    offset_to_next_tag: u32
}

// Relative volume adjustment (2)
#[derive(Debug)]
pub struct RVA2 {
    identification: String
}

// Reverb
#[derive(Debug)]
pub struct RVRB {
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
pub struct SEEK {
    next_tag: String
}

// Signature frame
#[derive(Debug)]
pub struct SIGN {
    group_symbol: u8,
    signature: vec::Vec<u8>
}

// Synchronised lyric/text
#[derive(Debug)]
pub struct SYLT {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    time_stamp_format: ::id3v2::tag::frame_constants::TimeStampFormat,
    content_type: ::id3v2::tag::frame_constants::ContentType,
    content_descriptor: String
}

// Synchronised tempo codes
#[derive(Debug)]
pub struct SYTC {
    time_stamp_format: ::id3v2::tag::frame_constants::TimeStampFormat,
    tempo_data: vec::Vec<u8>
}

// Unique file identifier
#[derive(Debug)]
pub struct UFID {
    owner_identifier: String,
    identifier: vec::Vec<u8>
}

// Terms of use
#[derive(Debug)]
pub struct USER {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    actual_text: String
}

// Unsynchronised lyric/text transcription
#[derive(Debug)]
pub struct USLT {
    text_encoding: ::id3v2::bytes::TextEncoding,
    language: String,
    content_descriptor: String,
    lyrics: String
}

#[derive(Debug)]
pub struct TEXT {
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

impl FrameDataBase<TEXT> for TEXT {
    fn to_framedata(readable: &mut Readable) -> Result<TEXT> {
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

        Ok(TEXT {
            text_encoding: text_encoding,
            text: text
        })
    }
}

#[derive(Debug)]
pub struct TXXX {
    text_encoding: ::id3v2::bytes::TextEncoding,
    description: String,
    value: String
}

impl TXXX {
    pub fn get_text_encoding(&self) -> &::id3v2::bytes::TextEncoding {
        &self.text_encoding
    }

    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }

    pub fn get_value(&self) -> &str {
        self.value.as_str()
    }
}

impl FrameDataBase<TXXX> for TXXX {
    fn to_framedata(readable: &mut Readable) -> Result<TXXX> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = encoded_text(&text_encoding, readable)?;
        let value = readable.all_string()?;
        Ok(TXXX {
            text_encoding: text_encoding,
            description: description,
            value: value
        })
    }
}

// User defined URL link frame
#[derive(Debug)]
pub struct WXXX {
    text_encoding: ::id3v2::bytes::TextEncoding,
    description: String,
    url: String
}

impl FrameDataBase<WXXX> for WXXX {
    fn to_framedata(readable: &mut Readable) -> Result<WXXX> {
        let text_encoding = ::id3v2::bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = encoded_text(&text_encoding, readable)?;
        let url = readable.all_string()?;
        Ok(WXXX {
            text_encoding: text_encoding,
            description: description,
            url: url
        })
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
    pub fn has_flag(&self, flag: ::id3v2::tag::frame_constants::FrameHeaderFlag, major_version: u8) -> bool {
        if major_version == 3 {
            match flag {
                ::id3v2::tag::frame_constants::FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 7 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 6 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 5 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 7 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 6 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 5 != 0,
                _ => false
            }
        } else if major_version == 4 {
            match flag {
                ::id3v2::tag::frame_constants::FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 6 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 5 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 4 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 6 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 3 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 2 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::Unsynchronisation => self.encoding_flag & 0x01 << 1 != 0,
                ::id3v2::tag::frame_constants::FrameHeaderFlag::DataLength => self.encoding_flag & 0x01 != 0
            }
        } else {
            warn!("Frame.has_flag=> Unknown version!");
            false
        }
    }

    // @see http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
    pub fn get_data(&self) -> Result<::id3v2::tag::frame_constants::FrameData> {
        let mut readable = ::readable::factory::from_byte(self.data.clone())?;
        Ok(match self.id.as_ref() {
            ::id3v2::tag::frame_constants::id::AENC_STR => ::id3v2::tag::frame_constants::FrameData::AENC(AENC::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::APIC_STR => ::id3v2::tag::frame_constants::FrameData::APIC(APIC::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::ASPI_STR => ::id3v2::tag::frame_constants::FrameData::ASPI(ASPI::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::COMM_STR => ::id3v2::tag::frame_constants::FrameData::COMM(COMM::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TALB_STR => ::id3v2::tag::frame_constants::FrameData::TALB(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TBPM_STR => ::id3v2::tag::frame_constants::FrameData::TBPM(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TCOM_STR => ::id3v2::tag::frame_constants::FrameData::TCOM(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TCON_STR => ::id3v2::tag::frame_constants::FrameData::TCON(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TCOP_STR => ::id3v2::tag::frame_constants::FrameData::TCOP(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TDEN_STR => ::id3v2::tag::frame_constants::FrameData::TDEN(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TDLY_STR => ::id3v2::tag::frame_constants::FrameData::TDLY(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TDOR_STR => ::id3v2::tag::frame_constants::FrameData::TDOR(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TDRC_STR => ::id3v2::tag::frame_constants::FrameData::TDRC(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TDRL_STR => ::id3v2::tag::frame_constants::FrameData::TDRL(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TDTG_STR => ::id3v2::tag::frame_constants::FrameData::TDTG(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TENC_STR => ::id3v2::tag::frame_constants::FrameData::TENC(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TEXT_STR => ::id3v2::tag::frame_constants::FrameData::TEXT(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TFLT_STR => ::id3v2::tag::frame_constants::FrameData::TFLT(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TIPL_STR => ::id3v2::tag::frame_constants::FrameData::TIPL(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TIT1_STR => ::id3v2::tag::frame_constants::FrameData::TIT1(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TIT2_STR => ::id3v2::tag::frame_constants::FrameData::TIT2(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TIT3_STR => ::id3v2::tag::frame_constants::FrameData::TIT3(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TKEY_STR => ::id3v2::tag::frame_constants::FrameData::TKEY(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TLAN_STR => ::id3v2::tag::frame_constants::FrameData::TLAN(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TLEN_STR => ::id3v2::tag::frame_constants::FrameData::TLEN(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TMCL_STR => ::id3v2::tag::frame_constants::FrameData::TMCL(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TMED_STR => ::id3v2::tag::frame_constants::FrameData::TMED(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TMOO_STR => ::id3v2::tag::frame_constants::FrameData::TMOO(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TOAL_STR => ::id3v2::tag::frame_constants::FrameData::TOAL(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TOFN_STR => ::id3v2::tag::frame_constants::FrameData::TOFN(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TOLY_STR => ::id3v2::tag::frame_constants::FrameData::TOLY(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TOPE_STR => ::id3v2::tag::frame_constants::FrameData::TOPE(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TOWN_STR => ::id3v2::tag::frame_constants::FrameData::TOWN(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPE1_STR => ::id3v2::tag::frame_constants::FrameData::TPE1(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPE2_STR => ::id3v2::tag::frame_constants::FrameData::TPE2(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPE3_STR => ::id3v2::tag::frame_constants::FrameData::TPE3(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPE4_STR => ::id3v2::tag::frame_constants::FrameData::TPE4(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPOS_STR => ::id3v2::tag::frame_constants::FrameData::TPOS(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPRO_STR => ::id3v2::tag::frame_constants::FrameData::TPRO(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TPUB_STR => ::id3v2::tag::frame_constants::FrameData::TPUB(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TRCK_STR => ::id3v2::tag::frame_constants::FrameData::TRCK(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TRSN_STR => ::id3v2::tag::frame_constants::FrameData::TRSN(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TRSO_STR => ::id3v2::tag::frame_constants::FrameData::TRSO(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TSOA_STR => ::id3v2::tag::frame_constants::FrameData::TSOA(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TSOP_STR => ::id3v2::tag::frame_constants::FrameData::TSOP(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TSOT_STR => ::id3v2::tag::frame_constants::FrameData::TSOT(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TSRC_STR => ::id3v2::tag::frame_constants::FrameData::TSRC(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TSSE_STR => ::id3v2::tag::frame_constants::FrameData::TSSE(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TSST_STR => ::id3v2::tag::frame_constants::FrameData::TSST(TEXT::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::TXXX_STR => ::id3v2::tag::frame_constants::FrameData::TXXX(TXXX::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WCOM_STR => ::id3v2::tag::frame_constants::FrameData::WCOM(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WOAF_STR => ::id3v2::tag::frame_constants::FrameData::WOAF(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WOAR_STR => ::id3v2::tag::frame_constants::FrameData::WOAR(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WOAS_STR => ::id3v2::tag::frame_constants::FrameData::WOAS(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WORS_STR => ::id3v2::tag::frame_constants::FrameData::WORS(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WPAY_STR => ::id3v2::tag::frame_constants::FrameData::WPAY(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WPUB_STR => ::id3v2::tag::frame_constants::FrameData::WPUB(LINK::to_framedata(&mut readable)?),
            ::id3v2::tag::frame_constants::id::WXXX_STR => ::id3v2::tag::frame_constants::FrameData::WXXX(WXXX::to_framedata(&mut readable)?),
            _ => ::id3v2::tag::frame_constants::FrameData::TEXT(self::TEXT::to_framedata(&mut readable)?)
        })
    }
}
