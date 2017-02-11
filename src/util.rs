extern crate encoding;

use self::encoding::all::ISO_8859_1;
use self::encoding::{Encoding, DecoderTrap, EncoderTrap};

use frame::{PictureType, ReceivedAs, InterpolationMethod, ContentType, TimestampFormat,
            EventTimingCode, TextEncoding};
use readable::Readable;
use writable::Writable;

use std::io::{Cursor, Result};
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
        _ => PictureType::Other,
    }
}

pub fn from_picture_type(t: &PictureType) -> u8 {
    match t {
        &PictureType::Other => 0x00,
        &PictureType::FileIcon => 0x01,
        &PictureType::OtherFileIcon => 0x02,
        &PictureType::CoverFront => 0x03,
        &PictureType::CoverBack => 0x04,
        &PictureType::LeafletPage => 0x05,
        &PictureType::Media => 0x06,
        &PictureType::LeadArtist => 0x07,
        &PictureType::Artist => 0x08,
        &PictureType::Conductor => 0x09,
        &PictureType::Band => 0x0a,
        &PictureType::Composer => 0x0b,
        &PictureType::Lyricist => 0x0c,
        &PictureType::RecordingLocation => 0x0d,
        &PictureType::DuringRecording => 0x0e,
        &PictureType::DuringPerformance => 0x0f,
        &PictureType::MovieScreenCapture => 0x10,
        &PictureType::BrightColouredFish => 0x11,
        &PictureType::Illustration => 0x12,
        &PictureType::BandLogotype => 0x13,
        &PictureType::PublisherLogoType => 0x14,
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
        _ => ReceivedAs::Other,
    }
}

pub fn from_received_as(t: &ReceivedAs) -> u8 {
    match t {
        &ReceivedAs::Other => 0x00,
        &ReceivedAs::StandardCDAlbum => 0x01,
        &ReceivedAs::CompressedAudioOnCD => 0x02,
        &ReceivedAs::FileOverInternet => 0x03,
        &ReceivedAs::StreamOverInternet => 0x04,
        &ReceivedAs::AsNoteSheets => 0x05,
        &ReceivedAs::AsNoteSheetsInBook => 0x06,
        &ReceivedAs::MusicOnMedia => 0x07,
        &ReceivedAs::NonMusicalMerchandise => 0x08,
    }
}

pub fn to_interpolation_method(t: u8) -> InterpolationMethod {
    match t {
        0x00 => InterpolationMethod::Band,
        0x01 => InterpolationMethod::Linear,
        _ => InterpolationMethod::Band,
    }
}

pub fn from_interpolation_method(t: &InterpolationMethod) -> u8 {
    match t {
        &InterpolationMethod::Band => 0x00,
        &InterpolationMethod::Linear => 0x01,
    }
}

pub fn to_timestamp_format(t: u8) -> TimestampFormat {
    match t {
        0x01 => TimestampFormat::MpecFrames,
        0x02 => TimestampFormat::Milliseconds,
        _ => TimestampFormat::MpecFrames,
    }
}

pub fn from_timestamp_format(t: &TimestampFormat) -> u8 {
    match t {
        &TimestampFormat::MpecFrames => 0x01,
        &TimestampFormat::Milliseconds => 0x02,
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
        0x17...0xdf => EventTimingCode::ReservedForFutureUse(timestamp, t),
        0xe0...0xef => EventTimingCode::NotPredefinedSynch(timestamp, t),
        0xf0...0xfc => EventTimingCode::ReservedForFutureUse(timestamp, t),
        0xfd => EventTimingCode::AudioEnd(timestamp),
        0xfe => EventTimingCode::AudioFileEnds(timestamp),
        0xff => EventTimingCode::OneMoreByteOfEventsFollows(timestamp),
        _ => EventTimingCode::Padding(timestamp),
    }
}

