extern crate encoding;
extern crate regex;

pub mod constants;
mod util;

use self::encoding::{Encoding, DecoderTrap};
use std::vec::Vec;
use std::io::{Result, Cursor, Error, ErrorKind};

use bytes;
use frame::constants::{
    id,
    PictureType,
    ReceivedAs,
    InterpolationMethod,
    ContentType,
    TimestampFormat,
    EventTimingCode,
    FrameHeaderFlag,
    TextEncoding
};

type Readable = ::readable::Readable<Cursor<Vec<u8>>>;

pub trait FrameDefault<T> {
    fn read(readable: &mut Readable, id: &str) -> Result<T>;
}

// TODO not yet tested!
// Recommended buffer size
#[derive(Debug)]
pub struct BUF {
    pub buffer_size: u32,
    pub embedded_info_flag: u8,
    pub offset_to_next_tag: u32
}

impl FrameDefault<BUF> for BUF {
    fn read(readable: &mut Readable, id: &str) -> Result<BUF> {
        let buffer_size = bytes::to_u32(&readable.as_bytes(3)?);
        let embedded_info_flag = readable.as_bytes(1)?[0];
        let offset_to_next_tag = bytes::to_u32(&readable.as_bytes(4)?);

        Ok(BUF {
            buffer_size: buffer_size,
            embedded_info_flag: embedded_info_flag,
            offset_to_next_tag: offset_to_next_tag
        })
    }
}

// TODO not yet tested!
// Encrypted meta frame
#[derive(Debug)]
pub struct CRM {
    pub owner_identifier: String,
    pub content: String,
    pub encrypted_datablock: Vec<u8>
}

impl FrameDefault<CRM> for CRM {
    fn read(readable: &mut Readable, id: &str) -> Result<CRM> {
        let (_, owner_identifier) = readable.non_utf16_string()?;
        let (_, content) = readable.non_utf16_string()?;
        let encrypted_datablock = readable.all_bytes()?;

        Ok(CRM {
            owner_identifier: owner_identifier,
            content: content,
            encrypted_datablock: encrypted_datablock
        })
    }
}

// TODO not yet tested!
// Audio encryption
#[derive(Debug)]
pub struct AENC {
    pub owner_identifier: String,
    pub preview_start: u16,
    pub preview_end: u16,
    pub encryption_info: Vec<u8>
}

impl FrameDefault<AENC> for AENC {
    fn read(readable: &mut Readable, id: &str) -> Result<AENC> {
        let (_, owner_identifier) = readable.non_utf16_string()?;

        Ok(AENC {
            owner_identifier: owner_identifier,
            preview_start: bytes::to_u16(&readable.as_bytes(2)?),
            preview_end: bytes::to_u16(&readable.as_bytes(2)?),
            encryption_info: readable.all_bytes()?
        })
    }
}

// TODO not yet tested!
// Attached picture
#[derive(Debug)]
pub struct APIC {
    pub text_encoding: TextEncoding,
    pub mime_type: String,
    pub picture_type: PictureType,
    pub description: String,
    pub picture_data: Vec<u8>
}

impl FrameDefault<APIC> for APIC {
    fn read(readable: &mut Readable, id: &str) -> Result<APIC> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, mine_type) = readable.non_utf16_string()?;
        let picture_type = util::to_picture_type(readable.as_bytes(1)?[0]);
        let (_, description) = util::read_null_terminated(&text_encoding, readable)?;
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

// TODO not yet tested!
// Audio seek point index
#[derive(Debug)]
pub struct ASPI {
    pub indexed_data_start: u32,
    pub indexed_data_length: u32,
    pub number_of_index_points: u16,
    pub bit_per_index_point: u8,
    pub fraction_at_index: u8
}

