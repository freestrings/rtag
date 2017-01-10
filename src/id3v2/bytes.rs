pub fn to_u32(bytes: &[u8]) -> u32 {
    let mut v: u32 = (bytes[3] & 0xff) as u32;
    v = v | ((bytes[2] & 0xff) as u32) << 8;
    v = v | ((bytes[1] & 0xff) as u32) << 16;
    v = v | ((bytes[0] & 0xff) as u32) << 24;
    v
}

// Sizes are 4bytes long big-endian but first bit is 0
pub fn to_synchsafe(bytes: &[u8]) -> u32 {
    let mut v: u32 = (bytes[3] & 0x7f) as u32;
    v = v | ((bytes[2] & 0x7f) as u32) << 7;
    v = v | ((bytes[1] & 0x7f) as u32) << 14;
    v = v | ((bytes[0] & 0x7f) as u32) << 21;
    v
}