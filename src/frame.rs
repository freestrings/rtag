extern crate encoding;
extern crate regex;

use self::encoding::{
    Encoding,
    DecoderTrap,
    EncoderTrap
};
use self::encoding::all::{
    ISO_8859_1,
    UTF_16LE,
    UTF_16BE,
    UTF_8
};

use errors::*;
use util;
use writable::Writable;

use std::io::{
    Cursor,
    Result,
    Error,
    ErrorKind
};
use std::result;
use std::vec::Vec;

type Readable = ::readable::Readable<Cursor<Vec<u8>>>;

fn trim(text: String) -> String {
    // TODO const
    let re = regex::Regex::new(r"(^[\x{0}|\x{feff}|\x{fffe}]*|[\x{0}|\x{feff}|\x{fffe}]*$)").unwrap();
    let text = text.trim();
    re.replace_all(text, "").into_owned()
}

pub trait FrameReaderDefault<T> {
    fn read(readable: &mut Readable) -> Result<T>;
}

pub trait FrameReaderIdAware<T> {
    fn read(readable: &mut Readable, id: &str) -> Result<T>;
}

pub trait FrameReaderVersionAware<T> {
    fn read(readable: &mut Readable, version: u8) -> Result<T>;
}

pub trait FrameWriterDefault {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()>;
}

pub trait FrameWriterVersionAware<T> {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>, version: u8) -> Result<()>;
}

pub trait FrameWriterIdAware<T> {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>, id: &str) -> Result<()>;
}

pub trait FlagAware<T> {
    fn has_flag(&self, flag: T) -> bool;
    fn set_flag(&mut self, flag: T);
}

#[derive(Clone, Debug, PartialEq)]
pub struct Head {
    pub version: u8,
    pub minor_version: u8,
    pub flag: u8,
    pub size: u32
}

// http://id3.org/id3v2.4.0-structure > 3.1 id3v2 Header
impl Head {
    pub fn read(readable: &mut Readable) -> result::Result<Self, ParsingError> {
        let tag_id = readable.string(3)?;
        let version = readable.u8()?;
        let minor_version = readable.u8()?;
        let flag = readable.u8()?;
        let size = readable.synchsafe()?;

        if tag_id != "ID3" {
            return Err(ParsingError::BadData(ParsingErrorKind::InvalidV2FrameId));
        }

        Ok(Head {
            version: version,
            minor_version: minor_version,
            flag: flag,
            size: size
        })
    }
}

impl FlagAware<HeadFlag> for Head {
    // ./id3v2_summary.md/id3v2.md#id3v2 Header
    //
    // Head level 'Unsynchronisation' does not work on "./test-resources/v2.4-unsync.mp3".
    fn has_flag(&self, flag: HeadFlag) -> bool {
        match self.version {
            2 => match flag {
                HeadFlag::Unsynchronisation => self.flag & util::BIT7 != 0,
                HeadFlag::Compression => self.flag & util::BIT6 != 0,
                _ => false
            },
            3 => match flag {
                HeadFlag::Unsynchronisation => self.flag & util::BIT7 != 0,
                HeadFlag::ExtendedHeader => self.flag & util::BIT6 != 0,
                HeadFlag::ExperimentalIndicator => self.flag & util::BIT5 != 0,
                _ => false
            },
            4 => match flag {
                //
                // HeadFlag::Unsynchronisation => self.flag & util::BIT7 != 0,
                HeadFlag::ExtendedHeader => self.flag & util::BIT6 != 0,
                HeadFlag::ExperimentalIndicator => self.flag & util::BIT5 != 0,
                HeadFlag::FooterPresent => self.flag & util::BIT4 != 0,
                _ => false
            },
            _ => {
                warn!("Header.has_flag=> Unknown version!");
                false
            }
        }
    }

    fn set_flag(&mut self, flag: HeadFlag) {
        match self.version {
            2 => match flag {
                HeadFlag::Unsynchronisation => self.flag = self.flag | util::BIT7,
                HeadFlag::Compression => self.flag = self.flag | util::BIT6,
                _ => ()
            },
            3 => match flag {
                HeadFlag::Unsynchronisation => self.flag = self.flag | util::BIT7,
                HeadFlag::ExtendedHeader => self.flag = self.flag | util::BIT6,
                HeadFlag::ExperimentalIndicator => self.flag = self.flag | util::BIT5,
                _ => ()
            },
            4 => match flag {
                //
                // HeadFlag::Unsynchronisation => self.flag & util::BIT7 != 0,
                HeadFlag::ExtendedHeader => self.flag = self.flag | util::BIT6,
                HeadFlag::ExperimentalIndicator => self.flag = self.flag | util::BIT5,
                HeadFlag::FooterPresent => self.flag = self.flag | util::BIT4,
                _ => ()
            },
            _ => {
                warn!("Header.has_flag=> Unknown version!");
            }
        }
    }
}

impl FrameWriterDefault for Head {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.string("ID3")?;
        writable.u8(self.version)?;
        writable.u8(self.minor_version)?;
        writable.u8(self.flag)?;
        writable.synchsafe(self.size)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame1 {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: String,
    pub genre: String
}

impl Frame1 {
    pub fn read(readable: &mut Readable) -> result::Result<Self, ParsingError> {
        readable.skip(3)?;

        // offset 3
        let title = util::to_iso8859_1(&readable.bytes(30)?).trim().to_string();
        // offset 33
        let artist = util::to_iso8859_1(&readable.bytes(30)?).trim().to_string();
        // offset 63
        let album = util::to_iso8859_1(&readable.bytes(30)?).trim().to_string();
        // offset 93
        let year = util::to_iso8859_1(&readable.bytes(4)?).trim().to_string();
        // goto track marker offset
        readable.skip(28)?;
        // offset 125
        let track_marker = readable.u8()?;
        // offset 126
        let _track = readable.u8()? & 0xff;
        // offset 127
        let genre = (readable.u8()? & 0xff).to_string();
        // goto comment offset
        readable.skip(-31)?;

        let (comment, track) = if track_marker != 0 {
            (
                util::to_iso8859_1(&readable.bytes(30)?).trim().to_string(),
                String::new()
            )
        } else {
            (
                util::to_iso8859_1(&readable.bytes(28)?).trim().to_string(),
                if _track == 0 {
                    String::new()
                } else {
                    _track.to_string()
                }
            )
        };

        Ok(Frame1 {
            title: title,
            artist: artist,
            album: album,
            year: year,
            comment: comment,
            track: track,
            genre: genre
        })
    }
}

impl FrameWriterDefault for Frame1 {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.string("TAG")?;
        writable.write(&util::from_iso8859_1(&self.title, 30))?;
        writable.write(&util::from_iso8859_1(&self.artist, 30))?;
        writable.write(&util::from_iso8859_1(&self.album, 30))?;
        writable.write(&util::from_iso8859_1(&self.year, 4))?;
        writable.write(&util::from_iso8859_1(&self.comment, 28))?;
        writable.u8(0)?;//track marker
        match self.track.as_str().parse::<u8>() {
            Ok(v) => writable.u8(v)?,
            Err(_) => writable.u8(0)?,
        };
        match self.genre.as_str().parse::<u8>() {
            Ok(v) => writable.u8(v)?,
            Err(_) => writable.u8(0)?,
        };

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameHeader {
    V22(FrameHeaderV2),
    V23(FrameHeaderV3),
    V24(FrameHeaderV4)
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameHeaderV2 {
    pub id: String,
    pub size: u32,
}

impl FrameHeaderV2 {
    pub fn read(readable: &mut Readable) -> result::Result<Self, ParsingError> {
        let id = readable.string(3)?;
        let size = readable.u24()?;

        Ok(FrameHeaderV2 {
            id: id,
            size: size
        })
    }
}

impl FlagAware<FrameHeaderFlag> for FrameHeaderV2 {
    // There is no flag for 2.2 frame.
    #[allow(unused_variables)]
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        return false;
    }
    #[allow(unused_variables)]
    fn set_flag(&mut self, flag: FrameHeaderFlag) {}
}

impl FrameWriterDefault for FrameHeaderV2 {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.string(self.id.as_str())?;
        writable.u24(self.size)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameHeaderV3 {
    pub id: String,
    pub size: u32,
    pub status_flag: u8,
    pub encoding_flag: u8
}

impl FrameHeaderV3 {
    pub fn read(readable: &mut Readable) -> result::Result<Self, ParsingError> {
        let id = readable.string(4)?;
        let size = readable.u32()?;
        let status_flag = readable.u8()?;
        let encoding_flag = readable.u8()?;

        Ok(FrameHeaderV3 {
            id: id,
            size: size,
            status_flag: status_flag,
            encoding_flag: encoding_flag
        })
    }
}

impl FlagAware<FrameHeaderFlag> for FrameHeaderV3 {
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        match flag {
            FrameHeaderFlag::TagAlter => self.status_flag & util::BIT7 != 0,
            FrameHeaderFlag::FileAlter => self.status_flag & util::BIT6 != 0,
            FrameHeaderFlag::ReadOnly => self.status_flag & util::BIT5 != 0,
            FrameHeaderFlag::Compression => self.encoding_flag & util::BIT7 != 0,
            FrameHeaderFlag::Encryption => self.encoding_flag & util::BIT6 != 0,
            FrameHeaderFlag::GroupIdentity => self.encoding_flag & util::BIT5 != 0,
            _ => false
        }
    }