pub fn from_event_timing_code(e: &EventTimingCode) -> (u8, u32) {
    match e {
        &EventTimingCode::Padding(timestamp) => (0x00, timestamp),
        &EventTimingCode::EndOfInitialSilence(timestamp) => (0x01, timestamp),
        &EventTimingCode::IntroStart(timestamp) => (0x02, timestamp),
        &EventTimingCode::MainPartStart(timestamp) => (0x03, timestamp),
        &EventTimingCode::OutroStart(timestamp) => (0x04, timestamp),
        &EventTimingCode::OutroEnd(timestamp) => (0x05, timestamp),
        &EventTimingCode::VerseStart(timestamp) => (0x06, timestamp),
        &EventTimingCode::RefrainStart(timestamp) => (0x07, timestamp),
        &EventTimingCode::InterludeStart(timestamp) => (0x08, timestamp),
        &EventTimingCode::ThemeStart(timestamp) => (0x09, timestamp),
        &EventTimingCode::VariationStart(timestamp) => (0x0a, timestamp),
        &EventTimingCode::KeyChange(timestamp) => (0x0b, timestamp),
        &EventTimingCode::TimeChange(timestamp) => (0x0c, timestamp),
        &EventTimingCode::MomentaryUnwantedNoise(timestamp) => (0x0d, timestamp),
        &EventTimingCode::SustainedNoise(timestamp) => (0x0e, timestamp),
        &EventTimingCode::SustainedNoiseEnd(timestamp) => (0x0f, timestamp),
        &EventTimingCode::IntroEnd(timestamp) => (0x10, timestamp),
        &EventTimingCode::MainPartEnd(timestamp) => (0x11, timestamp),
        &EventTimingCode::VerseEnd(timestamp) => (0x12, timestamp),
        &EventTimingCode::RefrainEnd(timestamp) => (0x13, timestamp),
        &EventTimingCode::ThemeEnd(timestamp) => (0x14, timestamp),
        &EventTimingCode::Profanity(timestamp) => (0x15, timestamp),
        &EventTimingCode::ProfanityEnd(timestamp) => (0x16, timestamp),
        &EventTimingCode::ReservedForFutureUse(timestamp, t) => {
            if (0x17 <= t && t < 0xdf) || (0xf0 <= t && t < 0xfc) {
                (t, timestamp)
            } else {
                (0x17, timestamp)
            }
        }
        &EventTimingCode::NotPredefinedSynch(timestamp, t) => {
            if 0xe0 <= t && t < 0xef {
                (t, timestamp)
            } else {
                (0xe0, timestamp)
            }
        }
        &EventTimingCode::AudioEnd(timestamp) => (0xfd, timestamp),
        &EventTimingCode::AudioFileEnds(timestamp) => (0xfe, timestamp),
        &EventTimingCode::OneMoreByteOfEventsFollows(timestamp) => (0xff, timestamp),
    }
}

pub fn read_null_terminated(text_encoding: &TextEncoding,
                            readable: &mut Readable<Cursor<Vec<u8>>>)
                            -> Result<String> {
    Ok(match text_encoding {
        &TextEncoding::ISO88591 |
        &TextEncoding::UTF8 => readable.non_utf16_string()?,
        _ => readable.utf16_string()?,
    })
}

pub fn write_null_terminated(text_encoding: &TextEncoding,
                             text: &str,
                             writable: &mut Writable<Cursor<Vec<u8>>>)
                             -> Result<()> {
    Ok(match text_encoding {
        &TextEncoding::ISO88591 |
        &TextEncoding::UTF8 => writable.non_utf16_string(text)?,
        _ => writable.utf16_string(text)?,
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
        _ => ContentType::Other,
    }
}

pub fn from_content_type(t: &ContentType) -> u8 {
    match t {
        &ContentType::Other => 0x00,
        &ContentType::Lyrics => 0x01,
        &ContentType::TextTranscription => 0x02,
        &ContentType::MovementName => 0x03,
        &ContentType::Events => 0x04,
        &ContentType::Chord => 0x05,
        &ContentType::Trivia => 0x06,
        &ContentType::UrlsToWebpages => 0x07,
        &ContentType::UrlsToImages => 0x08,
    }
}

pub fn to_encoding(encoding: u8) -> TextEncoding {
    match encoding {
        0 => TextEncoding::ISO88591,
        1 => TextEncoding::UTF16LE,
        2 => TextEncoding::UTF16BE,
        3 => TextEncoding::UTF8,
        _ => TextEncoding::ISO88591,
    }
}

pub fn from_encoding(encoding: &TextEncoding) -> u8 {
    match encoding {
        &TextEncoding::ISO88591 => 0,
        &TextEncoding::UTF16LE => 1,
        &TextEncoding::UTF16BE => 2,
        &TextEncoding::UTF8 => 3,
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

pub fn to_unsynchronize(bytes: &Vec<u8>) -> Vec<u8> {
    fn require_unsync(bytes: &Vec<u8>) -> usize {
        let mut count = 0;
        let len = bytes.len();
        for i in 0..len - 1 {
            if bytes[i] & 0xff == 0xff && (bytes[i + 1] & 0xe0 == 0xe0 || bytes[i + 1] == 0) {
                count = count + 1;
            }
        }
        if len > 0 && bytes[len - 1] == 0xff {
            count = count + 1;
        }
        count
    }

    let count = require_unsync(bytes);
    if count == 0 {
        return bytes.clone();
    }

    let len = bytes.len();
    let mut out = vec![0u8; len + count];
    let mut j = 0;
    for i in 0..len - 1 {
        out[j] = bytes[i];
        j = j + 1;
        if bytes[i] & 0xff == 0xff && (bytes[i + 1] & 0xe0 == 0xe0 || bytes[i + 1] == 0) {
            out[j] = 0;
            j = j + 1;
        }
    }
    out[j] = bytes[len - 1];
    j = j + 1;
    if bytes[len - 1] == 0xff {
        out[j] = 0;
    }

    out
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
        _ => "".to_string(),
    }
}

pub fn from_iso8859_1(v: &String, len: usize) -> Vec<u8> {
    let mut v = match ISO_8859_1.encode(&v, EncoderTrap::Strict) {
        Ok(value) => value,
        _ => vec![0u8; len],
    };

    for i in v.len()..len {
        v[i] = 0;
    }
    v.to_vec()
}