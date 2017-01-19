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
use std::{io, vec};


type FrameResult<T> = ::std::result::Result<T, ::errors::ParsingError>;
type Readable = ::readable::Readable<io::Cursor<vec::Vec<u8>>>;

fn is_valid_frame_id(id: &str) -> bool {
    // TODO const
    let reg = regex::Regex::new(r"^[A-Z][A-Z0-9]{3}$").unwrap();
    reg.is_match(id)
}

fn trim_white_space_include_bom(text: String) -> String {
    // TODO const
    let reg = regex::Regex::new(r"(^[\x{0}|\x{feff}|\x{fffe}]*|[\x{0}|\x{feff}|\x{fffe}]*$)").unwrap();
    reg.replace_all(text.as_str(), "").into_owned()
}

fn to_picture_type(t: u8) -> ::id3v2::frame_constants::PictureType {
    match t {
        0x00 => ::id3v2::frame_constants::PictureType::Other,
        0x01 => ::id3v2::frame_constants::PictureType::FileIcon,
        0x02 => ::id3v2::frame_constants::PictureType::OtherFileIcon,
        0x03 => ::id3v2::frame_constants::PictureType::CoverFront,
        0x04 => ::id3v2::frame_constants::PictureType::CoverBack,
        0x05 => ::id3v2::frame_constants::PictureType::LeafletPage,
        0x06 => ::id3v2::frame_constants::PictureType::Media,
        0x07 => ::id3v2::frame_constants::PictureType::LeadArtist,
        0x08 => ::id3v2::frame_constants::PictureType::Artist,
        0x09 => ::id3v2::frame_constants::PictureType::Conductor,
        0x0a => ::id3v2::frame_constants::PictureType::Band,
        0x0b => ::id3v2::frame_constants::PictureType::Composer,
        0x0c => ::id3v2::frame_constants::PictureType::Lyricist,
        0x0d => ::id3v2::frame_constants::PictureType::RecordingLocation,
        0x0e => ::id3v2::frame_constants::PictureType::DuringRecording,
        0x0f => ::id3v2::frame_constants::PictureType::DuringPerformance,
        0x10 => ::id3v2::frame_constants::PictureType::MovieScreenCapture,
        0x11 => ::id3v2::frame_constants::PictureType::BrightColouredFish,
        0x12 => ::id3v2::frame_constants::PictureType::Illustration,
        0x13 => ::id3v2::frame_constants::PictureType::BandLogotype,
        0x14 => ::id3v2::frame_constants::PictureType::PublisherLogoType,
        _ => ::id3v2::frame_constants::PictureType::Other
    }
}

fn to_received_as(t: u8) -> ::id3v2::frame_constants::ReceivedAs {
    match t {
        0x00 => ::id3v2::frame_constants::ReceivedAs::Other,
        0x01 => ::id3v2::frame_constants::ReceivedAs::StandardCDAlbum,
        0x02 => ::id3v2::frame_constants::ReceivedAs::CompressedAudioOnCD,
        0x03 => ::id3v2::frame_constants::ReceivedAs::FileOverInternet,
        0x04 => ::id3v2::frame_constants::ReceivedAs::StreamOverInternet,
        0x05 => ::id3v2::frame_constants::ReceivedAs::AsNoteSheets,
        0x06 => ::id3v2::frame_constants::ReceivedAs::AsNoteSheetsInBook,
        0x07 => ::id3v2::frame_constants::ReceivedAs::MusicOnMedia,
        0x08 => ::id3v2::frame_constants::ReceivedAs::NonMusicalMerchandise,
        _ => ::id3v2::frame_constants::ReceivedAs::Other
    }
}

fn to_interpolation_method(t: u8) -> ::id3v2::frame_constants::InterpolationMethod {
    match t {
        0x00 => ::id3v2::frame_constants::InterpolationMethod::Band,
        0x01 => ::id3v2::frame_constants::InterpolationMethod::Linear,
        _ => ::id3v2::frame_constants::InterpolationMethod::Band
    }
}

fn to_timestamp_format(t: u8) -> ::id3v2::frame_constants::TimestampFormat {
    match t {
        0x01 => ::id3v2::frame_constants::TimestampFormat::MpecFrames,
        0x02 => ::id3v2::frame_constants::TimestampFormat::Milliseconds,
        _ => ::id3v2::frame_constants::TimestampFormat::MpecFrames
    }
}

fn to_event_timing_code(t: u8, timestamp: u32) -> ::id3v2::frame_constants::EventTimingCode {
    match t {
        0x00 => ::id3v2::frame_constants::EventTimingCode::Padding(timestamp),
        0x01 => ::id3v2::frame_constants::EventTimingCode::EndOfInitialSilence(timestamp),
        0x02 => ::id3v2::frame_constants::EventTimingCode::IntroStart(timestamp),
        0x03 => ::id3v2::frame_constants::EventTimingCode::MainPartStart(timestamp),
        0x04 => ::id3v2::frame_constants::EventTimingCode::OutroStart(timestamp),
        0x05 => ::id3v2::frame_constants::EventTimingCode::OutroEnd(timestamp),
        0x06 => ::id3v2::frame_constants::EventTimingCode::VerseStart(timestamp),
        0x07 => ::id3v2::frame_constants::EventTimingCode::RefrainStart(timestamp),
        0x08 => ::id3v2::frame_constants::EventTimingCode::InterludeStart(timestamp),
        0x09 => ::id3v2::frame_constants::EventTimingCode::ThemeStart(timestamp),
        0x0a => ::id3v2::frame_constants::EventTimingCode::VariationStart(timestamp),
        0x0b => ::id3v2::frame_constants::EventTimingCode::KeyChange(timestamp),
        0x0c => ::id3v2::frame_constants::EventTimingCode::TimeChange(timestamp),
        0x0d => ::id3v2::frame_constants::EventTimingCode::MomentaryUnwantedNoise(timestamp),
        0x0e => ::id3v2::frame_constants::EventTimingCode::SustainedNoise(timestamp),
        0x0f => ::id3v2::frame_constants::EventTimingCode::SustainedNoiseEnd(timestamp),
        0x10 => ::id3v2::frame_constants::EventTimingCode::IntroEnd(timestamp),
        0x11 => ::id3v2::frame_constants::EventTimingCode::MainPartEnd(timestamp),
        0x12 => ::id3v2::frame_constants::EventTimingCode::VerseEnd(timestamp),
        0x13 => ::id3v2::frame_constants::EventTimingCode::RefrainEnd(timestamp),
        0x14 => ::id3v2::frame_constants::EventTimingCode::ThemeEnd(timestamp),
        0x15 => ::id3v2::frame_constants::EventTimingCode::Profanity(timestamp),
        0x16 => ::id3v2::frame_constants::EventTimingCode::ProfanityEnd(timestamp),
        0x17 ... 0xdf => ::id3v2::frame_constants::EventTimingCode::ReservedForFutureUse(timestamp),
        0xe0 ... 0xef => ::id3v2::frame_constants::EventTimingCode::NotPredefinedSynch(timestamp),
        0xf0 ... 0xfc => ::id3v2::frame_constants::EventTimingCode::ReservedForFutureUse(timestamp),
        0xfd => ::id3v2::frame_constants::EventTimingCode::AudioEnd(timestamp),
        0xfe => ::id3v2::frame_constants::EventTimingCode::AudioFileEnds(timestamp),
        0xff => ::id3v2::frame_constants::EventTimingCode::OneMoreByteOfEventsFollows(timestamp),
        _ => ::id3v2::frame_constants::EventTimingCode::Padding(timestamp)
    }
}

