extern crate encoding;
extern crate regex;

use self::encoding::{Encoding, DecoderTrap, EncoderTrap};
use self::encoding::all::{ISO_8859_1, UTF_16LE, UTF_16BE, UTF_8};

use rw::{Readable, Writable};
use frame::types::*;

use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind, Result, Write};
use std::vec::Vec;

//
// It map between logical type and real type of rust.
//
macro_rules! real_type {
    (String) => { String };
    (VersionString) => { String };
    (EncodedString) => { String };
    (NonUtf16String) => { String };
    (Utf16String) => { String };

    (Unsigned8) => { u8 };
    (Unsigned16) => { u16 };
    (Unsigned24) => { u32 };
    (Unsigned32) => { u32 };
    (Synchsafe) => { u32 };
    
    (Bytes) => { Vec<u8> };

    (TextEncoding) => { TextEncoding };
    (PictureType) => { PictureType };
    (ReceivedAs) => { ReceivedAs };
    (InterpolationMethod) => { InterpolationMethod };
    (TimestampFormat) => { TimestampFormat };
    (ContentType) => { ContentType };
}

macro_rules! convert_to_string {
    (String, $value:expr) => { $value };
    (VersionString, $value:expr) => { $value };
    (EncodedString, $value:expr) => { $value };
    (NonUtf16String, $value:expr) => { $value };
    (Utf16String, $value:expr) => { $value };

    (Unsigned8, $value:expr) => { $value.to_string() };
    (Unsigned16, $value:expr) => { $value.to_string() };
    (Unsigned24, $value:expr) => { $value.to_string() };
    (Unsigned32, $value:expr) => { $value.to_string() };
    (Synchsafe, $value:expr) => { $value.to_string() };

    (Bytes, $value:expr) => { String::new() };

    (TextEncoding, $value:expr) => { format!("{:?}", $value) };
    (PictureType, $value:expr) => { format!("{:?}", $value) };
    (ReceivedAs, $value:expr) => { format!("{:?}", $value) };
    (InterpolationMethod, $value:expr) => { format!("{:?}", $value) };
    (TimestampFormat, $value:expr) => { format!("{:?}", $value) };
    (ContentType, $value:expr) => { format!("{:?}", $value) };
}

//
// It read a frame bytes by logical type.
//
macro_rules! frame_read {
    (String, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            let s = match $value {
                -1 => String::new(),
                0 => $readable.all_string()?,
                _ => $readable.read_string($value)?
            };
            trace!("String, value: {:?}, {}", $value, s);

            s
        }
    };

    (VersionString, $value:expr, $readable:expr, $version:expr) => {
        {
            let id = match $version {
                2 | 3 => $readable.read_string(3)?,
                _ => $readable.read_string(4)?
            };
            trace!("VersionString, {}, {}", $version, id);

            id
        }
    };

    (Unsigned8, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            $readable.read_u8()?
        }
    };

    (Unsigned16, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            $readable.read_u16()?
        }
    };

    (Unsigned24, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            $readable.read_u24()?
        }
    };
    
    (Unsigned32, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            $readable.read_u32()?
        }
    };

    (Synchsafe, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            $readable.read_synchsafe()?
        }
    };

    (NonUtf16String, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            let r = $readable.read_non_utf16_string()?;
            trace!("NonUtf16String {}", r);

            r
        }
    };

    (Utf16String, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            let r = $readable.read_utf16_string()?;
            trace!("Utf16String {}", r);

            r
        }
    };

    (Bytes, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            match $value {
                -1 => {
                    let v: Vec<u8> = Vec::new();
                    v
                },
                0 => $readable.all_bytes()?,
                _ => $readable.read_bytes($value)?
            }
        }
    };

    (TextEncoding, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            types::to_encoding($readable.read_u8()?)
        }
    };

    (PictureType, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            types::to_picture_type($readable.read_u8()?)
        }
    };

    (ReceivedAs, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            types::to_received_as($readable.read_u8()?)
        }
    };

    (InterpolationMethod, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            types::to_interpolation_method($readable.read_u8()?)
        }
    };

    (TimestampFormat, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            types::to_timestamp_format($readable.read_u8()?)
        }
    };

    (ContentType, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;
            types::to_content_type($readable.read_u8()?)
        }
    };

    (EncodedString, $value:expr, $readable:expr, $version:expr) => {
        {
            let _ = $version;

            let curr_pos = $readable.skip_bytes(0)?;
            //
            // The encoding of frame always id located at most first bit.
            //
            let _ = $readable.position(0)?;

            let encoding = $readable.read_u8()?;
            let _ = $readable.position(curr_pos)?;

            fn decode(decode: ::std::result::Result<String, ::std::borrow::Cow<'static, str>>) -> String {
                match decode {
                    Ok(text) => text,
                    Err(e) => {
                        debug!("Encoding error {:?}", e);
                        "".to_string()
                    }
                }
            }
            
            match encoding {
                0 => {
                    let data = $readable.read_non_utf16_bytes()?;
                    decode(ISO_8859_1.decode(&data, DecoderTrap::Strict))
                }
                1 => {
                    let mut data = $readable.read_utf16_bytes()?;
                    data.push(0);
                    decode(UTF_16LE.decode(&data[2..], DecoderTrap::Strict))
                }
                2 => {
                    let data = $readable.read_utf16_bytes()?;
                    decode(UTF_16BE.decode(&data, DecoderTrap::Strict))
                }
                3 => {
                    let data = $readable.read_non_utf16_bytes()?;
                    decode(UTF_8.decode(&data, DecoderTrap::Strict))
                }
                _ => {
                    let data = $readable.read_non_utf16_bytes()?;
                    decode(ISO_8859_1.decode(&data, DecoderTrap::Strict))
                }
            }
        }
    };
}

macro_rules! frame_write {
    (String, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            match $value {
                -1 => {
                    // do nothing
                },
                0 => {
                    $writable.write_string(&$_self.$attr_name)?
                },
                _ => {
                    $writable.write_string(&$_self.$attr_name[0..$value])?
                }
            }
        }
    };

    (VersionString, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            match $version {
                2 | 3 => $writable.write_string(&$_self.$attr_name[0..3])?,
                _ => $writable.write_string(&$_self.$attr_name[0..4])?
            }
        }
    };

    (Unsigned8, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8($_self.$attr_name)?
        }
    };

    (Unsigned16, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u16($_self.$attr_name)?
        }
    };

    (Unsigned24, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u24($_self.$attr_name)?
        }
    };

    (Unsigned32, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u32($_self.$attr_name)?
        }
    };

    (Synchsafe, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_synchsafe($_self.$attr_name)?
        }
    };

    (NonUtf16String, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_non_utf16_string($_self.$attr_name.as_str())?
        }
    };

    (Utf16String, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_utf16_string($_self.$attr_name.as_str())?
        }
    };

    (Bytes, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write(&$_self.$attr_name)?
        }
    };

    (TextEncoding, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8(types::from_encoding(&$_self.$attr_name))?
        }
    };

    (PictureType, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8(types::from_picture_type(&$_self.$attr_name))?
        }
    };

    (ReceivedAs, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8(types::from_received_as(&$_self.$attr_name))?
        }
    };

    (InterpolationMethod, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8(types::from_interpolation_method(&$_self.$attr_name))?
        }
    };

    (TimestampFormat, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8(types::from_timestamp_format(&$_self.$attr_name))?
        }
    };

    (ContentType, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;
            $writable.write_u8(types::from_content_type(&$_self.$attr_name))?
        }
    };

    (EncodedString, $_self:expr, $attr_name:ident, $value:expr, $writable:expr, $version:expr) => {
        {
            let _ = $version;

            // 
            // The name of encoding field is fixed name with 'text_encoding'.
            //
            match $_self.text_encoding {
                TextEncoding::ISO88591 | TextEncoding::UTF8 => {
                    $writable.write_non_utf16_string($_self.$attr_name.as_str())?
                },
                _ => {
                    $writable.write_utf16_string($_self.$attr_name.as_str())?
                }
            }
        }
    };

}