impl FrameDefault<ASPI> for ASPI {
    fn read(readable: &mut Readable, id: &str) -> Result<ASPI> {
        let indexed_data_start = bytes::to_u32(&readable.as_bytes(4)?);
        let indexed_data_length = bytes::to_u32(&readable.as_bytes(4)?);
        let number_of_index_points = bytes::to_u16(&readable.as_bytes(2)?);
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
    pub text_encoding: TextEncoding,
    pub language: String,
    pub short_description: String,
    pub actual_text: String
}

impl FrameDefault<COMM> for COMM {
    fn read(readable: &mut Readable, id: &str) -> Result<COMM> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, short_description) = util::read_null_terminated(&text_encoding, readable)?;
        let actual_text = readable.all_string()?;

        Ok(COMM {
            text_encoding: text_encoding,
            language: language,
            short_description: short_description,
            actual_text: actual_text
        })
    }
}

// TODO not yet tested!
// Commercial frame
#[derive(Debug)]
pub struct COMR {
    pub text_encoding: TextEncoding,
    pub price_string: String,
    // 8 bit long
    pub valid_util: String,
    pub contact_url: String,
    pub received_as: ReceivedAs,
    pub name_of_seller: String,
    pub description: String,
    pub picture_mime_type: String,
    pub seller_logo: Vec<u8>
}

impl FrameDefault<COMR> for COMR {
    fn read(readable: &mut Readable, id: &str) -> Result<COMR> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, price_string) = readable.non_utf16_string()?;
        let valid_util = readable.as_string(8)?;
        let (_, contact_url) = readable.non_utf16_string()?;
        let received_as = util::to_received_as(readable.as_bytes(1)?[0]);
        let (_, name_of_seller) = readable.utf16_string()?;
        let (_, description) = readable.utf16_string()?;
        let (_, picture_mime_type) = readable.non_utf16_string()?;
        let seller_logo = readable.all_bytes()?;

        Ok(COMR {
            text_encoding: text_encoding,
            price_string: price_string,
            valid_util: valid_util,
            contact_url: contact_url,
            received_as: received_as,
            name_of_seller: name_of_seller,
            description: description,
            picture_mime_type: picture_mime_type,
            seller_logo: seller_logo
        })
    }
}

// TODO not yet tested!
// Encryption method registration
#[derive(Debug)]
pub struct ENCR {
    pub owner_identifier: String,
    pub method_symbol: u8,
    pub encryption_data: Vec<u8>
}

impl FrameDefault<ENCR> for ENCR {
    fn read(readable: &mut Readable, id: &str) -> Result<ENCR> {
        let (_, owner_identifier) = readable.non_utf16_string()?;
        let method_symbol = readable.as_bytes(1)?[0];
        let encryption_data = readable.all_bytes()?;

        Ok(ENCR {
            owner_identifier: owner_identifier,
            method_symbol: method_symbol,
            encryption_data: encryption_data
        })
    }
}

// TODO not yet tested!
// Equalisation
#[derive(Debug)]
pub struct EQUA {
    pub adjustment_bit: u8
}

impl FrameDefault<EQUA> for EQUA {
    fn read(readable: &mut Readable, id: &str) -> Result<EQUA> {
        let adjustment_bit = readable.as_bytes(1)?[0];

        Ok(EQUA {
            adjustment_bit: adjustment_bit
        })
    }
}

// TODO not yet tested!
// Equalisation (2)
#[derive(Debug)]
pub struct EQU2 {
    pub interpolation_method: InterpolationMethod,
    pub identification: String
}

impl FrameDefault<EQU2> for EQU2 {
    fn read(readable: &mut Readable, id: &str) -> Result<EQU2> {
        let interpolation_method = util::to_interpolation_method(readable.as_bytes(1)?[0]);
        let (_, identification) = readable.non_utf16_string()?;

        Ok(EQU2 {
            interpolation_method: interpolation_method,
            identification: identification
        })
    }
}

// Event timing codes
#[derive(Debug)]
pub struct ETCO {
    pub timestamp_format: TimestampFormat,
    pub event_timing_codes: Vec<EventTimingCode>
}

