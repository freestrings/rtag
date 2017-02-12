extern crate encoding;

use self::encoding::all::ISO_8859_1;
use self::encoding::{Encoding, DecoderTrap, EncoderTrap};

use errors::*;
use frame::*;
use frame::id::*;
use readable::Readable;
use writable::{Writable, WritableFactory};

use std::collections::HashMap;
use std::io::{Cursor, Result};
use std::result;
use std::vec::Vec;

pub const BIT7: u8 = 0x80;
pub const BIT6: u8 = 0x40;
pub const BIT5: u8 = 0x20;
pub const BIT4: u8 = 0x10;
pub const BIT3: u8 = 0x08;
pub const BIT2: u8 = 0x04;
pub const BIT1: u8 = 0x02;
pub const BIT0: u8 = 0x01;

lazy_static! {
    pub static ref ID_V2_V4: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(id::BUF_STR, id::RBUF_STR);
        m.insert(id::CNT_STR, id::PCNT_STR);
        m.insert(id::COM_STR, id::COMM_STR);
        m.insert(id::CRA_STR, id::AENC_STR);
        m.insert(id::ETC_STR, id::ETCO_STR);
        m.insert(id::EQU_STR, id::EQUA_STR);
        m.insert(id::GEO_STR, id::GEOB_STR);
        m.insert(id::IPL_STR, id::IPLS_STR);
        m.insert(id::LNK_STR, id::LINK_STR);
        m.insert(id::MCI_STR, id::MCDI_STR);
        m.insert(id::MLL_STR, id::MLLT_STR);
        m.insert(id::POP_STR, id::POPM_STR);
        m.insert(id::REV_STR, id::RVRB_STR);
        m.insert(id::RVA_STR, id::RVAD_STR);
        m.insert(id::SLT_STR, id::SYLT_STR);
        m.insert(id::STC_STR, id::SYTC_STR);
        m.insert(id::TAL_STR, id::TALB_STR);
        m.insert(id::TBP_STR, id::TBPM_STR);
        m.insert(id::TCM_STR, id::TCOM_STR);
        m.insert(id::TCO_STR, id::TCON_STR);
        m.insert(id::TCR_STR, id::TCOP_STR);
        m.insert(id::TDA_STR, id::TDAT_STR);
        m.insert(id::TDY_STR, id::TDLY_STR);
        m.insert(id::TEN_STR, id::TENC_STR);
        m.insert(id::TFT_STR, id::TFLT_STR);
        m.insert(id::TIM_STR, id::TIME_STR);
        m.insert(id::TKE_STR, id::TKEY_STR);
        m.insert(id::TLA_STR, id::TLAN_STR);
        m.insert(id::TLE_STR, id::TLEN_STR);
        m.insert(id::TMT_STR, id::TMED_STR);
        m.insert(id::TOA_STR, id::TOPE_STR);
        m.insert(id::TOF_STR, id::TOFN_STR);
        m.insert(id::TOL_STR, id::TOLY_STR);
        m.insert(id::TOR_STR, id::TORY_STR);
        m.insert(id::TOT_STR, id::TOAL_STR);
        m.insert(id::TP1_STR, id::TPE1_STR);
        m.insert(id::TP2_STR, id::TPE2_STR);
        m.insert(id::TP3_STR, id::TPE3_STR);
        m.insert(id::TP4_STR, id::TPE4_STR);
        m.insert(id::TPA_STR, id::TPOS_STR);
        m.insert(id::TPB_STR, id::TPUB_STR);
        m.insert(id::TRC_STR, id::TSRC_STR);
        m.insert(id::TRD_STR, id::TRDA_STR);
        m.insert(id::TRK_STR, id::TRCK_STR);
        m.insert(id::TSI_STR, id::TSIZ_STR);
        m.insert(id::TSS_STR, id::TSSE_STR);
        m.insert(id::TT1_STR, id::TIT1_STR);
        m.insert(id::TT2_STR, id::TIT2_STR);
        m.insert(id::TT3_STR, id::TIT1_STR);
        m.insert(id::TXT_STR, id::TEXT_STR);
        m.insert(id::TXX_STR, id::TXXX_STR);
        m.insert(id::TYE_STR, id::TYER_STR);
        m.insert(id::UFI_STR, id::UFID_STR);
        m.insert(id::ULT_STR, id::USLT_STR);
        m.insert(id::WAF_STR, id::WOAF_STR);
        m.insert(id::WAR_STR, id::WOAR_STR);
        m.insert(id::WAS_STR, id::WOAS_STR);
        m.insert(id::WCM_STR, id::WCOM_STR);
        m.insert(id::WCP_STR, id::WCOP_STR);
        m.insert(id::WPB_STR, id::WPUB_STR);
        m.insert(id::WXX_STR, id::WXXX_STR);
        
        m
    };
}

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