//
// It declare struct and implement read/write operation.
//
// ex)
// struct <frame name> {
//
//      <frame prpoerty> : <logical type> = <parameter of logical type>
// }
//
macro_rules! id3 {
    (
        $name:ident {
            $( $attr_name:ident : $attr_type:ident = $value:expr ),*
        }
    ) => (
        
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            $( pub $attr_name : real_type!($attr_type) ),*
        }

        impl $name {
            pub fn read(readable: &mut Readable, version: u8, id: &str) -> Result<$name> {

                let _ = id;

                let ret = $name {
                    $(
                        $attr_name : frame_read!($attr_type, $value, readable, version)
                    ),*
                };

                Ok(ret)
            }

            pub fn write(&self, writable: &mut Writable, version: u8) -> Result<()> {

                $(
                    frame_write!($attr_type, self, $attr_name, $value, writable, version) 
                );*

                ;
                Ok(())
            }
        }

        impl Look for $name {
            fn to_map(&self) -> Result<HashMap<&str, String>> {

                let mut map = HashMap::new();

                $(
                    let key = stringify!($attr_name);
                    let value = convert_to_string!($attr_type, &self.$attr_name);
                    map.insert(key, value.to_string());
                );*

                ;
                Ok(map)
            }

            fn inside<T>(&self, callback: T) where T: Fn(&str, String) -> bool {

                $(
                    let key = stringify!($attr_name);
                    let value = convert_to_string!($attr_type, &self.$attr_name);

                    if callback(key, value.to_string()) == false {
                        return;
                    }
                );*
            }
        }
    );

    (
        $name:ident {
            $( $attr_name:ident : $attr_type:ident = $value:expr ),+,
        }
    ) => (
        id3!( $name { $( $attr_name : $attr_type = $value ),+ } );
    );
}

pub trait FlagAware<T> {
    fn has_flag(&self, flag: T) -> bool;
    fn set_flag(&mut self, flag: T);
}

pub trait FrameHeaderDefault {
    fn id(&self) -> String;
    fn size(&self) -> u32;
}

pub trait Look {
    fn to_map(&self) -> Result<HashMap<&str, String>>;
    fn inside<T>(&self, callback: T) where T: Fn(&str, String) -> bool;
}

///
/// # ID3V2 Header
///
/// - [V2.3](http://id3.org/id3v2.3.0#ID3v2_header)
/// - [V2.4](http://id3.org/id3v2.4.0-structure) > 3.1. ID3v2 header
///
id3!(Head {
    tag_id: String = 3,
    version: Unsigned8 = -1,
    minor_version: Unsigned8 = -1,
    flag: Unsigned8 = -1,
    size: Synchsafe = -1,
});

///
/// # Frame Header V2.2
///
/// [See](http://id3.org/id3v2-00) > 3.2. ID3v2 frames overview
///
id3!(FrameHeaderV2 {
    id: String = 3,
    size: Unsigned24 = -1,
});

///
/// # Frame Header V2.3
///
/// [See](http://id3.org/id3v2.3.0#ID3v2_frame_overview)
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameHeaderV3 {
    pub id: String,
    pub size: u32,
    pub status_flag: u8,
    pub encoding_flag: u8,
}

impl FrameHeaderV3 {
    pub fn read(readable: &mut Readable, version: u8, id: &str) -> Result<Self> {
        let _ = version;
        let _ = id;

        let id = readable.read_string(4)?;
        let size = readable.read_u32()?;
        let status_flag = readable.read_u8()?;
        let encoding_flag = readable.read_u8()?;

        Ok(FrameHeaderV3 {
            id: id,
            size: size,
            status_flag: status_flag,
            encoding_flag: encoding_flag,
        })
    }

    pub fn write(&self, writable: &mut Cursor<Vec<u8>>, version: u8) -> Result<()> {
        let _ = version;

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
        writable.write_string(self.id.as_str())?;
        writable.write_u32(self.size + ext_size)?;
        writable.write_u8(self.status_flag)?;
        writable.write_u8(self.encoding_flag)?;

        if self.has_flag(FrameHeaderFlag::GroupIdentity) {
            writable.write_u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::Encryption) {
            writable.write_u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::Compression) {
            writable.write_u32(0)?;
        }

        Ok(())
    }
}



///
/// # Frame Header V2.4
///
/// [See](http://id3.org/id3v2.4.0-structure) > 4. ID3v2 frames overview
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameHeaderV4 {
    pub id: String,
    pub size: u32,
    pub status_flag: u8,
    pub encoding_flag: u8,
}

impl FrameHeaderV4 {
    pub fn read(readable: &mut Readable, version: u8, id: &str) -> Result<Self> {
        let _ = version;
        let _ = id;

        let id = readable.read_string(4)?;
        let size = readable.read_synchsafe()?;
        let status_flag = readable.read_u8()?;
        let encoding_flag = readable.read_u8()?;

        Ok(FrameHeaderV4 {
            id: id,
            size: size,
            status_flag: status_flag,
            encoding_flag: encoding_flag,
        })
    }

    pub fn write(&self, writable: &mut Cursor<Vec<u8>>, version: u8) -> Result<()> {
        let _ = version;

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

        writable.write_string(self.id.as_str())?;
        writable.write_synchsafe(self.size + ext_size)?;
        writable.write_u8(self.status_flag)?;
        writable.write_u8(self.encoding_flag)?;

        if self.has_flag(FrameHeaderFlag::GroupIdentity) {
            writable.write_u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::Encryption) {
            writable.write_u8(0)?;
        }
        if self.has_flag(FrameHeaderFlag::DataLength) {
            writable.write_u32(0)?;
        }

        Ok(())
    }
}