impl FrameDefault<ETCO> for ETCO {
    fn read(readable: &mut Readable, id: &str) -> Result<ETCO> {
        let timestamp_format = util::to_timestamp_format(readable.as_bytes(1)?[0]);
        let mut event_timing_codes: Vec<EventTimingCode> = Vec::new();
        loop {
            let mut is_break = true;
            if let Ok(code_type) = readable.as_bytes(1) {
                if let Ok(timestamp) = readable.as_bytes(4) {
                    let event_timing_code = util::to_event_timing_code(code_type[0], bytes::to_u32(&timestamp));
                    event_timing_codes.push(event_timing_code);
                    is_break = false;
                }
            }

            if is_break {
                break;
            }
        }

        Ok(ETCO {
            timestamp_format: timestamp_format,
            event_timing_codes: event_timing_codes
        })
    }
}

// TODO not yet tested!
// General encapsulated object
#[derive(Debug)]
pub struct GEOB {
    pub text_encoding: TextEncoding,
    pub mine_type: String,
    pub filename: String,
    pub content_description: String,
    pub encapsulation_object: Vec<u8>
}

impl FrameDefault<GEOB> for GEOB {
    fn read(readable: &mut Readable, id: &str) -> Result<GEOB> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, mine_type) = readable.non_utf16_string()?;
        let (_, filename) = readable.utf16_string()?;
        let (_, content_description) = readable.utf16_string()?;
        let encapsulation_object = readable.all_bytes()?;

        Ok(GEOB {
            text_encoding: text_encoding,
            mine_type: mine_type,
            filename: filename,
            content_description: content_description,
            encapsulation_object: encapsulation_object
        })
    }
}

// TODO not yet tested!
// Group identification registration
#[derive(Debug)]
pub struct GRID {
    pub owner_identifier: String,
    pub group_symbol: u8,
    pub group_dependent_data: Vec<u8>
}

impl FrameDefault<GRID> for GRID {
    fn read(readable: &mut Readable, id: &str) -> Result<GRID> {
        let (_, owner_identifier) = readable.non_utf16_string()?;
        let group_symbol = readable.as_bytes(1)?[0];
        let group_dependent_data = readable.all_bytes()?;

        Ok(GRID {
            owner_identifier: owner_identifier,
            group_symbol: group_symbol,
            group_dependent_data: group_dependent_data
        })
    }
}

#[derive(Debug)]
// Involved people list
pub struct IPLS {
    pub text_encoding: TextEncoding,
    pub people_list_strings: String
}

impl FrameDefault<IPLS> for IPLS {
    fn read(readable: &mut Readable, id: &str) -> Result<IPLS> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, people_list_strings) = util::read_null_terminated(&text_encoding, readable)?;

        Ok(IPLS {
            text_encoding: text_encoding,
            people_list_strings: people_list_strings
        })
    }
}

// TODO not yet tested!
// Linked information
#[derive(Debug)]
pub struct LINK {
    pub frame_identifier: u32,
    pub url: String,
    pub additional_data: String
}

impl FrameDefault<LINK> for LINK {
    fn read(readable: &mut Readable, id: &str) -> Result<LINK> {
        let frame_id = bytes::to_u32(&readable.as_bytes(4)?);
        let (_, url) = readable.non_utf16_string()?;
        let additional_data = readable.all_string()?;
        Ok(LINK {
            frame_identifier: frame_id,
            url: url,
            additional_data: additional_data
        })
    }
}

// TODO not yet tested!
// Music CD identifier
#[derive(Debug)]
pub struct MCDI {
    pub cd_toc: Vec<u8>
}

impl FrameDefault<MCDI> for MCDI {
    fn read(readable: &mut Readable, id: &str) -> Result<MCDI> {
        let cd_toc = readable.all_bytes()?;

        Ok(MCDI {
            cd_toc: cd_toc
        })
    }
}

// TODO not yet tested!
// TODO not yet implemented!
// MPEG location lookup table
#[derive(Debug)]
pub struct MLLT {
    pub data: Vec<u8>
}

impl FrameDefault<MLLT> for MLLT {
    fn read(readable: &mut Readable, id: &str) -> Result<MLLT> {
        let data = readable.all_bytes()?;

        Ok(MLLT {
            data: data
        })
    }
}

// TODO not yet tested!
// Ownership frame
#[derive(Debug)]
pub struct OWNE {
    pub text_encoding: TextEncoding,
    pub price_paid: String,
    // 8 bit long
    pub date_of_purch: String,
    pub seller: String
}

