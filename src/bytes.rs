pub const BIT7: u8 = 0x80;
pub const BIT6: u8 = 0x40;
pub const BIT5: u8 = 0x20;
pub const BIT4: u8 = 0x10;
pub const BIT3: u8 = 0x08;
pub const BIT2: u8 = 0x04;
pub const BIT1: u8 = 0x02;
pub const BIT0: u8 = 0x01;

pub fn to_encoding(encoding: u8) -> ::frame::constants::TextEncoding {
    match encoding {
        0 => ::frame::constants::TextEncoding::ISO8859_1,
        1 => ::frame::constants::TextEncoding::UTF16LE,
        2 => ::frame::constants::TextEncoding::UTF16BE,
        3 => ::frame::constants::TextEncoding::UTF8,
        _ => ::frame::constants::TextEncoding::ISO8859_1
    }
}

pub fn to_u16(bytes: &[u8]) -> u16 {
    let mut v: u16 = (bytes[1] & 0xff) as u16;
    v = v | ((bytes[0] & 0xff) as u16) << 8;

    v
}

pub fn to_u32(bytes: &[u8]) -> u32 {
    if bytes.len() == 3 {
        let mut v: u32 = (bytes[2] & 0xff) as u32;
        v = v | ((bytes[1] & 0xff) as u32) << 8;
        v = v | ((bytes[0] & 0xff) as u32) << 16;

        v
    } else {
        let mut v: u32 = (bytes[3] & 0xff) as u32;
        v = v | ((bytes[2] & 0xff) as u32) << 8;
        v = v | ((bytes[1] & 0xff) as u32) << 16;
        v = v | ((bytes[0] & 0xff) as u32) << 24;

        v
    }
}

// Sizes are 4bytes long big-endian but first bit is 0
// @see http://id3.org/id3v2.4.0-structure > 6.2. Synchsafe integers
pub fn to_synchsafe(bytes: &[u8]) -> u32 {
    let mut v: u32 = (bytes[3] & 0x7f) as u32;
    v = v | ((bytes[2] & 0x7f) as u32) << 7;
    v = v | ((bytes[1] & 0x7f) as u32) << 14;
    v = v | ((bytes[0] & 0x7f) as u32) << 21;

    v
}