///
/// # Frame Header Flags
///
/// - [V2.3](http://id3.org/id3v2.3.0#Frame_header_flags)
/// - [V2.4](http://id3.org/id3v2.4.0-structure) > 3.1. ID3v2 header
///
/// ## Note
///
/// Head level 'Unsynchronisation' does not work on V2.4
/// - Reference File: "<ROOT>/test-resources/v2.4-unsync.mp3"
///
impl FlagAware<HeadFlag> for Head {
    fn has_flag(&self, flag: HeadFlag) -> bool {
        match self.version {
            2 => {
                match flag {
                    HeadFlag::Unsynchronisation => self.flag & types::BIT7 != 0,
                    HeadFlag::Compression => self.flag & types::BIT6 != 0,
                    _ => false,
                }
            }
            3 => {
                match flag {
                    HeadFlag::Unsynchronisation => self.flag & types::BIT7 != 0,
                    HeadFlag::ExtendedHeader => self.flag & types::BIT6 != 0,
                    HeadFlag::ExperimentalIndicator => self.flag & types::BIT5 != 0,
                    _ => false,
                }
            }
            4 => {
                match flag {
                    //
                    // HeadFlag::Unsynchronisation => self.flag & types::BIT7 != 0,
                    HeadFlag::ExtendedHeader => self.flag & types::BIT6 != 0,
                    HeadFlag::ExperimentalIndicator => self.flag & types::BIT5 != 0,
                    HeadFlag::FooterPresent => self.flag & types::BIT4 != 0,
                    _ => false,
                }
            }
            _ => {
                warn!("Header.has_flag=> Unknown version!");
                false
            }
        }
    }

    fn set_flag(&mut self, flag: HeadFlag) {
        match self.version {
            2 => {
                match flag {
                    HeadFlag::Unsynchronisation => self.flag = self.flag | types::BIT7,
                    HeadFlag::Compression => self.flag = self.flag | types::BIT6,
                    _ => (),
                }
            }
            3 => {
                match flag {
                    HeadFlag::Unsynchronisation => self.flag = self.flag | types::BIT7,
                    HeadFlag::ExtendedHeader => self.flag = self.flag | types::BIT6,
                    HeadFlag::ExperimentalIndicator => self.flag = self.flag | types::BIT5,
                    _ => (),
                }
            }
            4 => {
                match flag {
                    //
                    // HeadFlag::Unsynchronisation => self.flag & util::BIT7 != 0,
                    HeadFlag::ExtendedHeader => self.flag = self.flag | types::BIT6,
                    HeadFlag::ExperimentalIndicator => self.flag = self.flag | types::BIT5,
                    HeadFlag::FooterPresent => self.flag = self.flag | types::BIT4,
                    _ => (),
                }
            }
            _ => {
                warn!("Header.has_flag=> Unknown version!");
            }
        }
    }
}

///
/// # No flags
///
/// There is no flag for 2.2 frame.
///
impl FlagAware<FrameHeaderFlag> for FrameHeaderV2 {
    #[allow(unused_variables)]
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        return false;
    }
    #[allow(unused_variables)]
    fn set_flag(&mut self, flag: FrameHeaderFlag) {}
}

///
/// # Frame header flags V2.3
///
/// [See](http://id3.org/id3v2.3.0#Frame_header_flags)
///
impl FlagAware<FrameHeaderFlag> for FrameHeaderV3 {
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        match flag {
            FrameHeaderFlag::TagAlter => self.status_flag & types::BIT7 != 0,
            FrameHeaderFlag::FileAlter => self.status_flag & types::BIT6 != 0,
            FrameHeaderFlag::ReadOnly => self.status_flag & types::BIT5 != 0,
            FrameHeaderFlag::Compression => self.encoding_flag & types::BIT7 != 0,
            FrameHeaderFlag::Encryption => self.encoding_flag & types::BIT6 != 0,
            FrameHeaderFlag::GroupIdentity => self.encoding_flag & types::BIT5 != 0,
            _ => false,
        }
    }

    fn set_flag(&mut self, flag: FrameHeaderFlag) {
        match flag {
            FrameHeaderFlag::TagAlter => self.status_flag = self.status_flag | types::BIT7,
            FrameHeaderFlag::FileAlter => self.status_flag = self.status_flag | types::BIT6,
            FrameHeaderFlag::ReadOnly => self.status_flag = self.status_flag | types::BIT5,
            FrameHeaderFlag::Compression => self.encoding_flag = self.encoding_flag | types::BIT7,
            FrameHeaderFlag::Encryption => self.encoding_flag = self.encoding_flag | types::BIT6,
            FrameHeaderFlag::GroupIdentity => self.encoding_flag = self.encoding_flag | types::BIT5,
            _ => (),
        }
    }
}

///
/// # Frame header flags V2.4
///
/// [See](http://id3.org/id3v2.4.0-structure) > 4.1 Frame header flags
///
impl FlagAware<FrameHeaderFlag> for FrameHeaderV4 {
    // http://id3.org/id3v2.4.0-structure > 4.1. Frame header flags
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        match flag {
            FrameHeaderFlag::TagAlter => self.status_flag & types::BIT6 != 0,
            FrameHeaderFlag::FileAlter => self.status_flag & types::BIT5 != 0,
            FrameHeaderFlag::ReadOnly => self.status_flag & types::BIT4 != 0,
            FrameHeaderFlag::GroupIdentity => self.encoding_flag & types::BIT6 != 0,
            FrameHeaderFlag::Compression => self.encoding_flag & types::BIT3 != 0,
            FrameHeaderFlag::Encryption => self.encoding_flag & types::BIT2 != 0,
            FrameHeaderFlag::Unsynchronisation => self.encoding_flag & types::BIT1 != 0,
            FrameHeaderFlag::DataLength => self.encoding_flag & types::BIT0 != 0,
        }
    }

    fn set_flag(&mut self, flag: FrameHeaderFlag) {
        match flag {
            FrameHeaderFlag::TagAlter => self.status_flag = self.status_flag | types::BIT6,
            FrameHeaderFlag::FileAlter => self.status_flag = self.status_flag | types::BIT5,
            FrameHeaderFlag::ReadOnly => self.status_flag = self.status_flag | types::BIT4,
            FrameHeaderFlag::GroupIdentity => self.encoding_flag = self.encoding_flag | types::BIT6,
            FrameHeaderFlag::Compression => self.encoding_flag = self.encoding_flag | types::BIT3,
            FrameHeaderFlag::Encryption => self.encoding_flag = self.encoding_flag | types::BIT2,
            FrameHeaderFlag::Unsynchronisation => {
                self.encoding_flag = self.encoding_flag | types::BIT1
            }
            FrameHeaderFlag::DataLength => self.encoding_flag = self.encoding_flag | types::BIT0,
        }
    }
}

///
/// # Frame 1.0
///
/// [See](https://en.wikipedia.org/wiki/ID3#ID3v1)
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame1 {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: String,
    pub genre: String,
}

impl Frame1 {
    pub fn read(readable: &mut Readable) -> Result<Self> {
        // offset 3
        let title = types::to_iso8859_1(&readable.read_bytes(30)?).trim().to_string();
        // offset 33
        let artist = types::to_iso8859_1(&readable.read_bytes(30)?).trim().to_string();
        // offset 63
        let album = types::to_iso8859_1(&readable.read_bytes(30)?).trim().to_string();
        // offset 93
        let year = types::to_iso8859_1(&readable.read_bytes(4)?).trim().to_string();
        // goto track marker offset
        readable.skip_bytes(28)?;
        // offset 125
        let track_marker = readable.read_u8()?;
        // offset 126
        let _track = readable.read_u8()? & 0xff;
        // offset 127
        let genre = (readable.read_u8()? & 0xff).to_string();
        // goto comment offset
        readable.skip_bytes(-31)?;

        let (comment, track) = if track_marker != 0 {
            (types::to_iso8859_1(&readable.read_bytes(30)?).trim().to_string(), String::new())
        } else {
            (types::to_iso8859_1(&readable.read_bytes(28)?).trim().to_string(),
             if _track == 0 {
                 String::new()
             } else {
                 _track.to_string()
             })
        };

        Ok(Frame1 {
            title: title,
            artist: artist,
            album: album,
            year: year,
            comment: comment,
            track: track,
            genre: genre,
        })
    }