impl FrameDefault<OWNE> for OWNE {
    fn read(readable: &mut Readable, id: &str) -> Result<OWNE> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, price_paid) = readable.non_utf16_string()?;
        let date_of_purch = readable.as_string(4)?;
        let (_, seller) = util::read_null_terminated(&text_encoding, readable)?;

        Ok(OWNE {
            text_encoding: text_encoding,
            price_paid: price_paid,
            date_of_purch: date_of_purch,
            seller: seller
        })
    }
}

// TODO not yet tested!
// Private frame
#[derive(Debug)]
pub struct PRIV {
    pub owner_identifier: String,
    pub private_data: Vec<u8>
}

impl FrameDefault<PRIV> for PRIV {
    fn read(readable: &mut Readable, id: &str) -> Result<PRIV> {
        let (_, owner_identifier) = readable.non_utf16_string()?;
        let private_data = readable.all_bytes()?;

        Ok(PRIV {
            owner_identifier: owner_identifier,
            private_data: private_data
        })
    }
}

// NOTE it support that only the 32-bit unsigned integer type.
// Play counter
#[derive(Debug)]
pub struct PCNT {
    pub counter: u32
}

impl FrameDefault<PCNT> for PCNT {
    fn read(readable: &mut Readable, id: &str) -> Result<PCNT> {
        let mut all_bytes = readable.all_bytes()?;
        let counter = util::trim_to_u32(&mut all_bytes);

        Ok(PCNT {
            counter: counter
        })
    }
}

// TODO not yet tested!
// Popularimeter
#[derive(Debug)]
pub struct POPM {
    pub email_to_user: String,
    pub rating: u8,
    // NOTE it support that only the 32-bit unsigned integer type.
    pub counter: u32
}

impl FrameDefault<POPM> for POPM {
    fn read(readable: &mut Readable, id: &str) -> Result<POPM> {
        let (_, email_to_user) = readable.non_utf16_string()?;
        let rating = readable.as_bytes(1)?[0];
        let counter = {
            let mut all_bytes = readable.all_bytes()?;
            util::trim_to_u32(&mut all_bytes)
        };

        Ok(POPM {
            email_to_user: email_to_user,
            rating: rating,
            counter: counter
        })
    }
}

// TODO not yet tested!
// Position synchronisation frame
#[derive(Debug)]
pub struct POSS {
    pub timestamp_format: TimestampFormat,
    // TODO not yet implemented!
    pub position: Vec<u8>
}

impl FrameDefault<POSS> for POSS {
    fn read(readable: &mut Readable, id: &str) -> Result<POSS> {
        let timestamp_format = util::to_timestamp_format(readable.as_bytes(1)?[0]);
        let position = readable.all_bytes()?;

        Ok(POSS {
            timestamp_format: timestamp_format,
            position: position
        })
    }
}

// TODO not yet tested!
// Recommended buffer size
#[derive(Debug)]
pub struct RBUF {
    pub buffer_size: u32,
    pub embedded_info_flag: u8,
    pub offset_to_next_tag: u32
}

impl FrameDefault<RBUF> for RBUF {
    fn read(readable: &mut Readable, id: &str) -> Result<RBUF> {
        let buffer_size = bytes::to_u32(&readable.as_bytes(3)?);
        let embedded_info_flag = readable.as_bytes(1)?[0] & 0x01;
        let offset_to_next_tag = bytes::to_u32(&readable.as_bytes(4)?);

        Ok(RBUF {
            buffer_size: buffer_size,
            embedded_info_flag: embedded_info_flag,
            offset_to_next_tag: offset_to_next_tag
        })
    }
}

// TODO not yet tested!
// TODO not yet implemented!
// Relative volume adjustment (2)
#[derive(Debug)]
pub struct RVA2 {
    pub data: Vec<u8>
}

impl FrameDefault<RVA2> for RVA2 {
    fn read(readable: &mut Readable, id: &str) -> Result<RVA2> {
        let data = readable.all_bytes()?;

        Ok(RVA2 {
            data: data
        })
    }
}