    fn set_flag(&mut self, flag: FrameHeaderFlag) {
        match flag {
            FrameHeaderFlag::TagAlter =>
                self.status_flag = self.status_flag | util::BIT7,
            FrameHeaderFlag::FileAlter =>
                self.status_flag = self.status_flag | util::BIT6,
            FrameHeaderFlag::ReadOnly =>
                self.status_flag = self.status_flag | util::BIT5,
            FrameHeaderFlag::Compression =>
                self.encoding_flag = self.encoding_flag | util::BIT7,
            FrameHeaderFlag::Encryption =>
                self.encoding_flag = self.encoding_flag | util::BIT6,
            FrameHeaderFlag::GroupIdentity =>
                self.encoding_flag = self.encoding_flag | util::BIT5,
            _ => ()
        }
    }
}

impl FrameWriterDefault for FrameHeaderV3 {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        let mut ext_size = 0;
        if self.has_flag(FrameHeaderFlag::GroupIdentity) {
            ext_size = 1;
        }
        if self.has_flag(FrameHeaderFlag::Encryption) {
            ext_size = ext_size + 1;
        }
        if self.has_flag(FrameHeaderFlag::Compression) {
            ext_size = ext_size + 4;
        }
        writable.string(self.id.as_str())?;
        writable.u32(self.size + ext_size)?;
        writable.u8(self.status_flag)?;
        writable.u8(self.encoding_flag)?;

        if self.has_flag(FrameHeaderFlag::GroupIdentity) {
            writable.u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::Encryption) {
            writable.u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::Compression) {
            writable.u32(0)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameHeaderV4 {
    pub id: String,
    pub size: u32,
    pub status_flag: u8,
    pub encoding_flag: u8
}

impl FrameHeaderV4 {
    pub fn read(readable: &mut Readable) -> result::Result<Self, ParsingError> {
        let id = readable.string(4)?;
        let size = readable.synchsafe()?;
        let status_flag = readable.u8()?;
        let encoding_flag = readable.u8()?;

        Ok(FrameHeaderV4 {
            id: id,
            size: size,
            status_flag: status_flag,
            encoding_flag: encoding_flag
        })
    }
}

impl FlagAware<FrameHeaderFlag> for FrameHeaderV4 {
    // http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        match flag {
            FrameHeaderFlag::TagAlter => self.status_flag & util::BIT6 != 0,
            FrameHeaderFlag::FileAlter => self.status_flag & util::BIT5 != 0,
            FrameHeaderFlag::ReadOnly => self.status_flag & util::BIT4 != 0,
            FrameHeaderFlag::GroupIdentity => self.encoding_flag & util::BIT6 != 0,
            FrameHeaderFlag::Compression => self.encoding_flag & util::BIT3 != 0,
            FrameHeaderFlag::Encryption => self.encoding_flag & util::BIT2 != 0,
            FrameHeaderFlag::Unsynchronisation => self.encoding_flag & util::BIT1 != 0,
            FrameHeaderFlag::DataLength => self.encoding_flag & util::BIT0 != 0
        }
    }

    fn set_flag(&mut self, flag: FrameHeaderFlag) {
        match flag {
            FrameHeaderFlag::TagAlter =>
                self.status_flag = self.status_flag | util::BIT6,
            FrameHeaderFlag::FileAlter =>
                self.status_flag = self.status_flag | util::BIT5,
            FrameHeaderFlag::ReadOnly =>
                self.status_flag = self.status_flag | util::BIT4,
            FrameHeaderFlag::GroupIdentity =>
                self.encoding_flag = self.encoding_flag | util::BIT6,
            FrameHeaderFlag::Compression =>
                self.encoding_flag = self.encoding_flag | util::BIT3,
            FrameHeaderFlag::Encryption =>
                self.encoding_flag = self.encoding_flag | util::BIT2,
            FrameHeaderFlag::Unsynchronisation =>
                self.encoding_flag = self.encoding_flag | util::BIT1,
            FrameHeaderFlag::DataLength =>
                self.encoding_flag = self.encoding_flag | util::BIT0
        }
    }
}

impl FrameWriterDefault for FrameHeaderV4 {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        let mut ext_size = 0;
        if self.has_flag(FrameHeaderFlag::GroupIdentity) {
            ext_size = 1;
        }
        if self.has_flag(FrameHeaderFlag::Encryption) {
            ext_size = ext_size + 1;
        }
        if self.has_flag(FrameHeaderFlag::DataLength) {
            ext_size = ext_size + 4;
        }

        writable.string(self.id.as_str())?;
        writable.synchsafe(self.size + ext_size)?;
        writable.u8(self.status_flag)?;
        writable.u8(self.encoding_flag)?;