    pub fn write(&self, writable: &mut Cursor<Vec<u8>>) -> Result<()> {
        writable.write_string("TAG")?;
        writable.write(&types::from_iso8859_1(&self.title, 30))?;
        writable.write(&types::from_iso8859_1(&self.artist, 30))?;
        writable.write(&types::from_iso8859_1(&self.album, 30))?;
        writable.write(&types::from_iso8859_1(&self.year, 4))?;
        writable.write(&types::from_iso8859_1(&self.comment, 28))?;
        writable.write_u8(0)?; //track marker
        match self.track.as_str().parse::<u8>() {
            Ok(v) => writable.write_u8(v)?,
            Err(_) => writable.write_u8(0)?,
        };
        match self.genre.as_str().parse::<u8>() {
            Ok(v) => writable.write_u8(v)?,
            Err(_) => writable.write_u8(0)?,
        };

        Ok(())
    }
}

///
/// # Define Frame Header
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FrameHeader {
    V22(FrameHeaderV2),
    V23(FrameHeaderV3),
    V24(FrameHeaderV4),
}

impl FrameHeaderDefault for FrameHeader {
    fn id(&self) -> String {
        match self {
            &FrameHeader::V22(ref header) => header.id.to_string(),
            &FrameHeader::V23(ref header) => header.id.to_string(),
            &FrameHeader::V24(ref header) => header.id.to_string(),
        }
    }

    fn size(&self) -> u32 {
        match self {
            &FrameHeader::V22(ref header) => header.size,
            &FrameHeader::V23(ref header) => header.size,
            &FrameHeader::V24(ref header) => header.size,
        }
    }
}

impl FlagAware<FrameHeaderFlag> for FrameHeader {
    fn has_flag(&self, flag: FrameHeaderFlag) -> bool {
        match self {
            &FrameHeader::V22(ref header) => header.has_flag(flag),
            &FrameHeader::V23(ref header) => header.has_flag(flag),
            &FrameHeader::V24(ref header) => header.has_flag(flag),
        }
    }

    fn set_flag(&mut self, flag: FrameHeaderFlag) {
        match self {
            &mut FrameHeader::V22(ref mut header) => header.set_flag(flag),
            &mut FrameHeader::V23(ref mut header) => header.set_flag(flag),
            &mut FrameHeader::V24(ref mut header) => header.set_flag(flag),
        }
    }
}

///
/// # Recommended buffer size
///
/// > Not yet tested!
///
id3!(BUF {
    buffer_size: Unsigned24 = -1,
    embedded_info_flag: Unsigned8 = -1,
    offset_to_next_tag: Unsigned32 = -1,
});

///
/// # Encrypted meta frame
///
/// > Not yet tested!
///
id3!(CRM {
    owner_identifier: NonUtf16String = -1,
    content: NonUtf16String = -1,
    encrypted_datablock: Bytes = 0,
});

///
/// # Attached picture
///
id3!(PIC {
    text_encoding: TextEncoding = -1,
    image_format: String = 3,
    picture_type: PictureType = -1,
    description: EncodedString = -1,
    picture_data: Bytes = 0,
});

///
/// Audio encryption
///
id3!(AENC {
    owner_identifier: NonUtf16String = -1,
    preview_start: Unsigned16 = -1,
    preview_end: Unsigned16 = -1,
    encryption_info: Bytes = 0,
});

///
/// Attached picture
///
///**Not yet tested!**
///
id3!(APIC {
    text_encoding: TextEncoding = -1,
    mime_type: NonUtf16String = -1,
    picture_type: PictureType = -1,
    description: EncodedString = -1,
    picture_data: Bytes = 0,
});

///
/// Audio seek point index
///
///**Not yet tested!**
///
id3!(ASPI {
    indexed_data_start: Unsigned32 = -1,
    indexed_data_length: Unsigned32 = -1,
    number_of_index_points: Unsigned16 = -1,
    bit_per_index_point: Unsigned8 = -1,
    fraction_at_index: Unsigned8 = -1,
});

///
/// Comments
///
id3!(COMM {
    text_encoding: TextEncoding = -1,
    language: String = 3,
    short_description: EncodedString = -1,
    actual_text: EncodedString = -1,
});

///
/// Commercial frame
///
///**Not yet tested!**
///
id3!(COMR {
    text_encoding: TextEncoding = -1,
    price_string: NonUtf16String = -1,
    valid_until: String = 8,
    contact_url: NonUtf16String = -1,
    received_as: ReceivedAs = -1,
    name_of_seller: Utf16String = -1,
    description: Utf16String = -1,
    picture_mime_type: NonUtf16String = -1,
    seller_logo: Bytes = 0,
});

///
/// Encryption method registration
///
///**Not yet tested!**
///
id3!(ENCR {
    owner_identifier: NonUtf16String = -1,
    method_symbol: Unsigned8 = -1,
    encryption_data: Bytes = 0,
});

///
/// Equalisation
///
///**Not yet tested!**
///**Not yet implemented!**
///
id3!(EQUA { data: Bytes = 0 });

///
/// Equalisation (2)
///
///**Not yet tested!**
///
id3!(EQU2 {
    interpolation_method: InterpolationMethod = -1,
    identification: NonUtf16String = -1,
});

///
/// General encapsulated object
///
id3!(GEOB {
    text_encoding: TextEncoding = -1,
    mime_type: NonUtf16String = -1,
    filename: EncodedString = -1,
    content_description: EncodedString = -1,
    encapsulation_object: Bytes = 0,
});

///
/// Group identification registration
///
///**Not yet tested!**
///
id3!(GRID {
    owner_identifier: NonUtf16String = -1,
    group_symbol: Unsigned8 = -1,
    group_dependent_data: Bytes = 0,
});

///
/// Involved people list
///
id3!(IPLS {
    text_encoding: TextEncoding = -1,
    people_list_strings: EncodedString = -1,
});

///
/// Linked information
///
id3!(LINK {
    frame_identifier: VersionString = -1,
    url: NonUtf16String = -1,
    additional_data: String = 0,
});

///
/// Music CD identifier
///
id3!(MCDI { cd_toc: Bytes = 0 });

///
/// # MPEG location lookup table
///
/// > Not yet tested!
/// > Not yet implemented!
///
id3!(MLLT { data: Bytes = 0 });

///
/// Ownership frame
///
///**Not yet tested!**
///
id3!(OWNE {
    text_encoding: TextEncoding = -1,
    price_paid: NonUtf16String = -1,
    date_of_purch: String = 4,
    seller: EncodedString = -1,
});

///
/// Private frame
///
///**Not yet tested!**
///
id3!(PRIV {
    owner_identifier: NonUtf16String = -1,
    private_data: Bytes = 0,
});