// TODO not yet tested!
// Reverb
#[derive(Debug)]
pub struct RVRB {
    pub reverb_left: u16,
    pub reverb_right: u16,
    pub reverb_bounce_left: u8,
    pub reverb_bounce_right: u8,
    pub reverb_feedback_left_to_left: u8,
    pub reverb_feedback_left_to_right: u8,
    pub reverb_feedback_right_to_right: u8,
    pub reverb_feedback_right_to_left: u8,
    pub premix_left_to_right: u8,
    pub premix_right_to_left: u8
}

impl FrameDefault<RVRB> for RVRB {
    fn read(readable: &mut Readable, id: &str) -> Result<RVRB> {
        let reverb_left = bytes::to_u16(&readable.as_bytes(2)?);
        let reverb_right = bytes::to_u16(&readable.as_bytes(2)?);
        let reverb_bounce_left = readable.as_bytes(1)?[0];
        let reverb_bounce_right = readable.as_bytes(1)?[0];
        let reverb_feedback_left_to_left = readable.as_bytes(1)?[0];
        let reverb_feedback_left_to_right = readable.as_bytes(1)?[0];
        let reverb_feedback_right_to_right = readable.as_bytes(1)?[0];
        let reverb_feedback_right_to_left = readable.as_bytes(1)?[0];
        let premix_left_to_right = readable.as_bytes(1)?[0];
        let premix_right_to_left = readable.as_bytes(1)?[0];

        Ok(RVRB {
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
        })
    }
}

// TODO not yet tested!
// Seek frame
#[derive(Debug)]
pub struct SEEK {
    pub next_tag: String
}

impl FrameDefault<SEEK> for SEEK {
    fn read(readable: &mut Readable, id: &str) -> Result<SEEK> {
        let next_tag = readable.all_string()?;

        Ok(SEEK {
            next_tag: next_tag
        })
    }
}

// TODO not yet tested!
// Signature frame
#[derive(Debug)]
pub struct SIGN {
    pub group_symbol: u8,
    pub signature: Vec<u8>
}

impl FrameDefault<SIGN> for SIGN {
    fn read(readable: &mut Readable, id: &str) -> Result<SIGN> {
        let group_symbol = readable.as_bytes(1)?[0];
        let signature = readable.all_bytes()?;

        Ok(SIGN {
            group_symbol: group_symbol,
            signature: signature
        })
    }
}

// TODO not yet tested!
// Synchronised lyric/text
#[derive(Debug)]
pub struct SYLT {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub timestamp_format: TimestampFormat,
    pub content_type: ContentType,
    pub content_descriptor: String
}

impl FrameDefault<SYLT> for SYLT {
    fn read(readable: &mut Readable, id: &str) -> Result<SYLT> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let timestamp_format = util::to_timestamp_format(readable.as_bytes(1)?[0]);
        let content_type = util::to_content_type(readable.as_bytes(1)?[0]);
        let (_, content_descriptor) = util::read_null_terminated(&text_encoding, readable)?;

        Ok(SYLT {
            text_encoding: text_encoding,
            language: language,
            timestamp_format: timestamp_format,
            content_type: content_type,
            content_descriptor: content_descriptor
        })
    }
}

// TODO not yet tested!
// Synchronised tempo codes
#[derive(Debug)]
pub struct SYTC {
    pub timestamp_format: TimestampFormat,
    pub tempo_data: Vec<u8>
}

impl FrameDefault<SYTC> for SYTC {
    fn read(readable: &mut Readable, id: &str) -> Result<SYTC> {
        let timestamp_format = util::to_timestamp_format(readable.as_bytes(1)?[0]);
        let tempo_data = readable.all_bytes()?;

        Ok(SYTC {
            timestamp_format: timestamp_format,
            tempo_data: tempo_data
        })
    }
}

// TODO not yet tested!
// Unique file identifier
#[derive(Debug)]
pub struct UFID {
    pub owner_identifier: String,
    pub identifier: Vec<u8>
}