        if self.has_flag(FrameHeaderFlag::GroupIdentity) {
            writable.u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::Encryption) {
            writable.u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::DataLength) {
            writable.u32(0)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextEncoding {
    ISO88591,
    UTF16LE,
    UTF16BE,
    UTF8
}

#[derive(Clone, Debug, PartialEq)]
pub enum PictureType {
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

#[derive(Clone, Debug, PartialEq)]
pub enum ReceivedAs {
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

#[derive(Clone, Debug, PartialEq)]
pub enum InterpolationMethod {
    Band,
    Linear
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContentType {
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

#[derive(Clone, Debug, PartialEq)]
pub enum TimestampFormat {
    MpecFrames,
    Milliseconds
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventTimingCode {
    Padding(u32),
    EndOfInitialSilence(u32),
    IntroStart(u32),
    MainPartStart(u32),
    OutroStart(u32),
    OutroEnd(u32),
    VerseStart(u32),
    RefrainStart(u32),
    InterludeStart(u32),
    ThemeStart(u32),
    VariationStart(u32),
    KeyChange(u32),
    TimeChange(u32),
    MomentaryUnwantedNoise(u32),
    SustainedNoise(u32),
    SustainedNoiseEnd(u32),
    IntroEnd(u32),
    MainPartEnd(u32),
    VerseEnd(u32),
    RefrainEnd(u32),
    ThemeEnd(u32),
    Profanity(u32),
    ProfanityEnd(u32),
    ReservedForFutureUse(u32, u8),
    NotPredefinedSynch(u32, u8),
    AudioEnd(u32),
    AudioFileEnds(u32),
    OneMoreByteOfEventsFollows(u32)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameHeaderFlag {
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

#[derive(Clone, Debug, PartialEq)]
pub enum HeadFlag {
    Unsynchronisation,
    Compression,
    ExtendedHeader,
    ExperimentalIndicator,
    FooterPresent
}

// TODO not yet tested!
// Recommended buffer size
#[derive(Clone, Debug, PartialEq)]
pub struct BUF {
    pub buffer_size: u32,
    pub embedded_info_flag: u8,
    pub offset_to_next_tag: u32
}

impl FrameReaderDefault<BUF> for BUF {
    fn read(readable: &mut Readable) -> Result<BUF> {
        let buffer_size = readable.u24()?;
        let embedded_info_flag = readable.u8()?;
        let offset_to_next_tag = readable.u32()?;

        Ok(BUF {
            buffer_size: buffer_size,
            embedded_info_flag: embedded_info_flag,
            offset_to_next_tag: offset_to_next_tag
        })
    }
}

impl FrameWriterDefault for BUF {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u24(self.buffer_size)?;
        writable.u8(self.embedded_info_flag)?;
        writable.u32(self.offset_to_next_tag)
    }
}

// TODO not yet tested!
// Encrypted meta frame
#[derive(Clone, Debug, PartialEq)]
pub struct CRM {
    pub owner_identifier: String,
    pub content: String,
    pub encrypted_datablock: Vec<u8>
}

impl FrameReaderDefault<CRM> for CRM {
    fn read(readable: &mut Readable) -> Result<CRM> {
        let owner_identifier = readable.non_utf16_string()?;
        let content = readable.non_utf16_string()?;
        let encrypted_datablock = readable.all_bytes()?;

        Ok(CRM {
            owner_identifier: owner_identifier,
            content: content,
            encrypted_datablock: encrypted_datablock
        })
    }
}

impl FrameWriterDefault for CRM {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.non_utf16_string(self.owner_identifier.as_str())?;
        writable.non_utf16_string(self.content.as_str())?;
        writable.write(&self.encrypted_datablock)
    }
}

// Attached picture
#[derive(Clone, Debug, PartialEq)]
pub struct PIC {
    pub text_encoding: TextEncoding,
    pub image_format: String,
    pub picture_type: PictureType,
    pub description: String,
    pub picture_data: Vec<u8>
}

impl FrameReaderDefault<PIC> for PIC {
    fn read(readable: &mut Readable) -> Result<PIC> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let image_format = readable.string(3)?;
        let picture_type = util::to_picture_type(readable.u8()?);
        let description = util::read_null_terminated(&text_encoding, readable)?;
        let picture_data = readable.all_bytes()?;

        Ok(PIC {
            text_encoding: text_encoding,
            image_format: image_format,
            picture_type: picture_type,
            description: description,
            picture_data: picture_data
        })
    }
}

impl FrameWriterDefault for PIC {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(&self.image_format[0..3])?;
        writable.u8(util::from_picture_type(&self.picture_type))?;
        util::write_null_terminated(&self.text_encoding, self.description.as_str(), writable)?;
        writable.write(&self.picture_data)
    }
}

// Audio encryption
#[derive(Clone, Debug, PartialEq)]
pub struct AENC {
    pub owner_identifier: String,
    pub preview_start: u16,
    pub preview_end: u16,
    pub encryption_info: Vec<u8>
}

impl FrameReaderDefault<AENC> for AENC {
    fn read(readable: &mut Readable) -> Result<AENC> {
        let owner_identifier = readable.non_utf16_string()?;
        let preview_start = readable.u16()?;
        let preview_end = readable.u16()?;
        let encryption_info = readable.all_bytes()?;

        Ok(AENC {
            owner_identifier: owner_identifier,
            preview_start: preview_start,
            preview_end: preview_end,
            encryption_info: encryption_info
        })
    }
}

impl FrameWriterDefault for AENC {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.non_utf16_string(self.owner_identifier.as_str())?;
        writable.u16(self.preview_start)?;
        writable.u16(self.preview_end)?;
        writable.write(&self.encryption_info)
    }
}

// TODO not yet tested!
// Attached picture
#[derive(Clone, Debug, PartialEq)]
pub struct APIC {
    pub text_encoding: TextEncoding,
    pub mime_type: String,
    pub picture_type: PictureType,
    pub description: String,
    pub picture_data: Vec<u8>
}

impl FrameReaderDefault<APIC> for APIC {
    fn read(readable: &mut Readable) -> Result<APIC> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let mine_type = readable.non_utf16_string()?;
        let picture_type = util::to_picture_type(readable.u8()?);
        let description = util::read_null_terminated(&text_encoding, readable)?;
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

impl FrameWriterDefault for APIC {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(self.mime_type.as_str())?;
        writable.u8(util::from_picture_type(&self.picture_type))?;
        writable.write(&self.picture_data)
    }
}

// TODO not yet tested!
// Audio seek point index
#[derive(Clone, Debug, PartialEq)]
pub struct ASPI {
    pub indexed_data_start: u32,
    pub indexed_data_length: u32,
    pub number_of_index_points: u16,
    pub bit_per_index_point: u8,
    pub fraction_at_index: u8
}

impl FrameReaderDefault<ASPI> for ASPI {
    fn read(readable: &mut Readable) -> Result<ASPI> {
        let indexed_data_start = readable.u32()?;
        let indexed_data_length = readable.u32()?;
        let number_of_index_points = readable.u16()?;
        let bit_per_index_point = readable.u8()?;
        let fraction_at_index = readable.u8()?;

        Ok(ASPI {
            indexed_data_start: indexed_data_start,
            indexed_data_length: indexed_data_length,
            number_of_index_points: number_of_index_points,
            bit_per_index_point: bit_per_index_point,
            fraction_at_index: fraction_at_index
        })
    }
}

impl FrameWriterDefault for ASPI {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u32(self.indexed_data_start)?;
        writable.u32(self.indexed_data_length)?;
        writable.u16(self.number_of_index_points)?;
        writable.u8(self.bit_per_index_point)?;
        writable.u8(self.fraction_at_index)
    }
}

// Comments
#[derive(Clone, Debug, PartialEq)]
pub struct COMM {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub short_description: String,
    pub actual_text: String
}

impl FrameReaderDefault<COMM> for COMM {
    fn read(readable: &mut Readable) -> Result<COMM> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let language = readable.string(3)?;
        let short_description = util::read_null_terminated(&text_encoding, readable)?;
        let actual_text = self::trim(readable.all_string()?);

        Ok(COMM {
            text_encoding: text_encoding,
            language: language,
            short_description: short_description,
            actual_text: actual_text
        })
    }
}

impl FrameWriterDefault for COMM {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(&self.language[0..3])?;
        util::write_null_terminated(&self.text_encoding, self.short_description.as_str(), writable)?;
        writable.string(self.actual_text.as_str())
    }
}

// TODO not yet tested!
// Commercial frame
#[derive(Clone, Debug, PartialEq)]
pub struct COMR {
    pub text_encoding: TextEncoding,
    pub price_string: String,
    // 8 bit long
    pub valid_until: String,
    pub contact_url: String,
    pub received_as: ReceivedAs,
    pub name_of_seller: String,
    pub description: String,
    pub picture_mime_type: String,
    pub seller_logo: Vec<u8>
}

impl FrameReaderDefault<COMR> for COMR {
    fn read(readable: &mut Readable) -> Result<COMR> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let price_string = readable.non_utf16_string()?;
        let valid_until = readable.string(8)?;
        let contact_url = readable.non_utf16_string()?;
        let received_as = util::to_received_as(readable.u8()?);
        let name_of_seller = readable.utf16_string()?;
        let description = readable.utf16_string()?;
        let picture_mime_type = readable.non_utf16_string()?;
        let seller_logo = readable.all_bytes()?;

