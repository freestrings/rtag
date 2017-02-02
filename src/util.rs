extern crate encoding;

use self::encoding::all::ISO_8859_1;
use self::encoding::{
    Encoding,
    DecoderTrap
};

use frame::{
    PictureType,
    ReceivedAs,
    InterpolationMethod,
    ContentType,
    TimestampFormat,
    EventTimingCode,
    TextEncoding
};
use readable::Readable;

use std::io::{
    Cursor,
    Result
};
use std::vec::Vec;

pub const BIT7: u8 = 0x80;
pub const BIT6: u8 = 0x40;
pub const BIT5: u8 = 0x20;
pub const BIT4: u8 = 0x10;
pub const BIT3: u8 = 0x08;
pub const BIT2: u8 = 0x04;
pub const BIT1: u8 = 0x02;
pub const BIT0: u8 = 0x01;

pub fn to_picture_type(t: u8) -> PictureType {
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
        0x0a => PictureType::Band,
        0x0b => PictureType::Composer,
        0x0c => PictureType::Lyricist,
        0x0d => PictureType::RecordingLocation,
        0x0e => PictureType::DuringRecording,
        0x0f => PictureType::DuringPerformance,
        0x10 => PictureType::MovieScreenCapture,
        0x11 => PictureType::BrightColouredFish,
        0x12 => PictureType::Illustration,
        0x13 => PictureType::BandLogotype,
        0x14 => PictureType::PublisherLogoType,
        _ => PictureType::Other
    }
}

pub fn to_received_as(t: u8) -> ReceivedAs {
    match t {
        0x00 => ReceivedAs::Other,
        0x01 => ReceivedAs::StandardCDAlbum,
        0x02 => ReceivedAs::CompressedAudioOnCD,
        0x03 => ReceivedAs::FileOverInternet,
        0x04 => ReceivedAs::StreamOverInternet,
        0x05 => ReceivedAs::AsNoteSheets,
        0x06 => ReceivedAs::AsNoteSheetsInBook,
        0x07 => ReceivedAs::MusicOnMedia,
        0x08 => ReceivedAs::NonMusicalMerchandise,
        _ => ReceivedAs::Other
    }
}

pub fn to_interpolation_method(t: u8) -> InterpolationMethod {
    match t {
        0x00 => InterpolationMethod::Band,
        0x01 => InterpolationMethod::Linear,
        _ => InterpolationMethod::Band
    }
}

pub fn to_timestamp_format(t: u8) -> TimestampFormat {
    match t {
        0x01 => TimestampFormat::MpecFrames,
        0x02 => TimestampFormat::Milliseconds,
        _ => TimestampFormat::MpecFrames
    }
}

pub fn to_event_timing_code(t: u8, timestamp: u32) -> EventTimingCode {
    match t {
        0x00 => EventTimingCode::Padding(timestamp),
        0x01 => EventTimingCode::EndOfInitialSilence(timestamp),
        0x02 => EventTimingCode::IntroStart(timestamp),
        0x03 => EventTimingCode::MainPartStart(timestamp),
        0x04 => EventTimingCode::OutroStart(timestamp),
        0x05 => EventTimingCode::OutroEnd(timestamp),
        0x06 => EventTimingCode::VerseStart(timestamp),
        0x07 => EventTimingCode::RefrainStart(timestamp),
        0x08 => EventTimingCode::InterludeStart(timestamp),
        0x09 => EventTimingCode::ThemeStart(timestamp),
        0x0a => EventTimingCode::VariationStart(timestamp),
        0x0b => EventTimingCode::KeyChange(timestamp),
        0x0c => EventTimingCode::TimeChange(timestamp),
        0x0d => EventTimingCode::MomentaryUnwantedNoise(timestamp),
        0x0e => EventTimingCode::SustainedNoise(timestamp),
        0x0f => EventTimingCode::SustainedNoiseEnd(timestamp),
        0x10 => EventTimingCode::IntroEnd(timestamp),
        0x11 => EventTimingCode::MainPartEnd(timestamp),
        0x12 => EventTimingCode::VerseEnd(timestamp),
        0x13 => EventTimingCode::RefrainEnd(timestamp),
        0x14 => EventTimingCode::ThemeEnd(timestamp),
        0x15 => EventTimingCode::Profanity(timestamp),
        0x16 => EventTimingCode::ProfanityEnd(timestamp),
        0x17 ... 0xdf => EventTimingCode::ReservedForFutureUse(timestamp),
        0xe0 ... 0xef => EventTimingCode::NotPredefinedSynch(timestamp),
        0xf0 ... 0xfc => EventTimingCode::ReservedForFutureUse(timestamp),
        0xfd => EventTimingCode::AudioEnd(timestamp),
        0xfe => EventTimingCode::AudioFileEnds(timestamp),
        0xff => EventTimingCode::OneMoreByteOfEventsFollows(timestamp),
        _ => EventTimingCode::Padding(timestamp)
    }
}

pub fn read_null_terminated(text_encoding: &TextEncoding,
                            readable: &mut Readable<Cursor<Vec<u8>>>)
                            -> Result<String> {
    Ok(match text_encoding {
        &TextEncoding::ISO88591 |
        &TextEncoding::UTF8 =>
            readable.non_utf16_string()?,

        _ => readable.utf16_string()?
    })
}

pub fn to_content_type(t: u8) -> ContentType {
    match t {
        0x00 => ContentType::Other,
        0x01 => ContentType::Lyrics,
        0x02 => ContentType::TextTranscription,
        0x03 => ContentType::MovementName,
        0x04 => ContentType::Events,
        0x05 => ContentType::Chord,
        0x06 => ContentType::Trivia,
        0x07 => ContentType::UrlsToWebpages,
        0x08 => ContentType::UrlsToImages,
        _ => ContentType::Other
    }
}

pub fn to_encoding(encoding: u8) -> TextEncoding {
    match encoding {
        0 => TextEncoding::ISO88591,
        1 => TextEncoding::UTF16LE,
        2 => TextEncoding::UTF16BE,
        3 => TextEncoding::UTF8,
        _ => TextEncoding::ISO88591
    }
}

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

pub fn to_iso8859_1(bytes: &Vec<u8>) -> String {
    match ISO_8859_1.decode(&bytes, DecoderTrap::Strict) {
        Ok(value) => value.to_string(),
        _ => "".to_string()
    }
}