impl FrameDefault<UFID> for UFID {
    fn read(readable: &mut Readable, id: &str) -> Result<UFID> {
        let (_, owner_identifier) = readable.non_utf16_string()?;
        let identifier = readable.all_bytes()?;

        Ok(UFID {
            owner_identifier: owner_identifier,
            identifier: identifier
        })
    }
}

// TODO not yet tested!
// Terms of use
#[derive(Debug)]
pub struct USER {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub actual_text: String
}

impl FrameDefault<USER> for USER {
    fn read(readable: &mut Readable, id: &str) -> Result<USER> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, actual_text) = util::read_null_terminated(&text_encoding, readable)?;

        Ok(USER {
            text_encoding: text_encoding,
            language: language,
            actual_text: actual_text
        })
    }
}

// TODO not yet tested!
// Unsynchronised lyric/text transcription
#[derive(Debug)]
pub struct USLT {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub content_descriptor: String,
    pub lyrics: String
}

impl FrameDefault<USLT> for USLT {
    fn read(readable: &mut Readable, id: &str) -> Result<USLT> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let language = readable.as_string(3)?;
        let (_, content_descriptor) = util::read_null_terminated(&text_encoding, readable)?;
        let (_, lyrics) = util::read_null_terminated(&text_encoding, readable)?;

        Ok(USLT {
            text_encoding: text_encoding,
            language: language,
            content_descriptor: content_descriptor,
            lyrics: lyrics
        })
    }
}

#[derive(Debug)]
pub struct TEXT {
    pub text_encoding: TextEncoding,
    pub text: String
}

impl FrameDefault<TEXT> for TEXT {
    fn read(readable: &mut Readable, id: &str) -> Result<TEXT> {
        fn _default(id: &str, decode: ::std::result::Result<String, ::std::borrow::Cow<'static, str>>) -> String {
            match decode {
                Ok(text) => text,
                Err(e) => {
                    if id == id::TBPM_STR {
                        "0".to_string()
                    } else {
                        "".to_string()
                    }
                }
            }
        }

        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let data = readable.all_bytes()?;
        let text = match text_encoding {
            TextEncoding::ISO8859_1 => _default(id, encoding::all::ISO_8859_1.decode(&data, encoding::DecoderTrap::Strict)),
            TextEncoding::UTF16LE => _default(id, encoding::all::UTF_16LE.decode(&data, encoding::DecoderTrap::Strict)),
            TextEncoding::UTF16BE => _default(id, encoding::all::UTF_16BE.decode(&data, encoding::DecoderTrap::Strict)),
            TextEncoding::UTF8 => _default(id, encoding::all::UTF_8.decode(&data, encoding::DecoderTrap::Strict))
        };

        // TODO const
        let re = regex::Regex::new(r"(^[\x{0}|\x{feff}|\x{fffe}]*|[\x{0}|\x{feff}|\x{fffe}]*$)").unwrap();
        let text = text.trim();
        let text = re.replace_all(text, "").into_owned();

        Ok(TEXT {
            text_encoding: text_encoding,
            text: text
        })
    }
}

#[derive(Debug)]
pub struct TXXX {
    pub text_encoding: TextEncoding,
    pub description: String,
    pub value: String
}

impl FrameDefault<TXXX> for TXXX {
    fn read(readable: &mut Readable, id: &str) -> Result<TXXX> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = util::read_null_terminated(&text_encoding, readable)?;
        let value = readable.all_string()?;
        Ok(TXXX {
            text_encoding: text_encoding,
            description: description,
            value: value
        })
    }
}

// TODO not yet tested!
// User defined URL link frame
#[derive(Debug)]
pub struct WXXX {
    pub text_encoding: TextEncoding,
    pub description: String,
    pub url: String
}

impl FrameDefault<WXXX> for WXXX {
    fn read(readable: &mut Readable, id: &str) -> Result<WXXX> {
        let text_encoding = bytes::to_encoding(readable.as_bytes(1)?[0]);
        let (_, description) = util::read_null_terminated(&text_encoding, readable)?;
        let url = readable.all_string()?;
        Ok(WXXX {
            text_encoding: text_encoding,
            description: description,
            url: url
        })
    }
}