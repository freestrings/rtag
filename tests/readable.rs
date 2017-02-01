extern crate rust_id3 as id3;

use std::vec;
use id3::readable::factory;

#[test]
fn readable_bytes() {
    let valid = "0123456789".to_string();
    if let Ok(mut readable) = factory::from_bytes(valid.into_bytes()) {
        assert!(readable.bytes(10).is_ok());
        assert!(readable.bytes(10).is_err());
    } else {
        assert!(false);
    }

    let str = "AB가나01".to_string();
    if let Ok(mut readable) = factory::from_bytes(str.into_bytes()) {
        assert!(readable.skip(1).is_ok());
        assert_eq!(readable.string(1).unwrap(), "B");
        // utf8, 3bytes
        assert_eq!(readable.string(3).unwrap(), "가");
        assert_eq!(readable.string(5).unwrap(), "나01");
        assert!(readable.bytes(1).is_err());
    } else {
        assert!(false);
    }
}

#[test]
fn readable_file() {
    if let Ok(mut readable) = factory::from_path("./test-resources/file1.txt") {
        assert!(readable.bytes(10).is_ok());
        assert!(readable.bytes(10).is_ok());
        assert!(readable.skip(-5).is_ok());
        assert_eq!(readable.string(5).unwrap(), "fghij");
        assert!(readable.bytes(10).is_err());
    } else {
        assert!(false);
    }
}

#[test]
fn readable_utf16_string() {
    let str = "AB가나01".to_string();
    let mut bytes: vec::Vec<u8> = str.into_bytes();
    bytes.push(0x00);
    bytes.push(0x01);
    bytes.push(0x00);
    bytes.push(0x00);
    bytes.push(0x02);
    assert_eq!(bytes.len(), 15);
    let mut readable = factory::from_bytes(bytes).unwrap();
    let read = readable.utf16_string().unwrap();
    assert_eq!("AB\u{ac00}\u{b098}01\u{0}\u{1}", read);
    assert!(readable.skip(1).is_ok());
    assert!(readable.bytes(1).is_err());
}