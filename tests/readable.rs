extern crate rust_id3 as id3;

use std::vec;
use id3::readable::factory;

#[test]
fn bytes1() {
    let valid = "0123456789";
    if let Ok(mut readable) = factory::from_str(valid) {
        assert!(readable.as_bytes(10).is_ok());
        assert!(readable.as_bytes(10).is_err());
    } else {
        assert!(false);
    }
}

#[test]
fn byte2() {
    let str = "AB가나01".to_string();
    if let Ok(mut readable) = factory::from_byte(str.into_bytes()) {
        assert!(readable.skip(1).is_ok());
        assert_eq!(readable.as_string(1).unwrap(), "B");
        // utf8, 3bytes
        assert_eq!(readable.as_string(3).unwrap(), "가");
        assert_eq!(readable.as_string(5).unwrap(), "나01");
        assert!(readable.as_bytes(1).is_err());
    } else {
        assert!(false);
    }
}

#[test]
fn file1() {
    if let Ok(mut readable) = factory::from_path("./test-resources/file1.txt") {
        assert!(readable.as_bytes(10).is_ok());
        assert!(readable.as_bytes(10).is_ok());
        assert!(readable.skip(-5).is_ok());
        assert_eq!(readable.as_string(10).unwrap(), "fghij");
        assert!(readable.as_bytes(10).is_err());
    } else {
        assert!(false);
    }
}

#[test]
fn file2() {
    if let Ok(mut readable) = factory::from_path("./test-resources/file1.txt") {
        assert!(readable.skip(10).is_ok());
        assert!(readable.as_bytes(10).is_ok());
        assert!(readable.skip(-5).is_ok());
        assert_eq!(readable.as_string(10).unwrap(), "fghij");
        assert!(readable.as_bytes(10).is_err());
    } else {
        assert!(false);
    }
}


#[test]
fn utf16_string() {
    let str = "AB가나01".to_string();
    let mut bytes: vec::Vec<u8> = str.into_bytes();
    bytes.push(0x00);
    bytes.push(0x01);
    bytes.push(0x00);
    bytes.push(0x00);
    bytes.push(0x02);
    assert_eq!(bytes.len(), 15);
    let mut readable = factory::from_byte(bytes).unwrap();
    let (size, read) = readable.utf16_string().unwrap();
    assert_eq!(size, 14);
    assert_eq!("AB\u{ac00}\u{b098}01\u{0}\u{1}", read);
    assert!(readable.skip(1).is_ok());
    assert!(readable.as_bytes(1).is_err());
}