///
/// Play counter
///
///**It support that only the 32-bit unsigned integer type**
///
id3!(PCNT { counter: Unsigned32 = -1 });

///
/// Popularimeter
///
///**Not yet tested!**
///`counter`: support that only the 32-bit unsigned integer type
///
id3!(POPM {
    email_to_user: NonUtf16String = -1,
    rating: Unsigned8 = -1,
    // NOTE it support that only the 32-bit unsigned integer type.
    counter: Unsigned32 = -1,
});

///
/// Position synchronisation frame
///
///**Not yet tested!**
///
id3!(POSS {
    timestamp_format: TimestampFormat = -1,
    // TODO not yet implemented!
    position: Bytes = 0,
});

///
/// Recommended buffer size
///
///**Not yet tested!**
///
id3!(RBUF {
    buffer_size: Unsigned24 = -1,
    embedded_info_flag: Unsigned8 = -1,
    offset_to_next_tag: Unsigned32 = -1,
});

///
/// Relative volume adjustment (2)
///
///**Not yet tested!**
///**Not yet implemented!**
///
id3!(RVA2 { data: Bytes = 0 });

///
/// Reverb
///
///**Not yet tested!**
///
id3!(RVRB {
    reverb_left: Unsigned16 = -1,
    reverb_right: Unsigned16 = -1,
    reverb_bounce_left: Unsigned8 = -1,
    reverb_bounce_right: Unsigned8 = -1,
    reverb_feedback_left_to_left: Unsigned8 = -1,
    reverb_feedback_left_to_right: Unsigned8 = -1,
    reverb_feedback_right_to_right: Unsigned8 = -1,
    reverb_feedback_right_to_left: Unsigned8 = -1,
    premix_left_to_right: Unsigned8 = -1,
    premix_right_to_left: Unsigned8 = -1,
});

///
/// Seek frame
///
///**Not yet tested!**
///
id3!(SEEK { next_tag: String = 0 });

///
/// Signature frame
///
///**Not yet tested!**
///
id3!(SIGN {
    group_symbol: Unsigned8 = -1,
    signature: Bytes = 0,
});

///
/// Synchronised lyric/text
///
///**Not yet tested!**
///
id3!(SYLT {
    text_encoding: TextEncoding = -1,
    language: String = 3,
    timestamp_format: TimestampFormat = -1,
    content_type: ContentType = -1,
    content_descriptor: EncodedString = -1,
});

///
/// Synchronised tempo codes
///
///**Not yet tested!**
///
id3!(SYTC {
    timestamp_format: TimestampFormat = -1,
    tempo_data: Bytes = 0,
});

///
/// Unique file identifier
///
///**Not yet tested!**
///
id3!(UFID {
    owner_identifier: NonUtf16String = -1,
    identifier: Bytes = 0,
});

///
/// Terms of use
///
///**Not yet tested!**
///
id3!(USER {
    text_encoding: TextEncoding = -1,
    language: String = 3,
    actual_text: EncodedString = -1,
});

///
/// Unsynchronised lyric/text transcription
///
///**Not yet tested!**
///
id3!(USLT {
    text_encoding: TextEncoding = -1,
    language: String = 3,
    content_descriptor: EncodedString = -1,
    lyrics: EncodedString = -1,
});

///
/// User defined text information frame
///
id3!(TXXX {
    text_encoding: TextEncoding = -1,
    description: EncodedString = -1,
    value: EncodedString = -1,
});

///
/// User defined URL link frame
///
///**Not yet tested!**
///
id3!(WXXX {
    text_encoding: TextEncoding = -1,
    description: EncodedString = -1,
    url: String = 0,
});

///
/// Write anonymous bytes
///
id3!(OBJECT { data: Bytes = 0 });

///
/// Event timing codes
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ETCO {
    pub timestamp_format: TimestampFormat,
    pub event_timing_codes: Vec<EventTimingCode>,
}

impl ETCO {
    pub fn read(readable: &mut Readable, version: u8, id: &str) -> Result<ETCO> {
        let _ = version;
        let _ = id;
        let timestamp_format = types::to_timestamp_format(readable.read_u8()?);
        let mut event_timing_codes: Vec<EventTimingCode> = Vec::new();

        loop {
            let mut is_break = true;

            if let Ok(code_type) = readable.read_u8() {
                if let Ok(timestamp) = readable.read_u32() {
                    let event_timing_code = types::to_event_timing_code(code_type, timestamp);
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
            event_timing_codes: event_timing_codes,
        })
    }

    pub fn write(&self, writable: &mut Writable, version: u8) -> Result<()> {
        let _ = version;

        writable.write_u8(types::from_timestamp_format(&self.timestamp_format))?;
        for e in &self.event_timing_codes {
            let (code, timestamp) = types::from_event_timing_code(&e);
            writable.write_u8(code)?;
            writable.write_u32(timestamp)?;
        }

        Ok((()))
    }
}

impl Look for ETCO {
    fn to_map(&self) -> Result<HashMap<&str, String>> {
        let mut map = HashMap::new();

        map.insert("timestamp_format", format!("{:?}", self.timestamp_format));
        map.insert("event_timing_codes", String::new());

        Ok(map)
    }

    fn inside<T>(&self, callback: T) where T: Fn(&str, String) -> bool {
        if callback("timestamp_format", format!("{:?}", self.timestamp_format)) == false {
            return;
        }

        callback("event_timing_codes", String::new());
    }
}

///
/// For all the T??? types
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TEXT {
    pub text_encoding: TextEncoding,
    pub text: String,
}

impl TEXT {
    pub fn read(readable: &mut Readable, version: u8, id: &str) -> Result<TEXT> {
        fn _default(id: &str,
                    decode: ::std::result::Result<String, ::std::borrow::Cow<'static, str>>)
                    -> String {
            match decode {
                Ok(text) => text,
                Err(e) => {
                    debug!("TEXT Error {}, {:?}", id, e);
                    if id == id::TBPM || id == id::TBP {
                        "0".to_string()
                    } else {
                        "".to_string()
                    }
                }
            }
        }

        // trim bom character
        fn trim(text: String) -> String {
            let re =
                regex::Regex::new(r"(^[\x{0}|\x{feff}|\x{fffe}]*|[\x{0}|\x{feff}|\x{fffe}]*$)")
                    .unwrap();
            let text = text.trim();
            re.replace_all(text, "").into_owned()
        }

        let _ = version;
        let text_encoding = types::to_encoding(readable.read_u8()?);
        let data = readable.all_bytes()?;
        let text = match text_encoding {
            TextEncoding::ISO88591 => _default(id, ISO_8859_1.decode(&data, DecoderTrap::Strict)),
            TextEncoding::UTF16LE => _default(id, UTF_16LE.decode(&data, DecoderTrap::Strict)),
            TextEncoding::UTF16BE => _default(id, UTF_16BE.decode(&data, DecoderTrap::Strict)),
            TextEncoding::UTF8 => _default(id, UTF_8.decode(&data, DecoderTrap::Strict)),
        };

        Ok(TEXT {
            text_encoding: text_encoding,
            text: trim(text),
        })
    }

    pub fn write(&self, writable: &mut Writable, version: u8) -> Result<()> {
        let _ = version;

        writable.write_u8(types::from_encoding(&self.text_encoding))?;
        let text = match match self.text_encoding {
            TextEncoding::ISO88591 => ISO_8859_1.encode(self.text.as_str(), EncoderTrap::Strict),
            TextEncoding::UTF16LE => UTF_16LE.encode(self.text.as_str(), EncoderTrap::Strict),
            TextEncoding::UTF16BE => UTF_16BE.encode(self.text.as_str(), EncoderTrap::Strict),
            TextEncoding::UTF8 => UTF_8.encode(self.text.as_str(), EncoderTrap::Strict),
        } {
            Ok(text) => text,
            Err(msg) => return Err(Error::new(ErrorKind::InvalidInput, msg.to_owned().to_string())),
        };

        match self.text_encoding {
            TextEncoding::UTF16LE => {
                writable.write_u8(0xff)?;
                writable.write_u8(0xfe)?;
            }
            _ => (),
        }

        writable.write(&text)?;

        match self.text_encoding {
            TextEncoding::UTF16BE => {
                writable.write_u8(0xfe)?;
                writable.write_u8(0xff)?;
            }
            _ => (),
        }

        Ok(())
    }
}

impl Look for TEXT {
    fn to_map(&self) -> Result<HashMap<&str, String>> {
        let mut map = HashMap::new();

        map.insert("text_encoding", format!("{:?}", self.text_encoding));
        map.insert("text", self.text.clone());

        Ok(map)
    }