pub fn frame_body_to_id(frame_body: &FrameBody, version: u8) -> &'static str {
    match frame_body {
        &FrameBody::BUF(_) => {
            if version == 2 {
                id::BUF_STR
            } else {
                id::RBUF_STR
            }
        }
        &FrameBody::CRM(_) => id::CRM_STR,
        &FrameBody::PIC(_) => id::PIC_STR,
        &FrameBody::AENC(_) => {
            if version == 2 {
                id::CRA_STR
            } else {
                id::AENC_STR
            }
        }
        &FrameBody::APIC(_) => id::APIC_STR,
        &FrameBody::ASPI(_) => id::ASPI_STR,
        &FrameBody::COMM(_) => {
            if version == 2 {
                id::COM_STR
            } else {
                id::COMM_STR
            }
        }
        &FrameBody::COMR(_) => id::COMR_STR,
        &FrameBody::ENCR(_) => id::ENCR_STR,
        &FrameBody::EQUA(_) => {
            if version == 2 {
                id::EQU_STR
            } else {
                id::EQUA_STR
            }
        }
        &FrameBody::EQU2(_) => id::EQU2_STR,
        &FrameBody::ETCO(_) => {
            if version == 2 {
                id::ETC_STR
            } else {
                id::ETCO_STR
            }
        }
        &FrameBody::GEOB(_) => {
            if version == 2 {
                id::GEO_STR
            } else {
                id::GEOB_STR
            }
        }
        &FrameBody::GRID(_) => id::GRID_STR,
        &FrameBody::IPLS(_) => {
            if version == 2 {
                id::IPL_STR
            } else {
                id::IPLS_STR
            }
        }
        &FrameBody::LINK(_) => {
            if version == 2 {
                id::LNK_STR
            } else {
                id::LINK_STR
            }
        }
        &FrameBody::MCDI(_) => {
            if version == 2 {
                id::MCI_STR
            } else {
                id::MCDI_STR
            }
        }
        &FrameBody::MLLT(_) => {
            if version == 2 {
                id::MLL_STR
            } else {
                id::MLLT_STR
            }
        }
        &FrameBody::OWNE(_) => id::OWNE_STR,
        &FrameBody::PRIV(_) => id::PRIV_STR,
        &FrameBody::PCNT(_) => {
            if version == 2 {
                id::CNT_STR
            } else {
                id::PCNT_STR
            }
        }
        &FrameBody::POPM(_) => {
            if version == 2 {
                id::POP_STR
            } else {
                id::POPM_STR
            }
        }
        &FrameBody::POSS(_) => id::POSS_STR,
        &FrameBody::RBUF(_) => id::RBUF_STR,
        &FrameBody::RVAD(_) => {
            if version == 2 {
                id::RVA_STR
            } else {
                id::RVAD_STR
            }
        }
        &FrameBody::RVA2(_) => id::RVA2_STR,
        &FrameBody::RVRB(_) => {
            if version == 2 {
                id::REV_STR
            } else {
                id::RVRB_STR
            }
        }
        &FrameBody::SEEK(_) => id::SEEK_STR,
        &FrameBody::SIGN(_) => id::SIGN_STR,
        &FrameBody::SYLT(_) => {
            if version == 2 {
                id::SLT_STR
            } else {
                id::SYLT_STR
            }
        }
        &FrameBody::SYTC(_) => {
            if version == 2 {
                id::STC_STR
            } else {
                id::SYTC_STR
            }
        }
        &FrameBody::TALB(_) => {
            if version == 2 {
                id::TAL_STR
            } else {
                id::TALB_STR
            }
        }
        &FrameBody::TBPM(_) => {
            if version == 2 {
                id::TBP_STR
            } else {
                id::TBPM_STR
            }
        }
        &FrameBody::TCOM(_) => {
            if version == 2 {
                id::TCM_STR
            } else {
                id::TCOM_STR
            }
        }
        &FrameBody::TCON(_) => {
            if version == 2 {
                id::TCO_STR
            } else {
                id::TCON_STR
            }
        }
        &FrameBody::TCOP(_) => {
            if version == 2 {
                id::TCR_STR
            } else {
                id::TCOP_STR
            }
        }
        &FrameBody::TDAT(_) => {
            if version == 2 {
                id::TDA_STR
            } else {
                id::TDAT_STR
            }
        }
        &FrameBody::TDEN(_) => id::TDEN_STR,
        &FrameBody::TDLY(_) => {
            if version == 2 {
                id::TDY_STR
            } else {
                id::TDLY_STR
            }
        }
        &FrameBody::TDOR(_) => id::TDOR_STR,
        &FrameBody::TDRC(_) => id::TDRC_STR,
        &FrameBody::TDRL(_) => id::TDRL_STR,
        &FrameBody::TDTG(_) => id::TDTG_STR,
        &FrameBody::TENC(_) => {
            if version == 2 {
                id::TEN_STR
            } else {
                id::TENC_STR
            }
        }
        &FrameBody::TEXT(_) => {
            if version == 2 {
                id::TXT_STR
            } else {
                id::TEXT_STR
            }
        }
        &FrameBody::TFLT(_) => {
            if version == 2 {
                id::TFT_STR
            } else {
                id::TFLT_STR
            }
        }
        &FrameBody::TIME(_) => {
            if version == 2 {
                id::TIM_STR
            } else {
                id::TIME_STR
            }
        }
        &FrameBody::TIPL(_) => id::TIPL_STR,
        &FrameBody::TIT1(_) => {
            if version == 2 {
                id::TT1_STR
            } else {
                id::TIT1_STR
            }
        }
        &FrameBody::TIT2(_) => {
            if version == 2 {
                id::TT2_STR
            } else {
                id::TIT2_STR
            }
        }
        &FrameBody::TIT3(_) => {
            if version == 2 {
                id::TT3_STR
            } else {
                id::TIT3_STR
            }
        }
        &FrameBody::TKEY(_) => {
            if version == 2 {
                id::TKE_STR
            } else {
                id::TKEY_STR
            }
        }
        &FrameBody::TLAN(_) => {
            if version == 2 {
                id::TLA_STR
            } else {
                id::TLAN_STR
            }
        }
        &FrameBody::TLEN(_) => {
            if version == 2 {
                id::TLE_STR
            } else {
                id::TLEN_STR
            }
        }
        &FrameBody::TMCL(_) => id::TMCL_STR,
        &FrameBody::TMED(_) => {
            if version == 2 {
                id::TMT_STR
            } else {
                id::TMED_STR
            }
        }
        &FrameBody::TMOO(_) => id::TMOO_STR,
        &FrameBody::TOAL(_) => {
            if version == 2 {
                id::TOT_STR
            } else {
                id::TOAL_STR
            }
        }
        &FrameBody::TOFN(_) => {
            if version == 2 {
                id::TOF_STR
            } else {
                id::TOFN_STR
            }
        }
        &FrameBody::TOLY(_) => {
            if version == 2 {
                id::TOL_STR
            } else {
                id::TOLY_STR
            }
        }
        &FrameBody::TOPE(_) => {
            if version == 2 {
                id::TOA_STR
            } else {
                id::TOPE_STR
            }
        }
        &FrameBody::TORY(_) => {
            if version == 2 {
                id::TOR_STR
            } else {
                id::TORY_STR
            }
        }
        &FrameBody::TOWN(_) => id::TOWN_STR,
        &FrameBody::TPE1(_) => {
            if version == 2 {
                id::TP1_STR
            } else {
                id::TPE1_STR
            }
        }
        &FrameBody::TPE2(_) => {
            if version == 2 {
                id::TP2_STR
            } else {
                id::TPE2_STR
            }
        }
        &FrameBody::TPE3(_) => {
            if version == 2 {
                id::TP3_STR
            } else {
                id::TPE3_STR
            }
        }
        &FrameBody::TPE4(_) => {
            if version == 2 {
                id::TP4_STR
            } else {
                id::TPE4_STR
            }
        }
        &FrameBody::TPOS(_) => {
            if version == 2 {
                id::TPA_STR
            } else {
                id::TPOS_STR
            }
        }
        &FrameBody::TPRO(_) => id::TPRO_STR,
        &FrameBody::TPUB(_) => {
            if version == 2 {
                id::TPB_STR
            } else {
                id::TPUB_STR
            }
        }
        &FrameBody::TRCK(_) => {
            if version == 2 {
                id::TRK_STR
            } else {
                id::TRCK_STR
            }
        }
        &FrameBody::TRDA(_) => {
            if version == 2 {
                id::TRD_STR
            } else {
                id::TRDA_STR
            }
        }
        &FrameBody::TRSN(_) => id::TRSN_STR,
        &FrameBody::TRSO(_) => id::TRSO_STR,
        &FrameBody::TSIZ(_) => {
            if version == 2 {
                id::TSI_STR
            } else {
                id::TSIZ_STR
            }
        }
        &FrameBody::TSOA(_) => id::TSOA_STR,
        &FrameBody::TSOP(_) => id::TSOP_STR,
        &FrameBody::TSOT(_) => id::TSOT_STR,
        &FrameBody::TSRC(_) => {
            if version == 2 {
                id::TRC_STR
            } else {
                id::TSRC_STR
            }
        }
        &FrameBody::TSSE(_) => {
            if version == 2 {
                id::TSS_STR
            } else {
                id::TSSE_STR
            }
        }
        &FrameBody::TYER(_) => {
            if version == 2 {
                id::TYE_STR
            } else {
                id::TYER_STR
            }
        }
        &FrameBody::TSST(_) => id::TSST_STR,
        &FrameBody::TXXX(_) => {
            if version == 2 {
                id::TXX_STR
            } else {
                id::TXXX_STR
            }
        }
        &FrameBody::UFID(_) => {
            if version == 2 {
                id::UFI_STR
            } else {
                id::UFID_STR
            }
        }
        &FrameBody::USER(_) => id::USER_STR,
        &FrameBody::USLT(_) => {
            if version == 2 {
                id::ULT_STR
            } else {
                id::USLT_STR
            }
        }
        &FrameBody::WCOM(_) => {
            if version == 2 {
                id::WCM_STR
            } else {
                id::WCOM_STR
            }
        }
        &FrameBody::WCOP(_) => {
            if version == 2 {
                id::WCP_STR
            } else {
                id::WCOP_STR
            }
        }
        &FrameBody::WOAF(_) => {
            if version == 2 {
                id::WAF_STR
            } else {
                id::WOAF_STR
            }
        }
        &FrameBody::WOAR(_) => {
            if version == 2 {
                id::WAR_STR
            } else {
                id::WOAR_STR
            }
        }
        &FrameBody::WOAS(_) => {
            if version == 2 {
                id::WAS_STR
            } else {
                id::WOAS_STR
            }
        }
        &FrameBody::WORS(_) => id::WORS_STR,
        &FrameBody::WPAY(_) => id::WPAY_STR,
        &FrameBody::WPUB(_) => {
            if version == 2 {
                id::WPB_STR
            } else {
                id::WPUB_STR
            }
        }
        &FrameBody::WXXX(_) => {
            if version == 2 {
                id::WXX_STR
            } else {
                id::WXXX_STR
            }
        }
        _ => "",

    }
}

