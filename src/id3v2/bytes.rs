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