fn read_null_terminated(text_encoding: &::id3v2::frame_constants::TextEncoding, readable: &mut Readable)
                        -> FrameResult<(usize, String)> {
    Ok(match text_encoding {
        &::id3v2::frame_constants::TextEncoding::ISO8859_1 | &::id3v2::frame_constants::TextEncoding::UTF8 =>
            readable.read_terminated_null()?,
        _ => {
            debug!("Unknown text_encoding {:?}", text_encoding);
            readable.read_terminated_utf16()?
        }
    })
}

fn read_null_terminated_bytes(text_encoding: &::id3v2::frame_constants::TextEncoding, readable: &mut Readable)
                              -> FrameResult<vec::Vec<u8>> {
    Ok(match text_encoding {
        &::id3v2::frame_constants::TextEncoding::ISO8859_1 | &::id3v2::frame_constants::TextEncoding::UTF8 =>
            readable.read_terminated_null_bytes()?,
        _ => {
            debug!("Unknown text_encoding {:?}", text_encoding);
            readable.read_terminated_utf16_bytes()?
        }
    })
}

fn trim_to_u32(bytes: &mut vec::Vec<u8>) -> u32 {
    let len = bytes.len();
    if len > 4 {
        bytes.split_off(len - 4);
    }
    ::id3v2::bytes::to_u32(&bytes)
}

fn as_encoded_text(text_encoding: &::id3v2::frame_constants::TextEncoding, data: &vec::Vec<u8>) -> FrameResult<String> {
    fn to_parsing_err(err: ::std::borrow::Cow<'static, str>) -> ::errors::ParsingError {
        ::errors::ParsingError::EncodeDecodeError(err)
    }

    Ok(match text_encoding {
        &::id3v2::frame_constants::TextEncoding::ISO8859_1 => encoding::all::ISO_8859_1
            .decode(data, encoding::DecoderTrap::Strict).map_err(to_parsing_err)?,

        &::id3v2::frame_constants::TextEncoding::UTF16LE => encoding::all::UTF_16LE
            .decode(data, encoding::DecoderTrap::Strict).map_err(to_parsing_err)?,

        &::id3v2::frame_constants::TextEncoding::UTF16BE => encoding::all::UTF_16BE
            .decode(data, encoding::DecoderTrap::Strict).map_err(to_parsing_err)?,

        &::id3v2::frame_constants::TextEncoding::UTF8 => encoding::all::UTF_8
            .decode(data, encoding::DecoderTrap::Strict).map_err(to_parsing_err)?,
    })
}

fn to_content_type(t: u8) -> ::id3v2::frame_constants::ContentType {
    match t {
        0x00 => ::id3v2::frame_constants::ContentType::Other,
        0x01 => ::id3v2::frame_constants::ContentType::Lyrics,
        0x02 => ::id3v2::frame_constants::ContentType::TextTranscription,
        0x03 => ::id3v2::frame_constants::ContentType::MovementName,
        0x04 => ::id3v2::frame_constants::ContentType::Events,
        0x05 => ::id3v2::frame_constants::ContentType::Chord,
        0x06 => ::id3v2::frame_constants::ContentType::Trivia,
        0x07 => ::id3v2::frame_constants::ContentType::UrlsToWebpages,
        0x08 => ::id3v2::frame_constants::ContentType::UrlsToImages,
        _ => ::id3v2::frame_constants::ContentType::Other
    }
}

fn to_encoding(encoding: u8) -> ::id3v2::frame_constants::TextEncoding {
    match encoding {
        0 => ::id3v2::frame_constants::TextEncoding::ISO8859_1,
        1 => ::id3v2::frame_constants::TextEncoding::UTF16LE,
        2 => ::id3v2::frame_constants::TextEncoding::UTF16BE,
        3 => ::id3v2::frame_constants::TextEncoding::UTF8,
        _ => ::id3v2::frame_constants::TextEncoding::ISO8859_1
    }
}

trait FrameDataBase<T> {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<T>;
}

// TODO not yet tested
// 2.2 only
// Encrypted meta frame
#[derive(Debug)]
pub struct CRM {
    owner_identifier: String,
    content: String,
    encrypted_datablock: vec::Vec<u8>
}

impl CRM {
    pub fn owner_identifier(&self) -> &str {
        self.owner_identifier.as_str()
    }
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
    pub fn encrypted_datablock(&self) -> &[u8] {
        &self.encrypted_datablock
    }
}

impl FrameDataBase<CRM> for CRM {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<CRM> {
        let (_, owner_identifier) = readable.read_terminated_null()?;
        let (_, content) = readable.read_terminated_null()?;
        let encrypted_datablock = readable.all_bytes()?;

        let ret = Ok(CRM {
            owner_identifier: owner_identifier,
            content: content,
            encrypted_datablock: encrypted_datablock
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested
// 2.2 only
// Attached picture
#[derive(Debug)]
pub struct PIC {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    image_format: String,
    picture_type: ::id3v2::frame_constants::PictureType,
    description: String,
    picture_data: vec::Vec<u8>
}

impl PIC {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_image_format(&self) -> &str {
        self.image_format.as_str()
    }

    pub fn get_picture_type(&self) -> &::id3v2::frame_constants::PictureType {
        &self.picture_type
    }

    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }

    pub fn get_picture_data(&self) -> &[u8] {
        &self.picture_data
    }
}

impl FrameDataBase<PIC> for PIC {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<PIC> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let image_format = readable.as_string(3)?;
        let picture_type = to_picture_type(readable.as_bytes(1)?[0]);
        let (_, description) = read_null_terminated(&text_encoding, readable)?;
        let picture_data = readable.all_bytes()?;

        let ret = Ok(PIC {
            text_encoding: text_encoding,
            image_format: image_format,
            picture_type: picture_type,
            description: description,
            picture_data: picture_data
        });

        trace!("{:?}", ret);

        ret
    }
}

//Audio encryption
#[derive(Debug)]
pub struct AENC {
    owner_identifier: String,
    preview_start: u16,
    preview_end: u16,
    encryption_info: vec::Vec<u8>
}

impl AENC {
    pub fn get_owner_identifier(&self) -> &str {
        self.owner_identifier.as_str()
    }

    pub fn get_preview_start(&self) -> u16 {
        self.preview_start
    }

    pub fn get_preview_end(&self) -> u16 {
        self.preview_end
    }