pub fn frame_body_as_bytes(frame_body: &FrameBody,
                        version: u8)
                        -> result::Result<(&str, Vec<u8>), WriteError> {
    let mut writable = Cursor::new(vec![0u8; 0]).to_writable();

    match frame_body {
        &FrameBody::BUF(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::CRM(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::PIC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::AENC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::APIC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::ASPI(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::COMM(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::COMR(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::ENCR(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::EQUA(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::EQU2(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::ETCO(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::GEOB(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::GRID(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::IPLS(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::LINK(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::MCDI(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::MLLT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::OWNE(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::PRIV(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::PCNT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::POPM(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::POSS(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::RBUF(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::RVAD(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::RVA2(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::RVRB(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::SEEK(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::SIGN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::SYLT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::SYTC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TALB(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TBPM(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TCOM(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TCON(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TCOP(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDAT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDEN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDLY(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDOR(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDRC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDRL(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TDTG(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TENC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TEXT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TFLT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TIME(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TIPL(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TIT1(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TIT2(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TIT3(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TKEY(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TLAN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TLEN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TMCL(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TMED(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TMOO(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TOAL(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TOFN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TOLY(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TOPE(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TORY(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TOWN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPE1(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPE2(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPE3(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPE4(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPOS(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPRO(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TPUB(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TRCK(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TRDA(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TRSN(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TRSO(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSIZ(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSOA(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSOP(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSOT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSRC(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSSE(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TYER(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TSST(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::TXXX(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::UFID(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::USER(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::USLT(ref frame) => {
            frame.write(&mut writable)?;
        }
        &FrameBody::WCOM(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WCOP(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WOAF(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WOAR(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WOAS(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WORS(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WPAY(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WPUB(ref frame) => {
            frame.write(&mut writable, version)?;
        }
        &FrameBody::WXXX(ref frame) => {
            frame.write(&mut writable)?;
        }
        _ => (),
    };

    let id = frame_body_to_id(frame_body, version);

    let mut buf = Vec::new();
    writable.copy(&mut buf)?;

    Ok((id, buf))
}

pub fn id_to_frame_body(id: &str,
              version: u8,
              mut readable: Readable<Cursor<Vec<u8>>>)
              -> result::Result<FrameBody, ParsingError> {
    let frame_body = match id.as_ref() {
        BUF_STR => FrameBody::BUF(RBUF::read(&mut readable)?),
        CNT_STR => FrameBody::PCNT(PCNT::read(&mut readable)?),
        COM_STR => FrameBody::COMM(COMM::read(&mut readable)?),
        CRA_STR => FrameBody::AENC(AENC::read(&mut readable)?),
        CRM_STR => FrameBody::CRM(CRM::read(&mut readable)?),
        ETC_STR => FrameBody::ETCO(ETCO::read(&mut readable)?),
        EQU_STR => FrameBody::EQUA(EQUA::read(&mut readable)?),
        GEO_STR => FrameBody::GEOB(GEOB::read(&mut readable)?),
        IPL_STR => FrameBody::IPLS(IPLS::read(&mut readable)?),
        LNK_STR => FrameBody::LINK(LINK::read(&mut readable, version)?),
        MCI_STR => FrameBody::MCDI(MCDI::read(&mut readable)?),
        MLL_STR => FrameBody::MLLT(MLLT::read(&mut readable)?),
        PIC_STR => FrameBody::PIC(PIC::read(&mut readable)?),
        POP_STR => FrameBody::POPM(POPM::read(&mut readable)?),
        REV_STR => FrameBody::RVRB(RVRB::read(&mut readable)?),
        RVA_STR => FrameBody::RVAD(RVA2::read(&mut readable)?),
        SLT_STR => FrameBody::SYLT(SYLT::read(&mut readable)?),
        STC_STR => FrameBody::SYTC(SYTC::read(&mut readable)?),
        TAL_STR => FrameBody::TALB(TEXT::read(&mut readable, id)?),
        TBP_STR => FrameBody::TBPM(TEXT::read(&mut readable, id)?),
        TCM_STR => FrameBody::TCOM(TEXT::read(&mut readable, id)?),
        TCO_STR => FrameBody::TCON(TEXT::read(&mut readable, id)?),
        TCR_STR => FrameBody::TCOP(TEXT::read(&mut readable, id)?),
        TDA_STR => FrameBody::TDAT(TEXT::read(&mut readable, id)?),
        TDY_STR => FrameBody::TDLY(TEXT::read(&mut readable, id)?),
        TEN_STR => FrameBody::TENC(TEXT::read(&mut readable, id)?),
        TFT_STR => FrameBody::TFLT(TEXT::read(&mut readable, id)?),
        TIM_STR => FrameBody::TIME(TEXT::read(&mut readable, id)?),
        TKE_STR => FrameBody::TKEY(TEXT::read(&mut readable, id)?),
        TLA_STR => FrameBody::TLAN(TEXT::read(&mut readable, id)?),
        TLE_STR => FrameBody::TLEN(TEXT::read(&mut readable, id)?),
        TMT_STR => FrameBody::TMED(TEXT::read(&mut readable, id)?),
        TOA_STR => FrameBody::TMED(TEXT::read(&mut readable, id)?),
        TOF_STR => FrameBody::TOFN(TEXT::read(&mut readable, id)?),
        TOL_STR => FrameBody::TOLY(TEXT::read(&mut readable, id)?),
        TOR_STR => FrameBody::TORY(TEXT::read(&mut readable, id)?),
        TOT_STR => FrameBody::TOAL(TEXT::read(&mut readable, id)?),
        TP1_STR => FrameBody::TPE1(TEXT::read(&mut readable, id)?),
        TP2_STR => FrameBody::TPE2(TEXT::read(&mut readable, id)?),
        TP3_STR => FrameBody::TPE3(TEXT::read(&mut readable, id)?),
        TP4_STR => FrameBody::TPE4(TEXT::read(&mut readable, id)?),
        TPA_STR => FrameBody::TPOS(TEXT::read(&mut readable, id)?),
        TPB_STR => FrameBody::TPUB(TEXT::read(&mut readable, id)?),
        TRC_STR => FrameBody::TSRC(TEXT::read(&mut readable, id)?),
        TRD_STR => FrameBody::TRDA(TEXT::read(&mut readable, id)?),
        TRK_STR => FrameBody::TRCK(TEXT::read(&mut readable, id)?),
        TSI_STR => FrameBody::TSIZ(TEXT::read(&mut readable, id)?),
        TSS_STR => FrameBody::TSSE(TEXT::read(&mut readable, id)?),
        TT1_STR => FrameBody::TIT1(TEXT::read(&mut readable, id)?),
        TT2_STR => FrameBody::TIT2(TEXT::read(&mut readable, id)?),
        TT3_STR => FrameBody::TIT3(TEXT::read(&mut readable, id)?),
        TXT_STR => FrameBody::TEXT(TEXT::read(&mut readable, id)?),
        TYE_STR => FrameBody::TYER(TEXT::read(&mut readable, id)?),
        TXX_STR => FrameBody::TXXX(TXXX::read(&mut readable)?),
        UFI_STR => FrameBody::UFID(UFID::read(&mut readable)?),
        ULT_STR => FrameBody::USLT(USLT::read(&mut readable)?),
        WAF_STR => FrameBody::WOAF(LINK::read(&mut readable, version)?),
        WAR_STR => FrameBody::WOAR(LINK::read(&mut readable, version)?),
        WAS_STR => FrameBody::WOAS(LINK::read(&mut readable, version)?),
        WCM_STR => FrameBody::WCOM(LINK::read(&mut readable, version)?),
        WCP_STR => FrameBody::WCOP(LINK::read(&mut readable, version)?),
        WPB_STR => FrameBody::WPUB(LINK::read(&mut readable, version)?),
        WXX_STR => FrameBody::WXXX(WXXX::read(&mut readable)?),
        AENC_STR => FrameBody::AENC(AENC::read(&mut readable)?),
        APIC_STR => FrameBody::APIC(APIC::read(&mut readable)?),
        ASPI_STR => FrameBody::ASPI(ASPI::read(&mut readable)?),
        COMM_STR => FrameBody::COMM(COMM::read(&mut readable)?),
        COMR_STR => FrameBody::COMR(COMR::read(&mut readable)?),
        ENCR_STR => FrameBody::ENCR(ENCR::read(&mut readable)?),
        EQUA_STR => FrameBody::EQUA(EQUA::read(&mut readable)?),
        EQU2_STR => FrameBody::EQU2(EQU2::read(&mut readable)?),
        ETCO_STR => FrameBody::ETCO(ETCO::read(&mut readable)?),
        GEOB_STR => FrameBody::GEOB(GEOB::read(&mut readable)?),
        GRID_STR => FrameBody::GRID(GRID::read(&mut readable)?),
        IPLS_STR => FrameBody::IPLS(IPLS::read(&mut readable)?),
        LINK_STR => FrameBody::LINK(LINK::read(&mut readable, version)?),
        MCDI_STR => FrameBody::MCDI(MCDI::read(&mut readable)?),
        MLLT_STR => FrameBody::MLLT(MLLT::read(&mut readable)?),
        OWNE_STR => FrameBody::OWNE(OWNE::read(&mut readable)?),
        PRIV_STR => FrameBody::PRIV(PRIV::read(&mut readable)?),
        PCNT_STR => FrameBody::PCNT(PCNT::read(&mut readable)?),
        POPM_STR => FrameBody::POPM(POPM::read(&mut readable)?),
        POSS_STR => FrameBody::POSS(POSS::read(&mut readable)?),
        RBUF_STR => FrameBody::RBUF(RBUF::read(&mut readable)?),
        RVAD_STR => FrameBody::RVAD(RVA2::read(&mut readable)?),
        RVA2_STR => FrameBody::RVA2(RVA2::read(&mut readable)?),
        RVRB_STR => FrameBody::RVRB(RVRB::read(&mut readable)?),
        SEEK_STR => FrameBody::SEEK(SEEK::read(&mut readable)?),
        SIGN_STR => FrameBody::SIGN(SIGN::read(&mut readable)?),
        SYLT_STR => FrameBody::SYLT(SYLT::read(&mut readable)?),
        SYTC_STR => FrameBody::SYTC(SYTC::read(&mut readable)?),
        UFID_STR => FrameBody::UFID(UFID::read(&mut readable)?),
        USER_STR => FrameBody::USER(USER::read(&mut readable)?),
        USLT_STR => FrameBody::USLT(USLT::read(&mut readable)?),
        TALB_STR => FrameBody::TALB(TEXT::read(&mut readable, id)?),
        TBPM_STR => FrameBody::TBPM(TEXT::read(&mut readable, id)?),
        TCOM_STR => FrameBody::TCOM(TEXT::read(&mut readable, id)?),
        TCON_STR => FrameBody::TCON(TEXT::read(&mut readable, id)?),
        TCOP_STR => FrameBody::TCOP(TEXT::read(&mut readable, id)?),
        TDAT_STR => FrameBody::TDAT(TEXT::read(&mut readable, id)?),
        TDEN_STR => FrameBody::TDEN(TEXT::read(&mut readable, id)?),
        TDLY_STR => FrameBody::TDLY(TEXT::read(&mut readable, id)?),
        TDOR_STR => FrameBody::TDOR(TEXT::read(&mut readable, id)?),
        TDRC_STR => FrameBody::TDRC(TEXT::read(&mut readable, id)?),
        TDRL_STR => FrameBody::TDRL(TEXT::read(&mut readable, id)?),
        TDTG_STR => FrameBody::TDTG(TEXT::read(&mut readable, id)?),
        TENC_STR => FrameBody::TENC(TEXT::read(&mut readable, id)?),
        TEXT_STR => FrameBody::TEXT(TEXT::read(&mut readable, id)?),
        TIME_STR => FrameBody::TIME(TEXT::read(&mut readable, id)?),
        TFLT_STR => FrameBody::TFLT(TEXT::read(&mut readable, id)?),
        TIPL_STR => FrameBody::TIPL(TEXT::read(&mut readable, id)?),
        TIT1_STR => FrameBody::TIT1(TEXT::read(&mut readable, id)?),
        TIT2_STR => FrameBody::TIT2(TEXT::read(&mut readable, id)?),
        TIT3_STR => FrameBody::TIT3(TEXT::read(&mut readable, id)?),
        TKEY_STR => FrameBody::TKEY(TEXT::read(&mut readable, id)?),
        TLAN_STR => FrameBody::TLAN(TEXT::read(&mut readable, id)?),
        TLEN_STR => FrameBody::TLEN(TEXT::read(&mut readable, id)?),
        TMCL_STR => FrameBody::TMCL(TEXT::read(&mut readable, id)?),
        TMED_STR => FrameBody::TMED(TEXT::read(&mut readable, id)?),
        TMOO_STR => FrameBody::TMOO(TEXT::read(&mut readable, id)?),
        TOAL_STR => FrameBody::TOAL(TEXT::read(&mut readable, id)?),
        TOFN_STR => FrameBody::TOFN(TEXT::read(&mut readable, id)?),
        TOLY_STR => FrameBody::TOLY(TEXT::read(&mut readable, id)?),
        TOPE_STR => FrameBody::TOPE(TEXT::read(&mut readable, id)?),
        TORY_STR => FrameBody::TORY(TEXT::read(&mut readable, id)?),
        TOWN_STR => FrameBody::TOWN(TEXT::read(&mut readable, id)?),
        TPE1_STR => FrameBody::TPE1(TEXT::read(&mut readable, id)?),
        TPE2_STR => FrameBody::TPE2(TEXT::read(&mut readable, id)?),
        TPE3_STR => FrameBody::TPE3(TEXT::read(&mut readable, id)?),
        TPE4_STR => FrameBody::TPE4(TEXT::read(&mut readable, id)?),
        TPOS_STR => FrameBody::TPOS(TEXT::read(&mut readable, id)?),
        TPRO_STR => FrameBody::TPRO(TEXT::read(&mut readable, id)?),
        TPUB_STR => FrameBody::TPUB(TEXT::read(&mut readable, id)?),
        TRCK_STR => FrameBody::TRCK(TEXT::read(&mut readable, id)?),
        TRDA_STR => FrameBody::TRDA(TEXT::read(&mut readable, id)?),
        TRSN_STR => FrameBody::TRSN(TEXT::read(&mut readable, id)?),
        TSIZ_STR => FrameBody::TSIZ(TEXT::read(&mut readable, id)?),
        TRSO_STR => FrameBody::TRSO(TEXT::read(&mut readable, id)?),
        TSOA_STR => FrameBody::TSOA(TEXT::read(&mut readable, id)?),
        TSOP_STR => FrameBody::TSOP(TEXT::read(&mut readable, id)?),
        TSOT_STR => FrameBody::TSOT(TEXT::read(&mut readable, id)?),
        TSRC_STR => FrameBody::TSRC(TEXT::read(&mut readable, id)?),
        TSSE_STR => FrameBody::TSSE(TEXT::read(&mut readable, id)?),
        TYER_STR => FrameBody::TYER(TEXT::read(&mut readable, id)?),
        TSST_STR => FrameBody::TSST(TEXT::read(&mut readable, id)?),
        TXXX_STR => FrameBody::TXXX(TXXX::read(&mut readable)?),
        WCOM_STR => FrameBody::WCOM(LINK::read(&mut readable, version)?),
        WCOP_STR => FrameBody::WCOP(LINK::read(&mut readable, version)?),
        WOAF_STR => FrameBody::WOAF(LINK::read(&mut readable, version)?),
        WOAR_STR => FrameBody::WOAR(LINK::read(&mut readable, version)?),
        WOAS_STR => FrameBody::WOAS(LINK::read(&mut readable, version)?),
        WORS_STR => FrameBody::WORS(LINK::read(&mut readable, version)?),
        WPAY_STR => FrameBody::WPAY(LINK::read(&mut readable, version)?),
        WPUB_STR => FrameBody::WPUB(LINK::read(&mut readable, version)?),
        WXXX_STR => FrameBody::WXXX(WXXX::read(&mut readable)?),
        _ => {
            warn!("No frame id found!! '{}'", id);
            FrameBody::TEXT(TEXT::read(&mut readable, id)?)
        }
    };

    debug!("total read: {}", readable.total_read());

    Ok(frame_body)
}