extern crate rtag;

use std::fs::File;
use std::io::Cursor;
use std::vec::Vec;

use rtag::rw::*;

#[test]
fn readable_bytes() {
    let valid = "0123456789".to_string();
    let mut readable = Cursor::new(valid.into_bytes());
    assert!(readable.read_bytes(10).is_ok());
    assert!(readable.read_bytes(10).is_err());

    let str = "AB가나01".to_string();
    let mut readable = Cursor::new(str.into_bytes());
    assert!(readable.skip_bytes(1).is_ok());
    assert_eq!(readable.read_string(1).unwrap(), "B");
    // utf8, 3bytes
    assert_eq!(readable.read_string(3).unwrap(), "가");
    assert_eq!(readable.read_string(5).unwrap(), "나01");
    assert!(readable.read_bytes(1).is_err());
}

#[test]
fn readable_file() {
    let mut readable = File::open("./test-resources/file1.txt").unwrap();
    assert!(readable.read_bytes(10).is_ok());
    assert!(readable.read_bytes(10).is_ok());
    assert!(readable.skip_bytes(-5).is_ok());
    assert_eq!(readable.read_string(5).unwrap(), "fghij");
    assert!(readable.read_bytes(10).is_err());
}

#[test]
fn readable_utf16_string() {
    let str = "AB가나01".to_string();
    let mut bytes: Vec<u8> = str.into_bytes();
    bytes.push(0x00);
    bytes.push(0x01);
    bytes.push(0x00);
    bytes.push(0x00);
    bytes.push(0x02);
    assert_eq!(bytes.len(), 15);
    let mut readable = Cursor::new(bytes);
    let read = readable.read_utf16_string().unwrap();
    assert_eq!("AB\u{ac00}\u{b098}01\u{0}\u{1}", read);
    assert!(readable.skip_bytes(1).is_ok());
    assert!(readable.read_bytes(1).is_err());
}