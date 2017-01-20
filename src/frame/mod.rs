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

pub trait FrameDataBase<T> {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<T>;
}

//Audio encryption
#[derive(Debug)]
pub struct AENC {
    owner_identifier: String,
    preview_start: u16,
    preview_end: u16,
    encryption_info: Vec<u8>
}

impl FrameDataBase<AENC> for AENC {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<AENC> {
        let (_, owner_identifier) = readable.non_utf16_string()?;
        Ok(AENC {
            owner_identifier: owner_identifier,
            preview_start: bytes::to_u16(&readable.as_bytes(2)?),
            preview_end: bytes::to_u16(&readable.as_bytes(2)?),
            encryption_info: readable.all_bytes()?
        })
    }
}

//Attached picture
#[derive(Debug)]
pub struct APIC {
    text_encoding: TextEncoding,
    mime_type: String,
    picture_type: PictureType,
    description: String,
    picture_data: Vec<u8>
}

impl FrameDataBase<APIC> for APIC {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<APIC> {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<ASPI> {
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
    text_encoding: TextEncoding,
    language: String,
    short_description: String,
    actual_text: String
}

impl COMM {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<COMM> {
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
    text_encoding: TextEncoding,
    price_string: String,
    // 8 bit long
    valid_util: String,
    contact_url: String,
    received_as: ReceivedAs,
    name_of_seller: String,
    description: String,
    picture_mime_type: String,
    seller_logo: Vec<u8>
}

impl COMR {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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

    pub fn get_received_as(&self) -> &ReceivedAs {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<COMR> {
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
    owner_identifier: String,
    method_symbol: u8,
    encryption_data: Vec<u8>
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<ENCR> {
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
    adjustment_bit: u8
}

impl EQUA {
    pub fn get_adjustment_bit(&self) -> u8 {
        self.adjustment_bit
    }
}

impl FrameDataBase<EQUA> for EQUA {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<EQUA> {
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
    interpolation_method: InterpolationMethod,
    identification: String
}

impl EQU2 {
    pub fn get_interpolation_method(&self) -> &InterpolationMethod {
        &self.interpolation_method
    }

    pub fn get_identification(&self) -> &str {
        self.identification.as_str()
    }
}

impl FrameDataBase<EQU2> for EQU2 {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<EQU2> {
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
    timestamp_format: TimestampFormat,
    event_timing_codes: Vec<EventTimingCode>
}

impl ETCO {
    pub fn get_timestamp_format(&self) -> &TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_event_timing_codes(&self) -> &[EventTimingCode] {
        &self.event_timing_codes
    }
}

impl FrameDataBase<ETCO> for ETCO {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<ETCO> {
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
    text_encoding: TextEncoding,
    mine_type: String,
    filename: String,
    content_description: String,
    encapsulation_object: Vec<u8>
}

impl GEOB {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<GEOB> {
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
    owner_identifier: String,
    group_symbol: u8,
    group_dependent_data: Vec<u8>
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<GRID> {
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
    text_encoding: TextEncoding,
    people_list_strings: String
}

impl IPLS {
    pub fn get_text_encoding(&self) -> &TextEncoding {
        &self.text_encoding
    }

    pub fn get_people_list_strings(&self) -> &str {
        self.people_list_strings.as_str()
    }
}

impl FrameDataBase<IPLS> for IPLS {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<IPLS> {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<LINK> {
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
    cd_toc: Vec<u8>
}

impl MCDI {
    pub fn get_cd_toc(&self) -> &[u8] {
        &self.cd_toc
    }
}

impl FrameDataBase<MCDI> for MCDI {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<MCDI> {
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
    data: Vec<u8>
}

impl MLLT {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl FrameDataBase<MLLT> for MLLT {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<MLLT> {
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
    text_encoding: TextEncoding,
    price_paid: String,
    // 8 bit long
    date_of_purch: String,
    seller: String
}

impl OWNE {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<OWNE> {
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
    owner_identifier: String,
    private_data: Vec<u8>
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<PRIV> {
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
    counter: u32
}

impl PCNT {
    pub fn get_counter(&self) -> u32 {
        self.counter
    }
}

impl FrameDataBase<PCNT> for PCNT {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<PCNT> {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<POPM> {
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
    timestamp_format: TimestampFormat,
    // TODO not yet implemented!
    position: Vec<u8>
}

impl POSS {
    pub fn get_timestamp_format(&self) -> &TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_position(&self) -> &[u8] {
        &self.position
    }
}

impl FrameDataBase<POSS> for POSS {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<POSS> {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<RBUF> {
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
    data: Vec<u8>
}

impl RVA2 {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl FrameDataBase<RVA2> for RVA2 {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<RVA2> {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<RVRB> {
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
    next_tag: String
}

impl SEEK {
    pub fn get_next_tag(&self) -> &str {
        self.next_tag.as_str()
    }
}

impl FrameDataBase<SEEK> for SEEK {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<SEEK> {
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
    group_symbol: u8,
    signature: Vec<u8>
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<SIGN> {
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
    text_encoding: TextEncoding,
    language: String,
    timestamp_format: TimestampFormat,
    content_type: ContentType,
    content_descriptor: String
}

impl SYLT {
    pub fn get_text_encoding(&self) -> &TextEncoding {
        &self.text_encoding
    }

    pub fn get_language(&self) -> &str {
        self.language.as_str()
    }

    pub fn get_timestamp_format(&self) -> &TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_content_type(&self) -> &ContentType {
        &self.content_type
    }

    pub fn get_content_descriptor(&self) -> &str {
        self.content_descriptor.as_str()
    }
}

impl FrameDataBase<SYLT> for SYLT {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<SYLT> {
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
    timestamp_format: TimestampFormat,
    tempo_data: Vec<u8>
}

impl SYTC {
    pub fn get_timestamp_format(&self) -> &TimestampFormat {
        &self.timestamp_format
    }

    pub fn get_temp_data(&self) -> &[u8] {
        &self.tempo_data
    }
}

impl FrameDataBase<SYTC> for SYTC {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<SYTC> {
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
    owner_identifier: String,
    identifier: Vec<u8>
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<UFID> {
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
    text_encoding: TextEncoding,
    language: String,
    actual_text: String
}

impl USER {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<USER> {
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
    text_encoding: TextEncoding,
    language: String,
    content_descriptor: String,
    lyrics: String
}

impl USLT {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<USLT> {
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
    text_encoding: TextEncoding,
    text: String
}

impl TEXT {
    pub fn get_text_encoding(&self) -> &TextEncoding {
        &self.text_encoding
    }

    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }
}

impl FrameDataBase<TEXT> for TEXT {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<TEXT> {
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
    text_encoding: TextEncoding,
    description: String,
    value: String
}

impl TXXX {
    pub fn get_text_encoding(&self) -> &TextEncoding {
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
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<TXXX> {
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
    text_encoding: TextEncoding,
    description: String,
    url: String
}

impl FrameDataBase<WXXX> for WXXX {
    fn to_framedata(readable: &mut Readable, id: &str) -> Result<WXXX> {
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