    fn inside<T>(&self, callback: T) where T: Fn(&str, String) -> bool {
        if callback("text_encoding", format!("{:?}", self.text_encoding)) == false {
            return;
        }

        callback("text", self.text.clone());
    }
}

pub mod types {
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

    pub fn to_iso8859_1(bytes: &Vec<u8>) -> String {
        use super::encoding::all::ISO_8859_1;
        use super::encoding::{Encoding, DecoderTrap};
        match ISO_8859_1.decode(&bytes, DecoderTrap::Strict) {
            Ok(value) => value.to_string(),
            _ => "".to_string(),
        }
    }

    pub fn from_iso8859_1(v: &String, len: usize) -> Vec<u8> {
        use super::encoding::all::ISO_8859_1;
        use super::encoding::{Encoding, EncoderTrap};
        let mut v = match ISO_8859_1.encode(&v, EncoderTrap::Strict) {
            Ok(value) => value,
            _ => vec![0u8; len],
        };

        for i in v.len()..len {
            v[i] = 0;
        }
        v.to_vec()
    }

    ///
    /// # Frame Encoding
    ///
    /// [See](http://id3.org/id3v2.4.0-structure) > 4. ID3v2 frame overview
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum TextEncoding {
        ISO88591,
        UTF16LE,
        UTF16BE,
        UTF8,
    }

    ///
    /// # Picture Type
    ///
    /// See: PIC, APIC
    ///
    /// [See](http://id3.org/id3v2.3.0#Attached_picture)
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
        PublisherLogoType,
    }

    ///
    /// # Commercial frame
    ///
    /// See: COMR
    ///
    /// [See](http://id3.org/id3v2.3.0#Commercial_frame)
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum ReceivedAs {
        Other,
        StandardCDAlbum,
        CompressedAudioOnCD,
        FileOverInternet,
        StreamOverInternet,
        AsNoteSheets,
        AsNoteSheetsInBook,
        MusicOnMedia,
        NonMusicalMerchandise,
    }

    ///
    /// # Interpolation method
    ///
    /// See: EQU2
    ///
    /// [See](http://id3.org/id3v2.4.0-frames) > 4.12. Equalisation (2)
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum InterpolationMethod {
        Band,
        Linear,
    }

    ///
    /// # Content Type
    ///
    /// See: SYLT
    ///
    /// [See](http://id3.org/id3v2.4.0-frames) > 4.9. Synchronised lyrics/text
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum ContentType {
        Other,
        Lyrics,
        TextTranscription,
        MovementName,
        Events,
        Chord,
        Trivia,
        UrlsToWebpages,
        UrlsToImages,
    }

    ///
    /// # Timestamp format
    ///
    /// See: ETCO, POSS, SYLT, SYTC
    ///
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum TimestampFormat {
        MpecFrames,
        Milliseconds,
    }

    ///
    /// # Event Timing Code
    ///
    /// See: ETCO
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
        OneMoreByteOfEventsFollows(u32),
    }

    ///
    /// # Frame header flag
    ///
    /// [See](http://id3.org/id3v2.3.0#Frame_header_flags)
    ///
    /// ## V2.4 only flags
    /// - Unsynchronisation
    /// - DataLength
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
        DataLength,
    }

    ///
    /// # Head flags
    ///
    /// - [See](http://id3.org/id3v2.3.0#ID3v2_header)
    /// - [See](http://id3.org/id3v2.4.0-structure) > 3.1. ID3v2 header
    ///
    /// ## V2.4 only flag
    /// - FooterPresent
    ///
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum HeadFlag {
        Unsynchronisation,
        Compression,
        ExtendedHeader,
        ExperimentalIndicator,
        FooterPresent,
    }

}

macro_rules! define_id_str {
    (
        $( $id:ident ),*
    ) => (

        pub mod id {
            $( pub const $id: &'static str = stringify!($id); )*
        }

    );

    (
        $( $id:ident ),+,
    ) => (
        define_id_str!( $($id),+ );
    )
}

macro_rules! define_id_to_framebody {
    (
        $( $id:ident = $body:ident : $frame:ident ),*
    ) => (

        pub fn read_framebody_with_id(id: &str, version: u8, mut readable: Cursor<Vec<u8>>) 
            -> Result<FrameBody> {

                trace!("id:{}, version:{}", id, version);

                let frame_body = match id.as_ref() {

                    $( stringify!($id) => FrameBody::$body($frame::read(&mut readable, version, id)?) ),*
                    
                    , 
                    _ => {
                        warn!("No frame id found!! '{}'", id);
                        FrameBody::TEXT(TEXT::read(&mut readable, version, id)?)
                    }
                };

                trace!("{:?}", frame_body);
                
                Ok(frame_body)
        }

        pub fn framebody_to_id(frame_body: &FrameBody, version: u8) -> &'static str {

            $(
                if let &FrameBody::$body(_) = frame_body {
                    let id_len = stringify!($id).len();
                    let body_len = stringify!($body).len();
                    let frame_len = stringify!($frame).len();

                    if id_len != body_len || body_len != frame_len {

                        if version == 2 {
                            return id::$id;
                        } else {
                            return id::$body
                        }
                    } else {
                        return id::$id;
                    }
                }
            );*

            "UNKNOWN"
        }

        pub fn frame2_to_frame4(id: &str) -> String  {

            let mut m = HashMap::new();

            $(
                let meta_id = stringify!($id);

                if meta_id.len() == 3 {
                    m.insert(id, stringify!($frame));
                }

            );*

            match m.get(id) {
                Some(v4) => v4.to_string(),
                _ => "UNKNOWN".to_string()
            }
        }
    );

    (
        $( $id:ident = $body:ident : $frame:ident ),+,
    ) => (
        define_id_to_framebody!( $($id = $body : $frame),+ );
    )
}