    pub fn get_encryption_info(&self) -> &[u8] {
        &self.encryption_info
    }
}

impl FrameDataBase<AENC> for AENC {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<AENC> {
        let (_, id) = readable.read_terminated_null()?;
        let preview_start = ::id3v2::bytes::to_u16(&readable.as_bytes(2)?);
        let preview_end = ::id3v2::bytes::to_u16(&readable.as_bytes(2)?);
        let encryption_info = readable.all_bytes()?;

        let ret = Ok(AENC {
            owner_identifier: id,
            preview_start: preview_start,
            preview_end: preview_end,
            encryption_info: encryption_info
        });

        trace!("{:?}", ret);

        ret
    }
}

//Attached picture
#[derive(Debug)]
pub struct APIC {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    mime_type: String,
    picture_type: ::id3v2::frame_constants::PictureType,
    description: String,
    picture_data: vec::Vec<u8>
}

impl APIC {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }
    pub fn get_mime_type(&self) -> &str {
        self.mime_type.as_str()
    }
    pub fn get_picture_type(&self) -> &::id3v2::frame_constants::PictureType {
        &self.picture_type
    }
    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }
    pub fn get_picture_data(&self) -> &[u8] {
        &self.picture_data
    }
}

impl FrameDataBase<APIC> for APIC {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<APIC> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, mine_type) = readable.read_terminated_null()?;
        let picture_type = to_picture_type(readable.as_bytes(1)?[0]);
        let (_, description) = read_null_terminated(&text_encoding, readable)?;
        let picture_data = readable.all_bytes()?;

        let ret = Ok(APIC {
            text_encoding: text_encoding,
            mime_type: mine_type,
            picture_type: picture_type,
            description: description,
            picture_data: picture_data
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
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
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<ASPI> {
        let indexed_data_start = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let indexed_data_length = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let number_of_index_points = ::id3v2::bytes::to_u16(&readable.as_bytes(2)?);
        let bit_per_index_point = readable.as_bytes(1)?[0];
        let fraction_at_index = readable.as_bytes(1)?[0];

        let ret = Ok(ASPI {
            indexed_data_start: indexed_data_start,
            indexed_data_length: indexed_data_length,
            number_of_index_points: number_of_index_points,
            bit_per_index_point: bit_per_index_point,
            fraction_at_index: fraction_at_index
        });

        trace!("{:?}", ret);

        ret
    }
}

// Comments
#[derive(Debug)]
pub struct COMM {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    language: String,
    short_description: String,
    actual_text: String
}

impl COMM {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<COMM> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, short_description) = read_null_terminated(&text_encoding, readable)?;
        let actual_text = readable.all_string()?;

        let ret = Ok(COMM {
            text_encoding: text_encoding,
            language: language,
            short_description: short_description,
            actual_text: actual_text
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Commercial frame
#[derive(Debug)]
pub struct COMR {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    price_string: String,
    // 8 bit long
    valid_util: String,
    contact_url: String,
    received_as: ::id3v2::frame_constants::ReceivedAs,
    name_of_seller: String,
    description: String,
    picture_mime_type: String,
    seller_logo: vec::Vec<u8>
}

impl COMR {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_price_string(&self) -> &str {
        self.price_string.as_str()
    }

    pub fn get_valid_util(&self) -> &str {
        self.valid_util.as_str()
    }

    pub fn get_contact_url(&self) -> &str {
        self.contact_url.as_str()
    }

    pub fn get_received_as(&self) -> &::id3v2::frame_constants::ReceivedAs {
        &self.received_as
    }

    pub fn get_name_of_seller(&self) -> &str {
        self.name_of_seller.as_str()
    }

    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }

    pub fn get_picture_mime_type(&self) -> &str {
        self.picture_mime_type.as_str()
    }

    pub fn get_seller_logo(&self) -> &[u8] {
        &self.seller_logo
    }
}

impl FrameDataBase<COMR> for COMR {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<COMR> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, price_string) = readable.read_terminated_null()?;
        let valid_util = readable.as_string(8)?;
        let (_, contact_url) = readable.read_terminated_null()?;
        let received_as = to_received_as(readable.as_bytes(1)?[0]);
        let (_, name_of_seller) = readable.read_terminated_utf16()?;
        let (_, description) = readable.read_terminated_utf16()?;
        let (_, picture_mime_type) = readable.read_terminated_null()?;
        let seller_logo = readable.all_bytes()?;

        let ret = Ok(COMR {
            text_encoding: text_encoding,
            price_string: price_string,
            valid_util: valid_util,
            contact_url: contact_url,
            received_as: received_as,
            name_of_seller: name_of_seller,
            description: description,
            picture_mime_type: picture_mime_type,
            seller_logo: seller_logo
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Encryption method registration
#[derive(Debug)]
pub struct ENCR {
    owner_identifier: String,
    method_symbol: u8,
    encryption_data: vec::Vec<u8>
}

impl ENCR {
    pub fn get_owner_identifier(&self) -> &str {
        self.owner_identifier.as_str()
    }

    pub fn get_method_symbol(&self) -> u8 {
        self.method_symbol
    }

    pub fn get_encryption_data(&self) -> &[u8] {
        &self.encryption_data
    }
}

impl FrameDataBase<ENCR> for ENCR {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<ENCR> {
        let (_, owner_identifier) = readable.read_terminated_null()?;
        let method_symbol = readable.as_bytes(1)?[0];
        let encryption_data = readable.all_bytes()?;

        let ret = Ok(ENCR {
            owner_identifier: owner_identifier,
            method_symbol: method_symbol,
            encryption_data: encryption_data
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Equalisation
#[derive(Debug)]
pub struct EQUA {
    adjustment_bit: u8
}

impl EQUA {
    pub fn get_adjustment_bit(&self) -> u8 {
        self.adjustment_bit
    }
}

impl FrameDataBase<EQUA> for EQUA {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<EQUA> {
        let adjustment_bit = readable.as_bytes(1)?[0];

        let ret = Ok(EQUA {
            adjustment_bit: adjustment_bit
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Equalisation (2)
#[derive(Debug)]
pub struct EQU2 {
    interpolation_method: ::id3v2::frame_constants::InterpolationMethod,
    identification: String
}

impl EQU2 {
    pub fn get_interpolation_method(&self) -> &::id3v2::frame_constants::InterpolationMethod {
        &self.interpolation_method
    }

    pub fn get_identification(&self) -> &str {
        self.identification.as_str()
    }
}

impl FrameDataBase<EQU2> for EQU2 {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<EQU2> {
        let interpolation_method = to_interpolation_method(readable.as_bytes(1)?[0]);
        let (_, identification) = readable.read_terminated_null()?;

        let ret = Ok(EQU2 {
            interpolation_method: interpolation_method,
            identification: identification
        });

        trace!("{:?}", ret);

        ret
    }
}

// Event timing codes
#[derive(Debug)]
pub struct ETCO {
    timestamp_format: ::id3v2::frame_constants::TimestampFormat,
    event_timing_codes: vec::Vec<::id3v2::frame_constants::EventTimingCode>
}

impl ETCO {
    pub fn get_timestamp_format(&self) -> &::id3v2::frame_constants::TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_event_timing_codes(&self) -> &[::id3v2::frame_constants::EventTimingCode] {
        &self.event_timing_codes
    }
}

impl FrameDataBase<ETCO> for ETCO {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<ETCO> {
        let timestamp_format = to_timestamp_format(readable.as_bytes(1)?[0]);
        let mut event_timing_codes: vec::Vec<::id3v2::frame_constants::EventTimingCode> = vec::Vec::new();
        loop {
            let mut is_break = true;
            if let Ok(code_type) = readable.as_bytes(1) {
                if let Ok(timestamp) = readable.as_bytes(4) {
                    let event_timing_code = to_event_timing_code(code_type[0], ::id3v2::bytes::to_u32(&timestamp));
                    event_timing_codes.push(event_timing_code);
                    is_break = false;
                }
            }

            if is_break {
                break;
            }
        }

        let ret = Ok(ETCO {
            timestamp_format: timestamp_format,
            event_timing_codes: event_timing_codes
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// General encapsulated object
#[derive(Debug)]
pub struct GEOB {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    mine_type: String,
    filename: String,
    content_description: String,
    encapsulation_object: vec::Vec<u8>
}

impl GEOB {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_mine_type(&self) -> &str {
        self.mine_type.as_str()
    }

    pub fn get_filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn get_content_description(&self) -> &str {
        self.content_description.as_str()
    }

    pub fn get_encapsulation_object(&self) -> &[u8] {
        &self.encapsulation_object
    }
}

impl FrameDataBase<GEOB> for GEOB {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<GEOB> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, mine_type) = readable.read_terminated_null()?;
        let (_, filename) = readable.read_terminated_utf16()?;
        let (_, content_description) = readable.read_terminated_utf16()?;
        let encapsulation_object = readable.all_bytes()?;

        let ret = Ok(GEOB {
            text_encoding: text_encoding,
            mine_type: mine_type,
            filename: filename,
            content_description: content_description,
            encapsulation_object: encapsulation_object
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Group identification registration
#[derive(Debug)]
pub struct GRID {
    owner_identifier: String,
    group_symbol: u8,
    group_dependent_data: vec::Vec<u8>
}

impl GRID {
    pub fn get_owner_identifier(&self) -> &str {
        self.owner_identifier.as_str()
    }

    pub fn get_group_symbol(&self) -> u8 {
        self.group_symbol
    }

    pub fn get_group_dependent_data(&self) -> &[u8] {
        &self.group_dependent_data
    }
}

impl FrameDataBase<GRID> for GRID {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<GRID> {
        let (_, owner_identifier) = readable.read_terminated_null()?;
        let group_symbol = readable.as_bytes(1)?[0];
        let group_dependent_data = readable.all_bytes()?;

        let ret = Ok(GRID {
            owner_identifier: owner_identifier,
            group_symbol: group_symbol,
            group_dependent_data: group_dependent_data
        });

        trace!("{:?}", ret);

        ret
    }
}

#[derive(Debug)]
// Involved people list
pub struct IPLS {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    people_list_strings: String
}

impl IPLS {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_people_list_strings(&self) -> &str {
        self.people_list_strings.as_str()
    }
}

impl FrameDataBase<IPLS> for IPLS {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<IPLS> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, people_list_strings) = read_null_terminated(&text_encoding, readable)?;

        let ret = Ok(IPLS {
            text_encoding: text_encoding,
            people_list_strings: people_list_strings
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Linked information
#[derive(Debug)]
pub struct LINK {
    frame_identifier: u32,
    url: String,
    additional_data: String
}

impl LINK {
    pub fn get_frame_identifier(&self) -> u32 {
        self.frame_identifier
    }

    pub fn get_url(&self) -> &str {
        self.url.as_str()
    }

    pub fn get_additional_data(&self) -> &str {
        self.additional_data.as_str()
    }
}

impl FrameDataBase<LINK> for LINK {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<LINK> {
        let frame_id = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);
        let (_, url) = readable.read_terminated_null()?;
        let additional_data = readable.all_string()?;

        let ret = Ok(LINK {
            frame_identifier: frame_id,
            url: url,
            additional_data: additional_data
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Music CD identifier
#[derive(Debug)]
pub struct MCDI {
    cd_toc: vec::Vec<u8>
}

impl MCDI {
    pub fn get_cd_toc(&self) -> &[u8] {
        &self.cd_toc
    }
}

impl FrameDataBase<MCDI> for MCDI {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<MCDI> {
        let cd_toc = readable.all_bytes()?;

        let ret = Ok(MCDI {
            cd_toc: cd_toc
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// TODO not yet implemented!
// MPEG location lookup table
#[derive(Debug)]
pub struct MLLT {
    data: vec::Vec<u8>
}

impl MLLT {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl FrameDataBase<MLLT> for MLLT {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<MLLT> {
        let data = readable.all_bytes()?;

        let ret = Ok(MLLT {
            data: data
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Ownership frame
#[derive(Debug)]
pub struct OWNE {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    price_paid: String,
    // 8 bit long
    date_of_purch: String,
    seller: String
}

impl OWNE {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_price_paid(&self) -> &str {
        self.price_paid.as_str()
    }

    pub fn get_date_of_purch(&self) -> &str {
        self.date_of_purch.as_str()
    }

    pub fn get_seller(&self) -> &str {
        self.seller.as_str()
    }
}

impl FrameDataBase<OWNE> for OWNE {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<OWNE> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, price_paid) = readable.read_terminated_null()?;
        let date_of_purch = readable.as_string(4)?;
        let (_, seller) = read_null_terminated(&text_encoding, readable)?;

        let ret = Ok(OWNE {
            text_encoding: text_encoding,
            price_paid: price_paid,
            date_of_purch: date_of_purch,
            seller: seller
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Private frame
#[derive(Debug)]
pub struct PRIV {
    owner_identifier: String,
    private_data: vec::Vec<u8>
}

impl PRIV {
    pub fn get_owner_identifier(&self) -> &str {
        self.owner_identifier.as_str()
    }

    pub fn get_private_data(&self) -> &[u8] {
        &self.private_data
    }
}

impl FrameDataBase<PRIV> for PRIV {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<PRIV> {
        let (_, owner_identifier) = readable.read_terminated_null()?;
        let private_data = readable.all_bytes()?;

        let ret = Ok(PRIV {
            owner_identifier: owner_identifier,
            private_data: private_data
        });

        trace!("{:?}", ret);

        ret
    }
}

// NOTE it support that only the 32-bit unsigned integer type.
// Play counter
#[derive(Debug)]
pub struct PCNT {
    counter: u32
}

impl PCNT {
    pub fn get_counter(&self) -> u32 {
        self.counter
    }
}

impl FrameDataBase<PCNT> for PCNT {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<PCNT> {
        let mut all_bytes = readable.all_bytes()?;
        let counter = trim_to_u32(&mut all_bytes);

        let ret = Ok(PCNT {
            counter: counter
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Popularimeter
#[derive(Debug)]
pub struct POPM {
    email_to_user: String,
    rating: u8,
    // NOTE it support that only the 32-bit unsigned integer type.
    counter: u32
}

impl POPM {
    pub fn get_email_to_user(&self) -> &str {
        self.email_to_user.as_str()
    }

    pub fn get_rating(&self) -> u8 {
        self.rating
    }

    pub fn get_counter(&self) -> u32 {
        self.counter
    }
}

impl FrameDataBase<POPM> for POPM {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<POPM> {
        let (_, email_to_user) = readable.read_terminated_null()?;
        let rating = readable.as_bytes(1)?[0];
        let counter = {
            let mut all_bytes = readable.all_bytes()?;
            trim_to_u32(&mut all_bytes)
        };

        let ret = Ok(POPM {
            email_to_user: email_to_user,
            rating: rating,
            counter: counter
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Position synchronisation frame
#[derive(Debug)]
pub struct POSS {
    timestamp_format: ::id3v2::frame_constants::TimestampFormat,
    // TODO not yet implemented!
    position: vec::Vec<u8>
}

impl POSS {
    pub fn get_timestamp_format(&self) -> &::id3v2::frame_constants::TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_position(&self) -> &[u8] {
        &self.position
    }
}

impl FrameDataBase<POSS> for POSS {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<POSS> {
        let timestamp_format = to_timestamp_format(readable.as_bytes(1)?[0]);
        let position = readable.all_bytes()?;

        let ret = Ok(POSS {
            timestamp_format: timestamp_format,
            position: position
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Recommended buffer size
#[derive(Debug)]
pub struct RBUF {
    buffer_size: u32,
    embedded_info_flag: u8,
    offset_to_next_tag: u32
}

impl RBUF {
    pub fn get_buffer_size(&self) -> u32 {
        self.buffer_size
    }

    pub fn get_embedded_info_flag(&self) -> u8 {
        self.embedded_info_flag
    }

    pub fn get_offset_to_next_tag(&self) -> u32 {
        self.offset_to_next_tag
    }
}

impl FrameDataBase<RBUF> for RBUF {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<RBUF> {
        let buffer_size = ::id3v2::bytes::to_u32(&readable.as_bytes(3)?);
        let embedded_info_flag = readable.as_bytes(1)?[0] & 0x01;
        let offset_to_next_tag = ::id3v2::bytes::to_u32(&readable.as_bytes(4)?);

        let ret = Ok(RBUF {
            buffer_size: buffer_size,
            embedded_info_flag: embedded_info_flag,
            offset_to_next_tag: offset_to_next_tag
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// TODO not yet implemented!
// Relative volume adjustment (2)
#[derive(Debug)]
pub struct RVA2 {
    data: vec::Vec<u8>
}

impl RVA2 {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl FrameDataBase<RVA2> for RVA2 {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<RVA2> {
        let data = readable.all_bytes()?;

        let ret = Ok(RVA2 {
            data: data
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
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

impl RVRB {
    pub fn get_reverb_left(&self) -> u16 {
        self.reverb_left
    }
    pub fn get_reverb_right(&self) -> u16 {
        self.reverb_right
    }
    pub fn get_reverb_bounce_left(&self) -> u8 {
        self.reverb_bounce_left
    }
    pub fn get_reverb_bounce_right(&self) -> u8 {
        self.reverb_bounce_right
    }
    pub fn get_reverb_feedback_left_to_left(&self) -> u8 {
        self.reverb_feedback_left_to_left
    }
    pub fn get_reverb_feedback_left_to_right(&self) -> u8 {
        self.reverb_feedback_left_to_right
    }
    pub fn get_reverb_feedback_right_to_right(&self) -> u8 {
        self.reverb_feedback_right_to_right
    }
    pub fn get_reverb_feedback_right_to_left(&self) -> u8 {
        self.reverb_feedback_right_to_left
    }
    pub fn get_premix_left_to_right(&self) -> u8 {
        self.premix_left_to_right
    }
    pub fn get_premix_right_to_left(&self) -> u8 {
        self.premix_right_to_left
    }
}

impl FrameDataBase<RVRB> for RVRB {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<RVRB> {
        let reverb_left = ::id3v2::bytes::to_u16(&readable.as_bytes(2)?);
        let reverb_right = ::id3v2::bytes::to_u16(&readable.as_bytes(2)?);
        let reverb_bounce_left = readable.as_bytes(1)?[0];
        let reverb_bounce_right = readable.as_bytes(1)?[0];
        let reverb_feedback_left_to_left = readable.as_bytes(1)?[0];
        let reverb_feedback_left_to_right = readable.as_bytes(1)?[0];
        let reverb_feedback_right_to_right = readable.as_bytes(1)?[0];
        let reverb_feedback_right_to_left = readable.as_bytes(1)?[0];
        let premix_left_to_right = readable.as_bytes(1)?[0];
        let premix_right_to_left = readable.as_bytes(1)?[0];

        let ret = Ok(RVRB {
            reverb_left: reverb_left,
            reverb_right: reverb_right,
            reverb_bounce_left: reverb_bounce_left,
            reverb_bounce_right: reverb_bounce_right,
            reverb_feedback_left_to_left: reverb_feedback_left_to_left,
            reverb_feedback_left_to_right: reverb_feedback_left_to_right,
            reverb_feedback_right_to_right: reverb_feedback_right_to_right,
            reverb_feedback_right_to_left: reverb_feedback_right_to_left,
            premix_left_to_right: premix_left_to_right,
            premix_right_to_left: premix_right_to_left
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Seek frame
#[derive(Debug)]
pub struct SEEK {
    next_tag: String
}

impl SEEK {
    pub fn get_next_tag(&self) -> &str {
        self.next_tag.as_str()
    }
}

impl FrameDataBase<SEEK> for SEEK {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<SEEK> {
        let next_tag = readable.all_string()?;

        let ret = Ok(SEEK {
            next_tag: next_tag
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Signature frame
#[derive(Debug)]
pub struct SIGN {
    group_symbol: u8,
    signature: vec::Vec<u8>
}

impl SIGN {
    pub fn get_group_symbol(&self) -> u8 {
        self.group_symbol
    }

    pub fn get_signature(&self) -> &[u8] {
        &self.signature
    }
}

impl FrameDataBase<SIGN> for SIGN {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<SIGN> {
        let group_symbol = readable.as_bytes(1)?[0];
        let signature = readable.all_bytes()?;

        let ret = Ok(SIGN {
            group_symbol: group_symbol,
            signature: signature
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Synchronised lyric/text
#[derive(Debug)]
pub struct SYLT {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    language: String,
    timestamp_format: ::id3v2::frame_constants::TimestampFormat,
    content_type: ::id3v2::frame_constants::ContentType,
    content_descriptor: String
}

impl SYLT {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_language(&self) -> &str {
        self.language.as_str()
    }

    pub fn get_timestamp_format(&self) -> &::id3v2::frame_constants::TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_content_type(&self) -> &::id3v2::frame_constants::ContentType {
        &self.content_type
    }

    pub fn get_content_descriptor(&self) -> &str {
        self.content_descriptor.as_str()
    }
}

impl FrameDataBase<SYLT> for SYLT {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<SYLT> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let timestamp_format = to_timestamp_format(readable.as_bytes(1)?[0]);
        let content_type = to_content_type(readable.as_bytes(1)?[0]);
        let (_, content_descriptor) = read_null_terminated(&text_encoding, readable)?;

        let ret = Ok(SYLT {
            text_encoding: text_encoding,
            language: language,
            timestamp_format: timestamp_format,
            content_type: content_type,
            content_descriptor: content_descriptor
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Synchronised tempo codes
#[derive(Debug)]
pub struct SYTC {
    timestamp_format: ::id3v2::frame_constants::TimestampFormat,
    tempo_data: vec::Vec<u8>
}

impl SYTC {
    pub fn get_timestamp_format(&self) -> &::id3v2::frame_constants::TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_temp_data(&self) -> &[u8] {
        &self.tempo_data
    }
}

impl FrameDataBase<SYTC> for SYTC {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<SYTC> {
        let timestamp_format = to_timestamp_format(readable.as_bytes(1)?[0]);
        let tempo_data = readable.all_bytes()?;

        let ret = Ok(SYTC {
            timestamp_format: timestamp_format,
            tempo_data: tempo_data
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Unique file identifier
#[derive(Debug)]
pub struct UFID {
    owner_identifier: String,
    identifier: vec::Vec<u8>
}

impl UFID {
    pub fn get_owner_identifier(&self) -> &str {
        self.owner_identifier.as_str()
    }

    pub fn get_identifier(&self) -> &[u8] {
        &self.identifier
    }
}

impl FrameDataBase<UFID> for UFID {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<UFID> {
        let (_, owner_identifier) = readable.read_terminated_null()?;
        let identifier = readable.all_bytes()?;

        let ret = Ok(UFID {
            owner_identifier: owner_identifier,
            identifier: identifier
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Terms of use
#[derive(Debug)]
pub struct USER {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    language: String,
    actual_text: String
}

impl USER {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_language(&self) -> &str {
        self.language.as_str()
    }

    pub fn get_actual_text(&self) -> &str {
        self.actual_text.as_str()
    }
}

impl FrameDataBase<USER> for USER {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<USER> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, actual_text) = read_null_terminated(&text_encoding, readable)?;

        let ret = Ok(USER {
            text_encoding: text_encoding,
            language: language,
            actual_text: actual_text
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// Unsynchronised lyric/text transcription
#[derive(Debug)]
pub struct USLT {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    language: String,
    content_descriptor: String,
    lyrics: String
}

impl USLT {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_language(&self) -> &str {
        self.language.as_str()
    }

    pub fn get_content_descriptor(&self) -> &str {
        self.content_descriptor.as_str()
    }

    pub fn get_lyrics(&self) -> &str {
        self.lyrics.as_str()
    }
}

impl FrameDataBase<USLT> for USLT {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<USLT> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, content_descriptor) = read_null_terminated(&text_encoding, readable)?;
        let (_, lyrics) = read_null_terminated(&text_encoding, readable)?;

        let ret = Ok(USLT {
            text_encoding: text_encoding,
            language: language,
            content_descriptor: content_descriptor,
            lyrics: lyrics
        });

        trace!("{:?}", ret);

        ret
    }
}

#[derive(Debug)]
pub struct TEXT {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    text: String
}

impl TEXT {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }
}

impl FrameDataBase<TEXT> for TEXT {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<TEXT> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let data = readable.all_bytes()?;
        let text = match as_encoded_text(&text_encoding, &data) {
            Err(::errors::ParsingError::EncodeDecodeError(_)) => {
                // The BPM is text frame but it is an integer and represented as a numerical string.
                if id == ::id3v2::frame_constants::id::TBP_STR || id == ::id3v2::frame_constants::id::TBPM_STR {
                    "0".to_string()
                } else {
                    "".to_string()
                }
            },
            Err(_) => "".to_string(),
            Ok(text) => trim_white_space_include_bom(text)
        };

        let ret = Ok(TEXT {
            text_encoding: text_encoding,
            text: text
        });

        trace!("{:?}", ret);

        ret
    }
}

#[derive(Debug)]
pub struct TXXX {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    description: String,
    value: String
}

impl TXXX {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<TXXX> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = read_null_terminated(&text_encoding, readable)?;
        let value = readable.all_string()?;

        let ret = Ok(TXXX {
            text_encoding: text_encoding,
            description: description,
            value: value
        });

        trace!("{:?}", ret);

        ret
    }
}

// TODO not yet tested!
// User defined URL link frame
#[derive(Debug)]
pub struct WXXX {
    text_encoding: ::id3v2::frame_constants::TextEncoding,
    description: String,
    url: String
}

impl WXXX {
    pub fn get_text_encoding(&self) -> &::id3v2::frame_constants::TextEncoding {
        &self.text_encoding
    }

    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }

    pub fn get_url(&self) -> &str {
        self.url.as_str()
    }
}

impl FrameDataBase<WXXX> for WXXX {
    fn to_framedata(readable: &mut Readable, id: &str) -> FrameResult<WXXX> {
        let text_encoding = to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = read_null_terminated(&text_encoding, readable)?;
        let url = readable.all_string()?;

        let ret = Ok(WXXX {
            text_encoding: text_encoding,
            description: description,
            url: url
        });

        trace!("{:?}", ret);

        ret
    }
}

#[derive(Debug)]
pub struct Frame {
    id: String,
    size: u32,
    data: vec::Vec<u8>,
    status_flag: u8,
    encoding_flag: u8,
    tag_version: u8
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
                let matched = is_valid_frame_id(&id);
                debug!("Frame.has_next_frame=> Frame Id:{}, matched: {}", id, matched);
                matched
            },
            Err(_) => {
                debug!("Frame.has_next_frame=> Fail");
                false
            }
        }
    }

    pub fn new<T>(readable: &mut ::readable::Readable<T>, tag_version: u8) -> FrameResult<Frame>
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

        trace!("Frame id: {}, size: {}, status_flag: {}, encoding_flag: {}", id, frame_size, header_bytes[8], header_bytes[9]);

        Ok(Frame {
            id: id,
            size: frame_size,
            data: body_bytes,
            // status_flag offset is 8
            status_flag: header_bytes[8],
            // encoding_flag offset is 9
            encoding_flag: header_bytes[9],
            // tag major version
            tag_version: tag_version
        })
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    // @see http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
    pub fn has_flag(&self, flag: ::id3v2::frame_constants::FrameHeaderFlag) -> bool {
        if self.tag_version == 3 {
            match flag {
                ::id3v2::frame_constants::FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 7 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 6 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 5 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 7 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 6 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 5 != 0,
                _ => false
            }
        } else if self.tag_version == 4 {
            match flag {
                ::id3v2::frame_constants::FrameHeaderFlag::TagAlter => self.status_flag & 0x01 << 6 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::FileAlter => self.status_flag & 0x01 << 5 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::ReadOnly => self.status_flag & 0x01 << 4 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::GroupIdentity => self.encoding_flag & 0x01 << 6 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::Compression => self.encoding_flag & 0x01 << 3 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::Encryption => self.encoding_flag & 0x01 << 2 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::Unsynchronisation => self.encoding_flag & 0x01 << 1 != 0,
                ::id3v2::frame_constants::FrameHeaderFlag::DataLength => self.encoding_flag & 0x01 != 0
            }
        } else {
            warn!("Frame.has_flag=> Unknown version!");
            false
        }
    }

    // @see http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
    pub fn get_data(&self) -> FrameResult<::id3v2::frame_constants::FrameData> {
        let mut readable = ::readable::factory::from_byte(self.data.clone())?;

        if self.tag_version == 2 {
            // Frame size
            readable.skip(3);
        }

        let id = self.id.as_str();

        Ok(match self.id.as_ref() {
            ::id3v2::frame_constants::id::BUF_STR => ::id3v2::frame_constants::FrameData::RBUF(RBUF::to_framedata(&mut readable, self.id.as_ref())?),
            ::id3v2::frame_constants::id::CNT_STR => ::id3v2::frame_constants::FrameData::PCNT(PCNT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::COM_STR => ::id3v2::frame_constants::FrameData::COMM(COMM::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::CRA_STR => ::id3v2::frame_constants::FrameData::AENC(AENC::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::CRM_STR => ::id3v2::frame_constants::FrameData::CRM(CRM::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::ETC_STR => ::id3v2::frame_constants::FrameData::ETCO(ETCO::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::EQU_STR => ::id3v2::frame_constants::FrameData::EQUA(EQUA::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::GEO_STR => ::id3v2::frame_constants::FrameData::GEOB(GEOB::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::IPL_STR => ::id3v2::frame_constants::FrameData::IPLS(IPLS::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::LNK_STR => ::id3v2::frame_constants::FrameData::LINK(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::MCI_STR => ::id3v2::frame_constants::FrameData::MCDI(MCDI::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::MLL_STR => ::id3v2::frame_constants::FrameData::MLLT(MLLT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::PIC_STR => ::id3v2::frame_constants::FrameData::PIC(PIC::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::POP_STR => ::id3v2::frame_constants::FrameData::POPM(POPM::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::REV_STR => ::id3v2::frame_constants::FrameData::RVRB(RVRB::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::RVA_STR => ::id3v2::frame_constants::FrameData::RVAD(RVA2::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::SLT_STR => ::id3v2::frame_constants::FrameData::SYLT(SYLT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::STC_STR => ::id3v2::frame_constants::FrameData::SYTC(SYTC::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TAL_STR => ::id3v2::frame_constants::FrameData::TALB(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TBP_STR => ::id3v2::frame_constants::FrameData::TBPM(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TCM_STR => ::id3v2::frame_constants::FrameData::TCOM(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TCO_STR => ::id3v2::frame_constants::FrameData::TCON(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TCR_STR => ::id3v2::frame_constants::FrameData::TCOP(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDA_STR => ::id3v2::frame_constants::FrameData::TDAT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDY_STR => ::id3v2::frame_constants::FrameData::TDLY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TEN_STR => ::id3v2::frame_constants::FrameData::TENC(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TFT_STR => ::id3v2::frame_constants::FrameData::TFLT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TIM_STR => ::id3v2::frame_constants::FrameData::TIME(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TKE_STR => ::id3v2::frame_constants::FrameData::TKEY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TLA_STR => ::id3v2::frame_constants::FrameData::TLAN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TLE_STR => ::id3v2::frame_constants::FrameData::TLEN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TMT_STR => ::id3v2::frame_constants::FrameData::TMED(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOA_STR => ::id3v2::frame_constants::FrameData::TOPE(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOF_STR => ::id3v2::frame_constants::FrameData::TOFN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOL_STR => ::id3v2::frame_constants::FrameData::TOLY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOR_STR => ::id3v2::frame_constants::FrameData::TORY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOT_STR => ::id3v2::frame_constants::FrameData::TOAL(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TP1_STR => ::id3v2::frame_constants::FrameData::TPE1(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TP2_STR => ::id3v2::frame_constants::FrameData::TPE2(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TP3_STR => ::id3v2::frame_constants::FrameData::TPE3(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TP4_STR => ::id3v2::frame_constants::FrameData::TPE4(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPA_STR => ::id3v2::frame_constants::FrameData::TPOS(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPB_STR => ::id3v2::frame_constants::FrameData::TPUB(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRC_STR => ::id3v2::frame_constants::FrameData::TRCK(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRD_STR => ::id3v2::frame_constants::FrameData::TRDA(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRK_STR => ::id3v2::frame_constants::FrameData::TRCK(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSI_STR => ::id3v2::frame_constants::FrameData::TSIZ(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSS_STR => ::id3v2::frame_constants::FrameData::TSSE(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TT1_STR => ::id3v2::frame_constants::FrameData::TIT1(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TT2_STR => ::id3v2::frame_constants::FrameData::TIT2(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TT3_STR => ::id3v2::frame_constants::FrameData::TIT3(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TXT_STR => ::id3v2::frame_constants::FrameData::TEXT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TXX_STR => ::id3v2::frame_constants::FrameData::TXXX(TXXX::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TYE_STR => ::id3v2::frame_constants::FrameData::TYER(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::UFI_STR => ::id3v2::frame_constants::FrameData::UFID(UFID::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::ULT_STR => ::id3v2::frame_constants::FrameData::USLT(USLT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WAF_STR => ::id3v2::frame_constants::FrameData::WOAF(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WAR_STR => ::id3v2::frame_constants::FrameData::WOAR(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WAS_STR => ::id3v2::frame_constants::FrameData::WOAS(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WCM_STR => ::id3v2::frame_constants::FrameData::WCOM(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WCP_STR => ::id3v2::frame_constants::FrameData::WCOP(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WPB_STR => ::id3v2::frame_constants::FrameData::WPUB(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WXX_STR => ::id3v2::frame_constants::FrameData::WXXX(WXXX::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::AENC_STR => ::id3v2::frame_constants::FrameData::AENC(AENC::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::APIC_STR => ::id3v2::frame_constants::FrameData::APIC(APIC::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::ASPI_STR => ::id3v2::frame_constants::FrameData::ASPI(ASPI::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::COMM_STR => ::id3v2::frame_constants::FrameData::COMM(COMM::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::COMR_STR => ::id3v2::frame_constants::FrameData::COMR(COMR::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::ENCR_STR => ::id3v2::frame_constants::FrameData::ENCR(ENCR::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::EQUA_STR => ::id3v2::frame_constants::FrameData::EQUA(EQUA::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::EQU2_STR => ::id3v2::frame_constants::FrameData::EQU2(EQU2::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::ETCO_STR => ::id3v2::frame_constants::FrameData::ETCO(ETCO::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::GEOB_STR => ::id3v2::frame_constants::FrameData::GEOB(GEOB::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::GRID_STR => ::id3v2::frame_constants::FrameData::GRID(GRID::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::IPLS_STR => ::id3v2::frame_constants::FrameData::IPLS(IPLS::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::LINK_STR => ::id3v2::frame_constants::FrameData::LINK(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::MCDI_STR => ::id3v2::frame_constants::FrameData::MCDI(MCDI::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::MLLT_STR => ::id3v2::frame_constants::FrameData::MLLT(MLLT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::OWNE_STR => ::id3v2::frame_constants::FrameData::OWNE(OWNE::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::PRIV_STR => ::id3v2::frame_constants::FrameData::PRIV(PRIV::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::PCNT_STR => ::id3v2::frame_constants::FrameData::PCNT(PCNT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::POPM_STR => ::id3v2::frame_constants::FrameData::POPM(POPM::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::POSS_STR => ::id3v2::frame_constants::FrameData::POSS(POSS::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::RBUF_STR => ::id3v2::frame_constants::FrameData::RBUF(RBUF::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::RVAD_STR => ::id3v2::frame_constants::FrameData::RVAD(RVA2::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::RVA2_STR => ::id3v2::frame_constants::FrameData::RVA2(RVA2::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::RVRB_STR => ::id3v2::frame_constants::FrameData::RVRB(RVRB::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::SEEK_STR => ::id3v2::frame_constants::FrameData::SEEK(SEEK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::SIGN_STR => ::id3v2::frame_constants::FrameData::SIGN(SIGN::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::SYLT_STR => ::id3v2::frame_constants::FrameData::SYLT(SYLT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::SYTC_STR => ::id3v2::frame_constants::FrameData::SYTC(SYTC::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::UFID_STR => ::id3v2::frame_constants::FrameData::UFID(UFID::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::USER_STR => ::id3v2::frame_constants::FrameData::USER(USER::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::USLT_STR => ::id3v2::frame_constants::FrameData::USLT(USLT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TALB_STR => ::id3v2::frame_constants::FrameData::TALB(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TBPM_STR => ::id3v2::frame_constants::FrameData::TBPM(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TCOM_STR => ::id3v2::frame_constants::FrameData::TCOM(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TCON_STR => ::id3v2::frame_constants::FrameData::TCON(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TCOP_STR => ::id3v2::frame_constants::FrameData::TCOP(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDAT_STR => ::id3v2::frame_constants::FrameData::TDAT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDEN_STR => ::id3v2::frame_constants::FrameData::TDEN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDLY_STR => ::id3v2::frame_constants::FrameData::TDLY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDOR_STR => ::id3v2::frame_constants::FrameData::TDOR(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDRC_STR => ::id3v2::frame_constants::FrameData::TDRC(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDRL_STR => ::id3v2::frame_constants::FrameData::TDRL(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TDTG_STR => ::id3v2::frame_constants::FrameData::TDTG(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TENC_STR => ::id3v2::frame_constants::FrameData::TENC(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TEXT_STR => ::id3v2::frame_constants::FrameData::TEXT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TIME_STR => ::id3v2::frame_constants::FrameData::TIME(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TFLT_STR => ::id3v2::frame_constants::FrameData::TFLT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TIPL_STR => ::id3v2::frame_constants::FrameData::TIPL(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TIT1_STR => ::id3v2::frame_constants::FrameData::TIT1(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TIT2_STR => ::id3v2::frame_constants::FrameData::TIT2(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TIT3_STR => ::id3v2::frame_constants::FrameData::TIT3(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TKEY_STR => ::id3v2::frame_constants::FrameData::TKEY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TLAN_STR => ::id3v2::frame_constants::FrameData::TLAN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TLEN_STR => ::id3v2::frame_constants::FrameData::TLEN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TMCL_STR => ::id3v2::frame_constants::FrameData::TMCL(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TMED_STR => ::id3v2::frame_constants::FrameData::TMED(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TMOO_STR => ::id3v2::frame_constants::FrameData::TMOO(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOAL_STR => ::id3v2::frame_constants::FrameData::TOAL(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOFN_STR => ::id3v2::frame_constants::FrameData::TOFN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOLY_STR => ::id3v2::frame_constants::FrameData::TOLY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOPE_STR => ::id3v2::frame_constants::FrameData::TOPE(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TORY_STR => ::id3v2::frame_constants::FrameData::TORY(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TOWN_STR => ::id3v2::frame_constants::FrameData::TOWN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPE1_STR => ::id3v2::frame_constants::FrameData::TPE1(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPE2_STR => ::id3v2::frame_constants::FrameData::TPE2(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPE3_STR => ::id3v2::frame_constants::FrameData::TPE3(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPE4_STR => ::id3v2::frame_constants::FrameData::TPE4(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPOS_STR => ::id3v2::frame_constants::FrameData::TPOS(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPRO_STR => ::id3v2::frame_constants::FrameData::TPRO(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TPUB_STR => ::id3v2::frame_constants::FrameData::TPUB(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRCK_STR => ::id3v2::frame_constants::FrameData::TRCK(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRDA_STR => ::id3v2::frame_constants::FrameData::TRDA(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRSN_STR => ::id3v2::frame_constants::FrameData::TRSN(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSIZ_STR => ::id3v2::frame_constants::FrameData::TSIZ(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TRSO_STR => ::id3v2::frame_constants::FrameData::TRSO(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSOA_STR => ::id3v2::frame_constants::FrameData::TSOA(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSOP_STR => ::id3v2::frame_constants::FrameData::TSOP(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSOT_STR => ::id3v2::frame_constants::FrameData::TSOT(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSRC_STR => ::id3v2::frame_constants::FrameData::TSRC(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSSE_STR => ::id3v2::frame_constants::FrameData::TSSE(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TYER_STR => ::id3v2::frame_constants::FrameData::TYER(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TSST_STR => ::id3v2::frame_constants::FrameData::TSST(TEXT::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::TXXX_STR => ::id3v2::frame_constants::FrameData::TXXX(TXXX::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WCOM_STR => ::id3v2::frame_constants::FrameData::WCOM(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WCOP_STR => ::id3v2::frame_constants::FrameData::WCOP(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WOAF_STR => ::id3v2::frame_constants::FrameData::WOAF(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WOAR_STR => ::id3v2::frame_constants::FrameData::WOAR(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WOAS_STR => ::id3v2::frame_constants::FrameData::WOAS(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WORS_STR => ::id3v2::frame_constants::FrameData::WORS(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WPAY_STR => ::id3v2::frame_constants::FrameData::WPAY(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WPUB_STR => ::id3v2::frame_constants::FrameData::WPUB(LINK::to_framedata(&mut readable, id)?),
            ::id3v2::frame_constants::id::WXXX_STR => ::id3v2::frame_constants::FrameData::WXXX(WXXX::to_framedata(&mut readable, id)?),
            _ => ::id3v2::frame_constants::FrameData::TEXT(self::TEXT::to_framedata(&mut readable, id)?)
        })
    }
}