        Ok(COMR {
            text_encoding: text_encoding,
            price_string: price_string,
            valid_until: valid_until,
            contact_url: contact_url,
            received_as: received_as,
            name_of_seller: name_of_seller,
            description: description,
            picture_mime_type: picture_mime_type,
            seller_logo: seller_logo
        })
    }
}

impl FrameWriterDefault for COMR {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(self.price_string.as_str())?;
        writable.string(&self.valid_until[0..8])?;
        writable.non_utf16_string(self.contact_url.as_str())?;
        writable.u8(util::from_received_as(&self.received_as))?;
        writable.utf16_string(self.name_of_seller.as_str())?;
        writable.string(self.description.as_str())?;
        writable.non_utf16_string(self.picture_mime_type.as_str())?;
        writable.write(&self.seller_logo)
    }
}

// TODO not yet tested!
// Encryption method registration
#[derive(Clone, Debug, PartialEq)]
pub struct ENCR {
    pub owner_identifier: String,
    pub method_symbol: u8,
    pub encryption_data: Vec<u8>
}

impl FrameReaderDefault<ENCR> for ENCR {
    fn read(readable: &mut Readable) -> Result<ENCR> {
        let owner_identifier = readable.non_utf16_string()?;
        let method_symbol = readable.u8()?;
        let encryption_data = readable.all_bytes()?;

        Ok(ENCR {
            owner_identifier: owner_identifier,
            method_symbol: method_symbol,
            encryption_data: encryption_data
        })
    }
}

impl FrameWriterDefault for ENCR {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.string(self.owner_identifier.as_str())?;
        writable.u8(self.method_symbol)?;
        writable.write(&self.encryption_data)
    }
}

// TODO not yet tested!
// Equalisation
#[derive(Clone, Debug, PartialEq)]
pub struct EQUA {
    pub adjustment_bit: u8
}

impl FrameReaderDefault<EQUA> for EQUA {
    fn read(readable: &mut Readable) -> Result<EQUA> {
        let adjustment_bit = readable.u8()?;

        Ok(EQUA {
            adjustment_bit: adjustment_bit
        })
    }
}

impl FrameWriterDefault for EQUA {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(self.adjustment_bit)
    }
}

// TODO not yet tested!
// Equalisation (2)
#[derive(Clone, Debug, PartialEq)]
pub struct EQU2 {
    pub interpolation_method: InterpolationMethod,
    pub identification: String
}

impl FrameReaderDefault<EQU2> for EQU2 {
    fn read(readable: &mut Readable) -> Result<EQU2> {
        let interpolation_method = util::to_interpolation_method(readable.u8()?);
        let identification = readable.non_utf16_string()?;

        Ok(EQU2 {
            interpolation_method: interpolation_method,
            identification: identification
        })
    }
}

impl FrameWriterDefault for EQU2 {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_interpolation_method(&self.interpolation_method))?;
        writable.string(self.identification.as_str())
    }
}

// Event timing codes
#[derive(Clone, Debug, PartialEq)]
pub struct ETCO {
    pub timestamp_format: TimestampFormat,
    pub event_timing_codes: Vec<EventTimingCode>
}

impl FrameReaderDefault<ETCO> for ETCO {
    fn read(readable: &mut Readable) -> Result<ETCO> {
        let timestamp_format = util::to_timestamp_format(readable.u8()?);
        let mut event_timing_codes: Vec<EventTimingCode> = Vec::new();
        loop {
            let mut is_break = true;
            if let Ok(code_type) = readable.u8() {
                if let Ok(timestamp) = readable.u32() {
                    let event_timing_code = util::to_event_timing_code(code_type, timestamp);
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

impl FrameWriterDefault for ETCO {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_timestamp_format(&self.timestamp_format))?;
        for e in &self.event_timing_codes {
            let (code, timestamp) = util::from_event_timing_code(&e);
            writable.u8(code)?;
            writable.u32(timestamp)?;
        }

        Ok((()))
    }
}

// General encapsulated object
#[derive(Clone, Debug, PartialEq)]
pub struct GEOB {
    pub text_encoding: TextEncoding,
    pub mime_type: String,
    pub filename: String,
    pub content_description: String,
    pub encapsulation_object: Vec<u8>
}

impl FrameReaderDefault<GEOB> for GEOB {
    fn read(readable: &mut Readable) -> Result<GEOB> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let mime_type = readable.non_utf16_string()?;
        let filename = util::read_null_terminated(&text_encoding, readable)?;
        let content_description = util::read_null_terminated(&text_encoding, readable)?;
        let encapsulation_object = readable.all_bytes()?;

        Ok(GEOB {
            text_encoding: text_encoding,
            mime_type: mime_type,
            filename: filename,
            content_description: content_description,
            encapsulation_object: encapsulation_object
        })
    }
}

impl FrameWriterDefault for GEOB {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.non_utf16_string(self.mime_type.as_str())?;
        util::write_null_terminated(&self.text_encoding, self.filename.as_str(), writable)?;
        util::write_null_terminated(&self.text_encoding, self.content_description.as_str(), writable)?;
        writable.write(&self.encapsulation_object)
    }
}

// TODO not yet tested!
// Group identification registration
#[derive(Clone, Debug, PartialEq)]
pub struct GRID {
    pub owner_identifier: String,
    pub group_symbol: u8,
    pub group_dependent_data: Vec<u8>
}

impl FrameReaderDefault<GRID> for GRID {
    fn read(readable: &mut Readable) -> Result<GRID> {
        let owner_identifier = readable.non_utf16_string()?;
        let group_symbol = readable.u8()?;
        let group_dependent_data = readable.all_bytes()?;

        Ok(GRID {
            owner_identifier: owner_identifier,
            group_symbol: group_symbol,
            group_dependent_data: group_dependent_data
        })
    }
}

impl FrameWriterDefault for GRID {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.non_utf16_string(self.owner_identifier.as_str())?;
        writable.u8(self.group_symbol)?;
        writable.write(&self.group_dependent_data)
    }
}

#[derive(Clone, Debug, PartialEq)]
// Involved people list
pub struct IPLS {
    pub text_encoding: TextEncoding,
    pub people_list_strings: String
}

impl FrameReaderDefault<IPLS> for IPLS {
    fn read(readable: &mut Readable) -> Result<IPLS> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let people_list_strings = util::read_null_terminated(&text_encoding, readable)?;

        Ok(IPLS {
            text_encoding: text_encoding,
            people_list_strings: people_list_strings
        })
    }
}

impl FrameWriterDefault for IPLS {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(self.people_list_strings.as_str())
    }
}

// Linked information
#[derive(Clone, Debug, PartialEq)]
pub struct LINK {
    pub frame_identifier: String,
    pub url: String,
    pub additional_data: String
}