macro_rules! define_framebody {
    (
        $( $id:ident = $frame:ident ),*
    ) => (

        ///
        /// Frame types
        ///
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        pub enum FrameBody {

            $( $id($frame) ),*
            ,
            /// It is used for unknown frame when to parse
            SKIP(String, Vec<u8>),

            /// Invalid frame
            INVALID(String)
        }

        impl Look for FrameBody {
            fn to_map(&self) -> Result<HashMap<&str, String>> {
                match self {
                    $(
                        &FrameBody::$id(ref frame) => frame.to_map()
                    ),*
                    ,
                    _ => Ok(HashMap::new())
                }
            }

            fn inside<T>(&self, callback: T) where T: Fn(&str, String) -> bool {
                match self {
                    $(
                        &FrameBody::$id(ref frame) => frame.inside(callback)
                    ),*
                    ,
                    _ => {}
                }
            }
        }

        pub fn framebody_as_bytes(frame_body: &FrameBody, version: u8) -> Result<(&str, Vec<u8>)> {

            let mut writable = Cursor::new(vec![0u8; 0]);

            match frame_body {

                $( &FrameBody::$id(ref frame) => { frame.write(&mut writable, version)?; } ),*
                ,
                _ => ()
            };

            let id = framebody_to_id(frame_body, version);

            let mut buf = Vec::new();
            writable.copy(&mut buf)?;

            Ok((id, buf))
        }
    );

    (
        $( $id:ident = $frame:ident ),+,
    ) => (
        define_framebody!( $($id = $frame),+ );
    )
}

define_id_str!(
    //
    // 2.2
    //
    BUF, CNT, COM, CRA, CRM, ETC, EQU, GEO, IPL, LNK, MCI, MLL, PIC, POP, REV,//
    RVA, SLT, STC, TAL, TBP, TCM, TCO, TCR, TDA, TDY, TEN, TFT, TIM, TKE, TLA,//
    TLE, TMT, TOA, TOF, TOL, TOR, TOT, TP1, TP2, TP3, TP4, TPA, TPB, TRC, TRD,//
    TRK, TSI, TSS, TT1, TT2, TT3, TXT, TXX, TYE, UFI, ULT, WAF, WAR, WAS,//
    WCM, WCP, WPB, WXX,
    //
    // 2.3 & 2.4
    //
    AENC, APIC, ASPI, COMM, COMR, ENCR, EQU2,//
    //
    // 2.3 only
    //
    EQUA, IPLS, RVAD, TDAT, TIME, TSIZ, TYER,//
    //
    ETCO, GEOB, GRID, LINK, MCDI, MLLT, OWNE, PRIV, PCNT, POPM, POSS, RBUF, RVA2,//
    RVRB, SEEK, SIGN, SYLT, SYTC, TALB, TBPM, TCOM, TCON, TCOP, TDEN, TDLY, TDOR,//
    TDRC, TDTG, TDRL, TENC, TEXT, TFLT, TIPL, TIT1, TIT2, TIT3, TKEY, TLAN, TLEN,//
    TMCL, TMED, TMOO, TOAL, TOFN, TOLY, TOPE, TORY, TOWN, TPE1, TPE2, TPE3, TPE4,//
    TPOS, TPRO, TPUB, TRCK, TRDA, TRSN, TRSO, TSOA, TSOP, TSOT, TSRC, TSSE, TSST,//
    TXXX, UFID, USER, USLT, WCOM, WCOP, WOAF, WOAR, WOAS, WORS, WPAY, WPUB, WXXX,//
);

define_id_to_framebody!(
     BUF = BUF : RBUF, CRM = CRM : CRM, PIC = PIC : PIC, //
     //
     CNT = PCNT : PCNT, COM = COMM : COMM, CRA = AENC : AENC,//
     ETC = ETCO : ETCO, EQU = EQUA : EQUA, GEO = GEOB : GEOB,//
     IPL = IPLS : IPLS, LNK = LINK : LINK, MCI = MCDI : MCDI,//
     MLL = MLLT : MLLT, POP = POPM : POPM, REV = RVRB : RVRB,//
     RVA = RVAD : RVA2, SLT = SYLT : SYLT, STC = SYTC : SYTC,//
     //
     TAL = TALB : TEXT, TBP = TBPM : TEXT, TCM = TCOM : TEXT,//
     TCO = TCON : TEXT, TCR = TCOP : TEXT, TDA = TDAT : TEXT,//
     TDY = TDLY : TEXT, TEN = TENC : TEXT, TFT = TFLT : TEXT,//
     TIM = TIME : TEXT, TKE = TKEY : TEXT, TLA = TLAN : TEXT,//
     TLE = TLEN : TEXT, TMT = TMED : TEXT, TOA = TMED : TEXT,//
     TOF = TOFN : TEXT, TOL = TOLY : TEXT, TOR = TORY : TEXT,//
     TOT = TOAL : TEXT, TP1 = TPE1 : TEXT, TP2 = TPE2 : TEXT,//
     TP3 = TPE3 : TEXT, TP4 = TPE4 : TEXT, TPA = TPOS : TEXT,//
     TPB = TPUB : TEXT, TRC = TSRC : TEXT, TRD = TRDA : TEXT,//
     TRK = TRCK : TEXT, TSI = TSIZ : TEXT, TSS = TSSE : TEXT,//
     TT1 = TIT1 : TEXT, TT2 = TIT2 : TEXT, TT3 = TIT3 : TEXT,//
     TXT = TEXT : TEXT, TYE = TYER : TEXT, //
     //
     TXX = TXXX : TXXX, UFI = UFID : UFID, ULT = USLT : USLT,//
     //
     WAF = WOAF : LINK, WAR = WOAR : LINK, WAS = WOAS : LINK,//
     WCM = WCOM : LINK, WCP = WCOP : LINK, WPB = WPUB : LINK,//
     //
     WXX = WXXX : WXXX,//
     //
     AENC = AENC : AENC, APIC = APIC : APIC, ASPI = ASPI : ASPI,//
     COMM = COMM : COMM, COMR = COMR : COMR, ENCR = ENCR : ENCR,//
     EQUA = EQUA : EQUA, EQU2 = EQU2 : EQU2, ETCO = ETCO : ETCO,//
     GEOB = GEOB : GEOB, GRID = GRID : GRID, IPLS = IPLS : IPLS,//
     LINK = LINK : LINK, MCDI = MCDI : MCDI, MLLT = MLLT : MLLT,//
     OWNE = OWNE : OWNE, PRIV = PRIV : PRIV, PCNT = PCNT : PCNT,//
     POPM = POPM : POPM, POSS = POSS : POSS, RBUF = RBUF : RBUF,//
     RVAD = RVAD : RVA2, RVA2 = RVA2 : RVA2, RVRB = RVRB : RVRB,//
     SEEK = SEEK : SEEK, SIGN = SIGN : SIGN, SYLT = SYLT : SYLT,//
     SYTC = SYTC : SYTC, UFID = UFID : UFID, USER = USER : USER,//
     USLT = USLT : USLT,//
     //
     TALB = TALB : TEXT, TBPM = TBPM : TEXT, TCOM = TCOM : TEXT,//
     TCON = TCON : TEXT, TCOP = TCOP : TEXT, TDAT = TDAT : TEXT,//
     TDEN = TDEN : TEXT, TDLY = TDLY : TEXT, TDOR = TDOR : TEXT,//
     TDRC = TDRC : TEXT, TDRL = TDRL : TEXT, TDTG = TDTG : TEXT,//
     TENC = TENC : TEXT, TEXT = TEXT : TEXT, TIME = TIME : TEXT,//
     TFLT = TFLT : TEXT, TIPL = TIPL : TEXT, TIT1 = TIT1 : TEXT,//
     TIT2 = TIT2 : TEXT, TIT3 = TIT3 : TEXT, TKEY = TKEY : TEXT,//
     TLAN = TLAN : TEXT, TLEN = TLEN : TEXT, TMCL = TMCL : TEXT,//
     TMED = TMED : TEXT, TMOO = TMOO : TEXT, TOAL = TOAL : TEXT,//
     TOFN = TOFN : TEXT, TOLY = TOLY : TEXT, TOPE = TOPE : TEXT,//
     TORY = TORY : TEXT, TOWN = TOWN : TEXT, TPE1 = TPE1 : TEXT,//
     TPE2 = TPE2 : TEXT, TPE3 = TPE3 : TEXT, TPE4 = TPE4 : TEXT,//
     TPOS = TPOS : TEXT, TPRO = TPRO : TEXT, TPUB = TPUB : TEXT,//
     TRCK = TRCK : TEXT, TRDA = TRDA : TEXT, TRSN = TRSN : TEXT,//
     TSIZ = TSIZ : TEXT, TRSO = TRSO : TEXT, TSOA = TSOA : TEXT,//
     TSOP = TSOP : TEXT, TSOT = TSOT : TEXT, TSRC = TSRC : TEXT,//
     TSSE = TSSE : TEXT, TYER = TYER : TEXT, TSST = TSST : TEXT,//
     //
     TXXX = TXXX : TXXX,//
     //
     WCOM = WCOM : LINK, WCOP = WCOP : LINK, WOAF = WOAF : LINK,//
     WOAR = WOAR : LINK, WOAS = WOAS : LINK, WORS = WORS : LINK,//
     WPAY = WPAY : LINK, WPUB = WPUB : LINK,//
     //
     WXXX = WXXX : WXXX,//
);

define_framebody!(
    BUF  = RBUF, // Recommended buffer size
    CRM  = CRM, // 2.2 only. Encrypted meta frame
    PIC  = PIC, // 2.2 only. Attached picture
    AENC = AENC, // Audio encryption
    APIC = APIC, // Attached picture
    ASPI = ASPI, // Audio seek point index
    COMM = COMM, // Comments
    COMR = COMR, // Commercial frame
    ENCR = ENCR, // Encryption method registration
    EQUA = EQUA, // 2.3 only // Equalisation
    EQU2 = EQU2, // Equalisation (2)
    ETCO = ETCO, // Event timing codes
    GEOB = GEOB, // General encapsulated object
    GRID = GRID, // Group identification registration
    IPLS = IPLS, // 2.3 only. Involved people list
    LINK = LINK, // Linked information
    MCDI = MCDI, // Music CD identifier
    MLLT = MLLT, // MPEG location lookup table
    OWNE = OWNE, // Ownership frame
    PRIV = PRIV, // Private frame
    PCNT = PCNT, // Play counter
    POPM = POPM, // Popularimeter
    POSS = POSS, // Position synchronisation frame
    RBUF = RBUF, // Recommended buffer size
    RVAD = RVA2, // 2.3 only. Relative volume adjustment
    RVA2 = RVA2, // Relative volume adjustment (2)
    RVRB = RVRB, // Reverb
    SEEK = SEEK, // Seek frame
    SIGN = SIGN, // Signature frame
    SYLT = SYLT, // Synchronised lyric/text
    SYTC = SYTC, // Synchronised tempo codes
    TALB = TEXT, // Album/Movie/Show title
    TBPM = TEXT, // BPM (beats per minute)
    TCOM = TEXT, // Composer
    TCON = TEXT, // Content type
    TCOP = TEXT, // Copyright message
    TDAT = TEXT, // 2.3 only. Date
    TDEN = TEXT, // Encoding time
    TDLY = TEXT, // Playlist delay
    TDOR = TEXT, // Original release time
    TDRC = TEXT, // Recording time
    TDRL = TEXT, // Release time
    TDTG = TEXT, // Tagging time
    TENC = TEXT, // Encoded by
    TEXT = TEXT, // Lyricist/Text writer
    TFLT = TEXT, // File type
    TIME = TEXT, // 2.3 only. Time
    TIPL = TEXT, // Involved people list
    TIT1 = TEXT, // Content group description
    TIT2 = TEXT, // Title/songname/content description
    TIT3 = TEXT, // Subtitle/Description refinement
    TKEY = TEXT, // Initial key
    TLAN = TEXT, // Language(s)
    TLEN = TEXT, // Length
    TMCL = TEXT, // Musician credits list
    TMED = TEXT, // Media type
    TMOO = TEXT, // Mood
    TOAL = TEXT, // Original album/movie/show title
    TOFN = TEXT, // Original filename
    TOLY = TEXT, // Original lyricist(s)/text writer(s)
    TOPE = TEXT, // Original artist(s)/performer(s)
    TORY = TEXT, // 2.3 only. Original release year
    TOWN = TEXT, // File owner/licensee
    TPE1 = TEXT, // Lead performer(s)/Soloist(s)
    TPE2 = TEXT, // Band/orchestra/accompaniment
    TPE3 = TEXT, // Conductor/performer refinement
    TPE4 = TEXT, // Interpreted, remixed, or otherwise modified by
    TPOS = TEXT, // Part of a set
    TPRO = TEXT, // Produced notice
    TPUB = TEXT, // Publisher
    TRCK = TEXT, // Track number/Position in set
    TRDA = TEXT, // 2.3 only. Recording dates
    TRSN = TEXT, // Internet radio station name
    TRSO = TEXT, // Internet radio station owner
    TSIZ = TEXT, // 2.3 only. Size
    TSOA = TEXT, // Album sort order
    TSOP = TEXT, // Performer sort order
    TSOT = TEXT, // Title sort order
    TSRC = TEXT, // ISRC (international standard recording code)
    TSSE = TEXT, // Software/Hardware and settings used for encoding
    TYER = TEXT, // 2.3 only. Year
    TSST = TEXT, // Software/Hardware and settings used for encoding
    TXXX = TXXX, // User defined text information frame
    UFID = UFID, // Unique file identifier
    USER = USER, // Terms of use
    USLT = USLT, // Unsychronized lyric/text transcription
    WCOM = LINK, // Commercial information
    WCOP = LINK, // Copyright/Legal information
    WOAF = LINK, // Official audio file webpage
    WOAR = LINK, // Official artist/performer webpage
    WOAS = LINK, // Official audio source webpage
    WORS = LINK, // Official internet radio station homepage
    WPAY = LINK, // Payment
    WPUB = LINK, // Publishers official webpage
    WXXX = WXXX, // User defined URL link frame
    OBJECT = OBJECT, // It only use to write a encrypted bytes directly
);