impl FrameReaderVersionAware<LINK> for LINK {
    fn read(readable: &mut Readable, version: u8) -> Result<LINK> {
        let frame_id = match version {
            2 | 3 => readable.string(3)?,
            _ => readable.string(4)?
        };
        let url = readable.non_utf16_string()?;
        let additional_data = readable.all_string()?;

        Ok(LINK {
            frame_identifier: frame_id,
            url: url,
            additional_data: additional_data
        })
    }
}

impl FrameWriterVersionAware<LINK> for LINK {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>, version: u8) -> Result<()> {
        match version {
            2 | 3 => writable.string(&self.frame_identifier[0..3])?,
            _ => writable.string(&self.frame_identifier[0..4])?
        }
        writable.string(self.url.as_str())?;
        writable.string(self.additional_data.as_str())?;

        Ok(())
    }
}

// Music CD identifier
#[derive(Clone, Debug, PartialEq)]
pub struct MCDI {
    pub cd_toc: Vec<u8>
}

impl FrameReaderDefault<MCDI> for MCDI {
    fn read(readable: &mut Readable) -> Result<MCDI> {
        let cd_toc = readable.all_bytes()?;

        Ok(MCDI {
            cd_toc: cd_toc
        })
    }
}

impl FrameWriterDefault for MCDI {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.write(&self.cd_toc)
    }
}

// TODO not yet tested!
// TODO not yet implemented!
// MPEG location lookup table
#[derive(Clone, Debug, PartialEq)]
pub struct MLLT {
    pub data: Vec<u8>
}

impl FrameReaderDefault<MLLT> for MLLT {
    fn read(readable: &mut Readable) -> Result<MLLT> {
        let data = readable.all_bytes()?;

        Ok(MLLT {
            data: data
        })
    }
}

impl FrameWriterDefault for MLLT {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.write(&self.data)
    }
}

// TODO not yet tested!
// Ownership frame
#[derive(Clone, Debug, PartialEq)]
pub struct OWNE {
    pub text_encoding: TextEncoding,
    pub price_paid: String,
    // 8 bit long
    pub date_of_purch: String,
    pub seller: String
}

impl FrameReaderDefault<OWNE> for OWNE {
    fn read(readable: &mut Readable) -> Result<OWNE> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let price_paid = readable.non_utf16_string()?;
        let date_of_purch = readable.string(4)?;
        let seller = util::read_null_terminated(&text_encoding, readable)?;

        Ok(OWNE {
            text_encoding: text_encoding,
            price_paid: price_paid,
            date_of_purch: date_of_purch,
            seller: seller
        })
    }
}

impl FrameWriterDefault for OWNE {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.non_utf16_string(self.price_paid.as_str())?;
        writable.string(&self.date_of_purch[0..4])?;
        util::write_null_terminated(&self.text_encoding, self.seller.as_str(), writable)
    }
}

// TODO not yet tested!
// Private frame
#[derive(Clone, Debug, PartialEq)]
pub struct PRIV {
    pub owner_identifier: String,
    pub private_data: Vec<u8>
}

impl FrameReaderDefault<PRIV> for PRIV {
    fn read(readable: &mut Readable) -> Result<PRIV> {
        let owner_identifier = readable.non_utf16_string()?;
        let private_data = readable.all_bytes()?;

        Ok(PRIV {
            owner_identifier: owner_identifier,
            private_data: private_data
        })
    }
}

impl FrameWriterDefault for PRIV {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.non_utf16_string(self.owner_identifier.as_str())?;
        writable.write(&self.private_data)
    }
}

// NOTE it support that only the 32-bit unsigned integer type.
// Play counter
#[derive(Clone, Debug, PartialEq)]
pub struct PCNT {
    pub counter: u32
}

impl FrameReaderDefault<PCNT> for PCNT {
    fn read(readable: &mut Readable) -> Result<PCNT> {
        let counter = readable.u32()?;

        Ok(PCNT {
            counter: counter
        })
    }
}

impl FrameWriterDefault for PCNT {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u32(self.counter)
    }
}

// TODO not yet tested!
// Popularimeter
#[derive(Clone, Debug, PartialEq)]
pub struct POPM {
    pub email_to_user: String,
    pub rating: u8,
    // NOTE it support that only the 32-bit unsigned integer type.
    pub counter: u32
}

impl FrameReaderDefault<POPM> for POPM {
    fn read(readable: &mut Readable) -> Result<POPM> {
        let email_to_user = readable.non_utf16_string()?;
        let rating = readable.u8()?;
        let counter = readable.u32()?;

        Ok(POPM {
            email_to_user: email_to_user,
            rating: rating,
            counter: counter
        })
    }
}

impl FrameWriterDefault for POPM {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.non_utf16_string(self.email_to_user.as_str())?;
        writable.u8(self.rating)?;
        writable.u32(self.counter)
    }
}

// TODO not yet tested!
// Position synchronisation frame
#[derive(Clone, Debug, PartialEq)]
pub struct POSS {
    pub timestamp_format: TimestampFormat,
    // TODO not yet implemented!
    pub position: Vec<u8>
}

impl FrameReaderDefault<POSS> for POSS {
    fn read(readable: &mut Readable) -> Result<POSS> {
        let timestamp_format = util::to_timestamp_format(readable.u8()?);
        let position = readable.all_bytes()?;

        Ok(POSS {
            timestamp_format: timestamp_format,
            position: position
        })
    }
}

impl FrameWriterDefault for POSS {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_timestamp_format(&self.timestamp_format))?;
        writable.write(&self.position)
    }
}

// TODO not yet tested!
// Recommended buffer size
#[derive(Clone, Debug, PartialEq)]
pub struct RBUF {
    pub buffer_size: u32,
    pub embedded_info_flag: u8,
    pub offset_to_next_tag: u32
}

impl FrameReaderDefault<RBUF> for RBUF {
    fn read(readable: &mut Readable) -> Result<RBUF> {
        let buffer_size = readable.u24()?;
        let embedded_info_flag = readable.u8()?;
        let offset_to_next_tag = readable.u32()?;

        Ok(RBUF {
            buffer_size: buffer_size,
            embedded_info_flag: embedded_info_flag,
            offset_to_next_tag: offset_to_next_tag
        })
    }
}

impl FrameWriterDefault for RBUF {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u24(self.buffer_size)?;
        writable.u8(self.embedded_info_flag)?;
        writable.u32(self.offset_to_next_tag)
    }
}

// TODO not yet tested!
// TODO not yet implemented!
// Relative volume adjustment (2)
#[derive(Clone, Debug, PartialEq)]
pub struct RVA2 {
    pub data: Vec<u8>
}

impl FrameReaderDefault<RVA2> for RVA2 {
    fn read(readable: &mut Readable) -> Result<RVA2> {
        let data = readable.all_bytes()?;

        Ok(RVA2 {
            data: data
        })
    }
}

impl FrameWriterDefault for RVA2 {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.write(&self.data)
    }
}

// TODO not yet tested!
// Reverb
#[derive(Clone, Debug, PartialEq)]
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

impl FrameReaderDefault<RVRB> for RVRB {
    fn read(readable: &mut Readable) -> Result<RVRB> {
        let reverb_left = readable.u16()?;
        let reverb_right = readable.u16()?;
        let reverb_bounce_left = readable.u8()?;
        let reverb_bounce_right = readable.u8()?;
        let reverb_feedback_left_to_left = readable.u8()?;
        let reverb_feedback_left_to_right = readable.u8()?;
        let reverb_feedback_right_to_right = readable.u8()?;
        let reverb_feedback_right_to_left = readable.u8()?;
        let premix_left_to_right = readable.u8()?;
        let premix_right_to_left = readable.u8()?;

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

impl FrameWriterDefault for RVRB {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u16(self.reverb_left)?;
        writable.u16(self.reverb_right)?;
        writable.u8(self.reverb_bounce_left)?;
        writable.u8(self.reverb_bounce_right)?;
        writable.u8(self.reverb_feedback_left_to_left)?;
        writable.u8(self.reverb_feedback_left_to_right)?;
        writable.u8(self.reverb_feedback_right_to_right)?;
        writable.u8(self.reverb_feedback_right_to_left)?;
        writable.u8(self.premix_left_to_right)?;
        writable.u8(self.premix_right_to_left)
    }
}


// TODO not yet tested!
// Seek frame
#[derive(Clone, Debug, PartialEq)]
pub struct SEEK {
    pub next_tag: String
}

impl FrameReaderDefault<SEEK> for SEEK {
    fn read(readable: &mut Readable) -> Result<SEEK> {
        let next_tag = readable.all_string()?;

        Ok(SEEK {
            next_tag: next_tag
        })
    }
}

impl FrameWriterDefault for SEEK {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.string(self.next_tag.as_str())
    }
}

// TODO not yet tested!
// Signature frame
#[derive(Clone, Debug, PartialEq)]
pub struct SIGN {
    pub group_symbol: u8,
    pub signature: Vec<u8>
}

impl FrameReaderDefault<SIGN> for SIGN {
    fn read(readable: &mut Readable) -> Result<SIGN> {
        let group_symbol = readable.u8()?;
        let signature = readable.all_bytes()?;

        Ok(SIGN {
            group_symbol: group_symbol,
            signature: signature
        })
    }
}

impl FrameWriterDefault for SIGN {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(self.group_symbol)?;
        writable.write(&self.signature)
    }
}

// TODO not yet tested!
// Synchronised lyric/text
#[derive(Clone, Debug, PartialEq)]
pub struct SYLT {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub timestamp_format: TimestampFormat,
    pub content_type: ContentType,
    pub content_descriptor: String
}

impl FrameReaderDefault<SYLT> for SYLT {
    fn read(readable: &mut Readable) -> Result<SYLT> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let language = readable.string(3)?;
        let timestamp_format = util::to_timestamp_format(readable.u8()?);
        let content_type = util::to_content_type(readable.u8()?);
        let content_descriptor = util::read_null_terminated(&text_encoding, readable)?;

        Ok(SYLT {
            text_encoding: text_encoding,
            language: language,
            timestamp_format: timestamp_format,
            content_type: content_type,
            content_descriptor: content_descriptor
        })
    }
}

impl FrameWriterDefault for SYLT {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(&self.language[0..3])?;
        writable.u8(util::from_timestamp_format(&self.timestamp_format))?;
        writable.u8(util::from_content_type(&self.content_type))?;
        util::write_null_terminated(&self.text_encoding, self.content_descriptor.as_str(), writable)
    }
}

// TODO not yet tested!
// Synchronised tempo codes
#[derive(Clone, Debug, PartialEq)]
pub struct SYTC {
    pub timestamp_format: TimestampFormat,
    pub tempo_data: Vec<u8>
}

impl FrameReaderDefault<SYTC> for SYTC {
    fn read(readable: &mut Readable) -> Result<SYTC> {
        let timestamp_format = util::to_timestamp_format(readable.u8()?);
        let tempo_data = readable.all_bytes()?;

        Ok(SYTC {
            timestamp_format: timestamp_format,
            tempo_data: tempo_data
        })
    }
}

impl FrameWriterDefault for SYTC {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_timestamp_format(&self.timestamp_format))?;
        writable.write(&self.tempo_data)
    }
}

// TODO not yet tested!
// Unique file identifier
#[derive(Clone, Debug, PartialEq)]
pub struct UFID {
    pub owner_identifier: String,
    pub identifier: Vec<u8>
}

impl FrameReaderDefault<UFID> for UFID {
    fn read(readable: &mut Readable) -> Result<UFID> {
        let owner_identifier = readable.non_utf16_string()?;
        let identifier = readable.all_bytes()?;

        Ok(UFID {
            owner_identifier: owner_identifier,
            identifier: identifier
        })
    }
}

impl FrameWriterDefault for UFID {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.non_utf16_string(self.owner_identifier.as_str())?;
        writable.write(&self.identifier)
    }
}

// TODO not yet tested!
// Terms of use
#[derive(Clone, Debug, PartialEq)]
pub struct USER {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub actual_text: String
}

impl FrameReaderDefault<USER> for USER {
    fn read(readable: &mut Readable) -> Result<USER> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let language = readable.string(3)?;
        let actual_text = util::read_null_terminated(&text_encoding, readable)?;

        Ok(USER {
            text_encoding: text_encoding,
            language: language,
            actual_text: actual_text
        })
    }
}

impl FrameWriterDefault for USER {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(&self.language[0..3])?;
        util::write_null_terminated(&self.text_encoding, self.actual_text.as_str(), writable)
    }
}

// TODO not yet tested!
// Unsynchronised lyric/text transcription
#[derive(Clone, Debug, PartialEq)]
pub struct USLT {
    pub text_encoding: TextEncoding,
    pub language: String,
    pub content_descriptor: String,
    pub lyrics: String
}

impl FrameReaderDefault<USLT> for USLT {
    fn read(readable: &mut Readable) -> Result<USLT> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let language = readable.string(3)?;
        let content_descriptor = util::read_null_terminated(&text_encoding, readable)?;
        let lyrics = util::read_null_terminated(&text_encoding, readable)?;

        Ok(USLT {
            text_encoding: text_encoding,
            language: language,
            content_descriptor: content_descriptor,
            lyrics: lyrics
        })
    }
}

impl FrameWriterDefault for USLT {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        writable.string(&self.language[0..3])?;
        util::write_null_terminated(&self.text_encoding, self.content_descriptor.as_str(), writable)?;
        util::write_null_terminated(&self.text_encoding, self.lyrics.as_str(), writable)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TEXT {
    pub text_encoding: TextEncoding,
    pub text: String
}

impl FrameReaderIdAware<TEXT> for TEXT {
    fn read(readable: &mut Readable, id: &str) -> Result<TEXT> {
        fn _default(id: &str, decode: ::std::result::Result<String, ::std::borrow::Cow<'static, str>>)
                    -> String {
            match decode {
                Ok(text) => text,
                Err(e) => {
                    debug!("TEXT Error {}, {:?}", id, e);
                    if id == id::TBPM_STR || id == id::TBP_STR {
                        "0".to_string()
                    } else {
                        "".to_string()
                    }
                }
            }
        }

        let text_encoding = util::to_encoding(readable.u8()?);
        let data = readable.all_bytes()?;
        let text = match text_encoding {
            TextEncoding::ISO88591 => _default(id, ISO_8859_1.decode(&data, DecoderTrap::Strict)),
            TextEncoding::UTF16LE => _default(id, UTF_16LE.decode(&data, DecoderTrap::Strict)),
            TextEncoding::UTF16BE => _default(id, UTF_16BE.decode(&data, DecoderTrap::Strict)),
            TextEncoding::UTF8 => _default(id, UTF_8.decode(&data, DecoderTrap::Strict))
        };

        Ok(TEXT {
            text_encoding: text_encoding,
            text: self::trim(text)
        })
    }
}

impl FrameWriterDefault for TEXT {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        let text = match match self.text_encoding {
            TextEncoding::ISO88591 => ISO_8859_1.encode(self.text.as_str(), EncoderTrap::Strict),
            TextEncoding::UTF16LE => UTF_16LE.encode(self.text.as_str(), EncoderTrap::Strict),
            TextEncoding::UTF16BE => UTF_16BE.encode(self.text.as_str(), EncoderTrap::Strict),
            TextEncoding::UTF8 => UTF_8.encode(self.text.as_str(), EncoderTrap::Strict)
        } {
            Ok(text) => text,
            Err(msg) => return Err(Error::new(ErrorKind::InvalidInput, msg.to_owned().to_string()))
        };


        writable.write(&text)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TXXX {
    pub text_encoding: TextEncoding,
    pub description: String,
    pub value: String
}

impl FrameReaderDefault<TXXX> for TXXX {
    fn read(readable: &mut Readable) -> Result<TXXX> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let description = util::read_null_terminated(&text_encoding, readable)?;
        let value = readable.all_string()?;

        Ok(TXXX {
            text_encoding: text_encoding,
            description: description,
            value: value
        })
    }
}

impl FrameWriterDefault for TXXX {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        util::write_null_terminated(&self.text_encoding, self.description.as_str(), writable)?;
        writable.string(self.value.as_str())
    }
}

// TODO not yet tested!
// User defined URL link frame
#[derive(Clone, Debug, PartialEq)]
pub struct WXXX {
    pub text_encoding: TextEncoding,
    pub description: String,
    pub url: String
}

impl FrameReaderDefault<WXXX> for WXXX {
    fn read(readable: &mut Readable) -> Result<WXXX> {
        let text_encoding = util::to_encoding(readable.u8()?);
        let description = util::read_null_terminated(&text_encoding, readable)?;
        let url = readable.all_string()?;

        Ok(WXXX {
            text_encoding: text_encoding,
            description: description,
            url: url
        })
    }
}

impl FrameWriterDefault for WXXX {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.u8(util::from_encoding(&self.text_encoding))?;
        util::write_null_terminated(&self.text_encoding, self.description.as_str(), writable)?;
        writable.string(self.url.as_str())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OBJECT {
    pub data: Vec<u8>
}

impl FrameWriterDefault for OBJECT {
    fn write(&self, writable: &mut Writable<Cursor<Vec<u8>>>) -> Result<()> {
        writable.write(&self.data)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameData {
    //2.2 only
    BUF(BUF),
    //2.2 only
    CRM(CRM),
    //2.2 only
    PIC(PIC),

    AENC(AENC),
    APIC(APIC),
    ASPI(ASPI),
    COMM(COMM),
    COMR(COMR),
    ENCR(ENCR),
    // 2.3 only
    EQUA(EQUA),
    EQU2(EQU2),
    ETCO(ETCO),
    GEOB(GEOB),
    GRID(GRID),
    // 2.3 only
    IPLS(IPLS),
    LINK(LINK),
    MCDI(MCDI),
    MLLT(MLLT),
    OWNE(OWNE),
    PRIV(PRIV),
    PCNT(PCNT),
    POPM(POPM),
    POSS(POSS),
    RBUF(RBUF),
    // 2.3 only
    RVAD(RVA2),
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
    // 2.3 only
    TDAT(TEXT),
    TDEN(TEXT),
    TDLY(TEXT),
    TDOR(TEXT),
    TDRC(TEXT),
    TDRL(TEXT),
    TDTG(TEXT),
    TENC(TEXT),
    TEXT(TEXT),
    TFLT(TEXT),
    // 2.3 only
    TIME(TEXT),
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
    TORY(TEXT),
    TOWN(TEXT),
    TPE1(TEXT),
    TPE2(TEXT),
    TPE3(TEXT),
    TPE4(TEXT),
    TPOS(TEXT),
    TPRO(TEXT),
    TPUB(TEXT),
    TRCK(TEXT),
    // 2.3 only
    TRDA(TEXT),
    TRSN(TEXT),
    TRSO(TEXT),
    // 2.3 only
    TSIZ(TEXT),
    TSOA(TEXT),
    TSOP(TEXT),
    TSOT(TEXT),
    TSRC(TEXT),
    TSSE(TEXT),
    // 2.3 only
    TYER(TEXT),
    TSST(TEXT),
    TXXX(TXXX),
    UFID(UFID),
    USER(USER),
    USLT(USLT),
    WCOM(LINK),
    WCOP(LINK),
    WOAF(LINK),
    WOAR(LINK),
    WOAS(LINK),
    WORS(LINK),
    WPAY(LINK),
    WPUB(LINK),
    WXXX(WXXX),
    OBJECT(OBJECT),
    SKIP(String, Vec<u8>),
    INVALID(String)
}

// 2.2 mapping
// BUF
// CNT - PCNT
// COM - COMM
// CRA - AENC
// CRM -
// ETC - ETCO
// EQU - EQUA
// GEO - GEOB
// IPL - IPLS
// LNK - LINK
// MCI - MCDI
// MLL - MLLT
// PIC
// POP - POPM
// REV - RVRB
// RVA - RVAD
// SLT - SYLT
// STC - SYTC
// TAL - TALB
// TBP - TBPM
// TCM - TCOM
// TCO - TCON
// TCR - TCOP
// TDA - TDAT
// TDY - TDLY
// TEN - TENC
// TFT - TFLT
// TIM - TIME
// TKE - TKEY
// TLA - TLAN
// TLE - TLEN
// TMT - TMED
// TOA - TOPE
// TOF - TOFN
// TOL - TOLY
// TOR - TORY
// TOT - TOAL
// TP1 - TPE1
// TP2 - TPE2
// TP3 - TPE3
// TP4 - TPE4
// TPA - TPOS
// TPB - TPUB
// TRC - TSRC
// TRD - TRDA
// TRK - TRCK
// TSI - TSIZ
// TSS - TSSE
// TT1 - TIT1
// TT2 - TIT2
// TT3 - TIT1
// TXT - TEXT
// TXX - TXXX
// TYE - TYER
// UFI - UFID
// ULT - USLT
// WAF - WOAF
// WAR - WOAR
// WAS - WOAS
// WCM - WCOM
// WCP - WCOP
// WPB - WPUB
// WXX - WXXX
pub mod id {
    //
    // 2.2
    //
    pub const BUF_STR: &'static str = "BUF";
    pub const CNT_STR: &'static str = "CNT";
    pub const COM_STR: &'static str = "COM";
    pub const CRA_STR: &'static str = "CRA";
    pub const CRM_STR: &'static str = "CRM";
    pub const ETC_STR: &'static str = "ETC";
    pub const EQU_STR: &'static str = "EQU";
    pub const GEO_STR: &'static str = "GEO";
    pub const IPL_STR: &'static str = "IPL";
    pub const LNK_STR: &'static str = "LNK";
    pub const MCI_STR: &'static str = "MCI";
    pub const MLL_STR: &'static str = "MLL";
    pub const PIC_STR: &'static str = "PIC";
    pub const POP_STR: &'static str = "POP";
    pub const REV_STR: &'static str = "REV";
    pub const RVA_STR: &'static str = "RVA";
    pub const SLT_STR: &'static str = "SLT";
    pub const STC_STR: &'static str = "STC";
    pub const TAL_STR: &'static str = "TAL";
    pub const TBP_STR: &'static str = "TBP";
    pub const TCM_STR: &'static str = "TCM";
    pub const TCO_STR: &'static str = "TCO";
    pub const TCR_STR: &'static str = "TCR";
    pub const TDA_STR: &'static str = "TDA";
    pub const TDY_STR: &'static str = "TDY";
    pub const TEN_STR: &'static str = "TEN";
    pub const TFT_STR: &'static str = "TFT";
    pub const TIM_STR: &'static str = "TIM";
    pub const TKE_STR: &'static str = "TKE";
    pub const TLA_STR: &'static str = "TLA";
    pub const TLE_STR: &'static str = "TLE";
    pub const TMT_STR: &'static str = "TMT";
    pub const TOA_STR: &'static str = "TOA";
    pub const TOF_STR: &'static str = "TOF";
    pub const TOL_STR: &'static str = "TOL";
    pub const TOR_STR: &'static str = "TOR";
    pub const TOT_STR: &'static str = "TOT";
    pub const TP1_STR: &'static str = "TP1";
    pub const TP2_STR: &'static str = "TP2";
    pub const TP3_STR: &'static str = "TP3";
    pub const TP4_STR: &'static str = "TP4";
    pub const TPA_STR: &'static str = "TPA";
    pub const TPB_STR: &'static str = "TPB";
    pub const TRC_STR: &'static str = "TRC";
    pub const TRD_STR: &'static str = "TRD";
    pub const TRK_STR: &'static str = "TRK";
    pub const TSI_STR: &'static str = "TSI";
    pub const TSS_STR: &'static str = "TSS";
    pub const TT1_STR: &'static str = "TT1";
    pub const TT2_STR: &'static str = "TT2";
    pub const TT3_STR: &'static str = "TT3";
    pub const TXT_STR: &'static str = "TXT";
    pub const TXX_STR: &'static str = "TXX";
    pub const TYE_STR: &'static str = "TYE";
    pub const UFI_STR: &'static str = "UFI";
    pub const ULT_STR: &'static str = "ULT";
    pub const WAF_STR: &'static str = "WAF";
    pub const WAR_STR: &'static str = "WAR";
    pub const WAS_STR: &'static str = "WAS";
    pub const WCM_STR: &'static str = "WCM";
    pub const WCP_STR: &'static str = "WCP";
    pub const WPB_STR: &'static str = "WPB";
    pub const WXX_STR: &'static str = "WXX";

    //
    // 2.3 & 2.4
    //
    pub const AENC_STR: &'static str = "AENC";
    pub const APIC_STR: &'static str = "APIC";
    pub const ASPI_STR: &'static str = "ASPI";
    pub const COMM_STR: &'static str = "COMM";
    pub const COMR_STR: &'static str = "COMR";
    pub const ENCR_STR: &'static str = "ENCR";
    pub const EQU2_STR: &'static str = "EQU2";
    // 2.3 only
    pub const EQUA_STR: &'static str = "EQUA";
    pub const ETCO_STR: &'static str = "ETCO";
    pub const GEOB_STR: &'static str = "GEOB";
    pub const GRID_STR: &'static str = "GRID";
    // 2.3 only
    pub const IPLS_STR: &'static str = "IPLS";
    pub const LINK_STR: &'static str = "LINK";
    pub const MCDI_STR: &'static str = "MCDI";
    pub const MLLT_STR: &'static str = "MLLT";
    pub const OWNE_STR: &'static str = "OWNE";
    pub const PRIV_STR: &'static str = "PRIV";
    pub const PCNT_STR: &'static str = "PCNT";
    pub const POPM_STR: &'static str = "POPM";
    pub const POSS_STR: &'static str = "POSS";
    pub const RBUF_STR: &'static str = "RBUF";
    // 2.3 only
    pub const RVAD_STR: &'static str = "RVAD";
    pub const RVA2_STR: &'static str = "RVA2";
    pub const RVRB_STR: &'static str = "RVRB";
    pub const SEEK_STR: &'static str = "SEEK";
    pub const SIGN_STR: &'static str = "SIGN";
    pub const SYLT_STR: &'static str = "SYLT";
    pub const SYTC_STR: &'static str = "SYTC";
    pub const TALB_STR: &'static str = "TALB";
    pub const TBPM_STR: &'static str = "TBPM";
    pub const TCOM_STR: &'static str = "TCOM";
    pub const TCON_STR: &'static str = "TCON";
    pub const TCOP_STR: &'static str = "TCOP";
    // 2.3 only
    pub const TDAT_STR: &'static str = "TDAT";
    pub const TDEN_STR: &'static str = "TDEN";
    pub const TDLY_STR: &'static str = "TDLY";
    pub const TDOR_STR: &'static str = "TDOR";
    pub const TDRC_STR: &'static str = "TDRC";
    pub const TDTG_STR: &'static str = "TDTG";
    pub const TDRL_STR: &'static str = "TDRL";
    pub const TENC_STR: &'static str = "TENC";
    pub const TEXT_STR: &'static str = "TEXT";
    pub const TFLT_STR: &'static str = "TFLT";
    // 2.3 only
    pub const TIME_STR: &'static str = "TIME";
    pub const TIPL_STR: &'static str = "TIPL";
    pub const TIT1_STR: &'static str = "TIT1";
    pub const TIT2_STR: &'static str = "TIT2";
    pub const TIT3_STR: &'static str = "TIT3";
    pub const TKEY_STR: &'static str = "TKEY";
    pub const TLAN_STR: &'static str = "TLAN";
    pub const TLEN_STR: &'static str = "TLEN";
    pub const TMCL_STR: &'static str = "TMCL";
    pub const TMED_STR: &'static str = "TMED";
    pub const TMOO_STR: &'static str = "TMOO";
    pub const TOAL_STR: &'static str = "TOAL";
    pub const TOFN_STR: &'static str = "TOFN";
    pub const TOLY_STR: &'static str = "TOLY";
    pub const TOPE_STR: &'static str = "TOPE";
    pub const TORY_STR: &'static str = "TORY";
    pub const TOWN_STR: &'static str = "TOWN";
    pub const TPE1_STR: &'static str = "TPE1";
    pub const TPE2_STR: &'static str = "TPE2";
    pub const TPE3_STR: &'static str = "TPE3";
    pub const TPE4_STR: &'static str = "TPE4";
    pub const TPOS_STR: &'static str = "TPOS";
    pub const TPRO_STR: &'static str = "TPRO";
    pub const TPUB_STR: &'static str = "TPUB";
    pub const TRCK_STR: &'static str = "TRCK";
    pub const TRDA_STR: &'static str = "TRDA";
    pub const TRSN_STR: &'static str = "TRSN";
    pub const TRSO_STR: &'static str = "TRSO";
    // 2.3 only
    pub const TSIZ_STR: &'static str = "TSIZ";
    pub const TSOA_STR: &'static str = "TSOA";
    pub const TSOP_STR: &'static str = "TSOP";
    pub const TSOT_STR: &'static str = "TSOT";
    pub const TSRC_STR: &'static str = "TSRC";
    pub const TSSE_STR: &'static str = "TSSE";
    // 2.3 only
    pub const TYER_STR: &'static str = "TYER";
    pub const TSST_STR: &'static str = "TSST";
    pub const TXXX_STR: &'static str = "TXXX";
    pub const UFID_STR: &'static str = "UFID";
    pub const USER_STR: &'static str = "USER";
    pub const USLT_STR: &'static str = "USLT";
    pub const WCOM_STR: &'static str = "WCOM";
    pub const WCOP_STR: &'static str = "WCOP";
    pub const WOAF_STR: &'static str = "WOAF";
    pub const WOAR_STR: &'static str = "WOAR";
    pub const WOAS_STR: &'static str = "WOAS";
    pub const WORS_STR: &'static str = "WORS";
    pub const WPAY_STR: &'static str = "WPAY";
    pub const WPUB_STR: &'static str = "WPUB";
    pub const WXXX_STR: &'static str = "WXXX";
}