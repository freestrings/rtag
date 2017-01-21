#[macro_use] extern crate log;

pub extern crate regex;
extern crate env_logger;
extern crate rust_id3 as id3;

use std::vec::Vec;
use id3::frame::constants::{EventTimingCode, FrameData, TimestampFormat};
use id3::metadata::{header, frames, MetadataIterator, Unit};
use id3::readable;

fn comp_frame(frame_data: FrameData, data: &mut Vec<&str>) {
    data.reverse();
    match frame_data {
        FrameData::COMM(frame) => assert_eq!(data.pop().unwrap(), format!("{}:{}:{}",
                                                                          frame.language,
                                                                          frame.short_description,
                                                                          frame.actual_text)),
        FrameData::PIC(frame) => assert_eq!(data.pop().unwrap(), format!("{}:{:?}:{}:{}",
                                                                         frame.image_format,
                                                                         frame.picture_type,
                                                                         frame.description,
                                                                         frame.picture_data.len())),
        FrameData::APIC(frame) => assert_eq!(data.pop().unwrap(), format!("{}:{:?}:{}:{}",
                                                                          frame.mime_type,
                                                                          frame.picture_type,
                                                                          frame.description,
                                                                          frame.picture_data.len())),
        FrameData::TALB(frame) |
        FrameData::TBPM(frame) |
        FrameData::TCOM(frame) |
        FrameData::TCON(frame) |
        FrameData::TCOP(frame) |
        FrameData::TDEN(frame) |
        FrameData::TDLY(frame) |
        FrameData::TDOR(frame) |
        FrameData::TDRC(frame) |
        FrameData::TDRL(frame) |
        FrameData::TDTG(frame) |
        FrameData::TENC(frame) |
        FrameData::TEXT(frame) |
        FrameData::TFLT(frame) |
        FrameData::TIPL(frame) |
        FrameData::TIT1(frame) |
        FrameData::TIT2(frame) |
        FrameData::TIT3(frame) |
        FrameData::TKEY(frame) |
        FrameData::TLAN(frame) |
        FrameData::TLEN(frame) |
        FrameData::TMCL(frame) |
        FrameData::TMED(frame) |
        FrameData::TMOO(frame) |
        FrameData::TOAL(frame) |
        FrameData::TOFN(frame) |
        FrameData::TOLY(frame) |
        FrameData::TOPE(frame) |
        FrameData::TOWN(frame) |
        FrameData::TPE1(frame) |
        FrameData::TPE2(frame) |
        FrameData::TPE3(frame) |
        FrameData::TPE4(frame) |
        FrameData::TPOS(frame) |
        FrameData::TPRO(frame) |
        FrameData::TPUB(frame) |
        FrameData::TRCK(frame) |
        FrameData::TRSN(frame) |
        FrameData::TRSO(frame) |
        FrameData::TSOA(frame) |
        FrameData::TSOP(frame) |
        FrameData::TSOT(frame) |
        FrameData::TSRC(frame) |
        FrameData::TSSE(frame) |
        FrameData::TSST(frame) => assert_eq!(data.pop().unwrap(), frame.text),
        FrameData::TXXX(frame) => assert_eq!(data.pop().unwrap(), format!("{}:{}",
                                                                          frame.description,
                                                                          frame.value)),
        _ => ()
    }
    data.reverse();
}

#[test]
fn regex() {
    let frame_id = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}$").unwrap();
    assert!(frame_id.is_match("AAA0"));
    assert!(frame_id.is_match("AAA"));
    assert!(!frame_id.is_match("0AA"));
    assert!(!frame_id.is_match("AA"));
    assert!(frame_id.is_match("COM"));
}

#[test]
fn iterator() {
    let _ = env_logger::init();

    match MetadataIterator::new("./test-resources/v1-v2.mp3") {
        Ok(metadata) => for m in metadata {
            match m {
                Unit::Header(bytes) => assert_eq! (10, bytes.len()),
                Unit::ExtendedHeader(bytes) => assert_eq! (0, bytes.len()),
                Unit::FrameV1(bytes) => assert_eq! (128, bytes.len()),
                Unit::FrameV2(_, head, _) => assert_eq! (6, head.len()),
            }
        },
        _ => ()
    }
}

#[test]
fn empty() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/empty-meta.mp3").unwrap() {
        match m {
            Unit::FrameV1(_) => assert!(false),
            Unit::FrameV2(_, _, _) => assert!(false),
            _ => ()
        }
    }
}

#[test]
fn v1() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/v1-v2.mp3").unwrap() {
        match m {
            Unit::FrameV1(bytes) => {
                let v1 = frames::V1::new(bytes);
                let frame = v1.read().unwrap();
                debug!("v1: {:?}", frame);
                assert_eq!("Artist", frame.artist);
                assert_eq!("!@#$", frame.comment);
                assert_eq!("1", frame.track);
                assert_eq!("137", frame.genre);
            },
            _ => ()
        }
    }

    let id3v1_tag = concat!("TAGTITLETITLETITLETITLETITLETITLE",
                "ARTISTARTISTARTISTARTISTARTIST",
                "ALBUMALBUMALBUMALBUMALBUMALBUM",
                "2017",
                "COMMENTCOMMENTCOMMENTCOMMENTCO4");

    let mut readable = readable::factory::from_str(id3v1_tag).unwrap();
    let v1 = frames::V1::new(readable.all_bytes().unwrap());
    let frame = v1.read().unwrap();
    assert_eq!(frame.title, "TITLETITLETITLETITLETITLETITLE");
    assert_eq!(frame.artist, "ARTISTARTISTARTISTARTISTARTIST");
    assert_eq!(frame.album, "ALBUMALBUMALBUMALBUMALBUMALBUM");
    assert_eq!(frame.comment, "COMMENTCOMMENTCOMMENTCOMMENTCO");
    assert_eq!(frame.year, "2017");

    let id3v1_tag = concat!("TAGTITLE                         ",
                "ARTIST                        ",
                "ALBUM                         ",
                "2017",
                "COMMENT                        ");

    let mut readable = readable::factory::from_str(id3v1_tag).unwrap();
    let v1 = frames::V1::new(readable.all_bytes().unwrap());
    let frame = v1.read().unwrap();
    assert_eq!(frame.title, "TITLE");
    assert_eq!(frame.artist, "ARTIST");
    assert_eq!(frame.album, "ALBUM");
    assert_eq!(frame.comment, "COMMENT");
    assert_eq!(frame.year, "2017");
}

#[test]
fn v1_no_id() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230-no-id3.mp3").unwrap() {
        match m {
            Unit::FrameV1(_) => assert!(false),
            _ => ()
        }
    }
}

#[test]
fn header() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230.mp3").unwrap() {
        match m {
            Unit::Header(bytes) => {
                let head = header::Head::new(bytes);
                let header = head.read().unwrap();
                assert_eq!(3, header.version);
                assert_eq!(0, header.minor_version);
                assert_eq!(header.has_flag(header::Flag::Unsynchronisation), false);
                assert_eq!(header.has_flag(header::Flag::ExtendedHeader), false);
                assert_eq!(header.has_flag(header::Flag::ExperimentalIndicator), false);
                assert_eq!(header.size, 1171);
            },
            _ => ()
        }
    }

    for m in MetadataIterator::new("./test-resources/240.mp3").unwrap() {
        match m {
            Unit::Header(bytes) => {
                let head = header::Head::new(bytes);
                let header = head.read().unwrap();
                assert_eq!(4, header.version);
                assert_eq!(0, header.minor_version);
                assert_eq!(header.has_flag(header::Flag::Unsynchronisation), false);
                assert_eq!(header.has_flag(header::Flag::ExtendedHeader), false);
                assert_eq!(header.has_flag(header::Flag::ExperimentalIndicator), false);
                assert_eq!(header.size, 165126);
            },
            _ => ()
        }
    }
}

#[test]
fn frame_id() {
    let _ = env_logger::init();

    fn comp_id(frame: frames::V2, data: &mut Vec<&str>) {
        data.reverse();
        assert_eq!(frame.id, data.pop().unwrap());
        data.reverse();
    }

    fn test(path: &str, mut data: Vec<&str>) {
        for m in MetadataIterator::new(path).unwrap() {
            match m {
                Unit::FrameV2(head, body, version) => comp_id(frames::V2::new(head, body, version), &mut data),
                _ => ()
            }
        }
    }

    test("./test-resources/v1-v2.mp3",
         vec!["TIT2", "TPE1", "TALB", "TPE2", "TCON", "COMM", "TRCK", "TPOS"]);

    test("./test-resources/230.mp3",
         vec!["TALB", "TCON", "TIT2", "TLEN", "TPE1", "TRCK", "COMM", "TYER"]);

    test("./test-resources/240.mp3",
         vec!["TDRC", "TRCK", "TPOS", "TPE1", "TALB", "TPE2", "TIT2", "TSRC", "TCON", "COMM"]);

    test("./test-resources/v1-v2-albumimage.mp3",
         vec!["TENC", "WXXX", "TCOP", "TOPE", "TCOM", "COMM", "TPE1", "TALB", "COMM", "TRCK", "TDRC", "TCON", "TIT2", "APIC", "WCOM", "WCOP", "WOAR", "WOAF", "WOAS", "WORS", "WPAY", "WPUB"]);
}

#[test]
fn frame_data() {
    let _ = env_logger::init();

    fn test(path: &str, mut data: Vec<&str>) {
        for m in MetadataIterator::new(path).unwrap() {
            match m {
                Unit::FrameV2(head, body, version) => {
                    let v2 = frames::V2::new(head, body, version);
                    let frame = v2.read().unwrap();
                    debug!("v2: {:?}", frame);
                    comp_frame(frame, &mut data);
                },
                _ => ()
            }
        }
    }

    test("./test-resources/v1-v2.mp3",
         vec!["타이틀", "Artist", "アルバム", "Album Artist", "Heavy Metal", "eng::!@#$", "1", "0"]);

    test("./test-resources/230.mp3",
         vec!["앨범", "Rock", "Tㅏi틀", "0", "아티st", "1", "eng::!!!@@#$@$^#$%^\\n123", "2017"]);

    test("./test-resources/240.mp3",
         vec!["2017", "1", "1", "아티스트", "Album", "Artist/아티스트", "타이틀", "ABAB", "Alternative", "eng::~~"]);

    test("./test-resources/v2.2.mp3",
         vec!["Test v2.2.0", "Pudge", "2", "(37)", "eng::All Rights Reserved", "1998"]);
}

#[test]
fn frame_etco() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230-etco.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, body, version) => {
                let v2 = frames::V2::new(head, body, version);
                let frame = v2.read().unwrap();
                match frame {
                    FrameData::ETCO(frame) => {
                        assert_eq!(&TimestampFormat::Milliseconds, &frame.timestamp_format);

                        match frame.event_timing_codes[0] {
                            EventTimingCode::MainPartStart(timestamp) => assert_eq!(timestamp, 152110),
                            _ => assert!(false)
                        }
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }
}

#[test]
fn frame_pcnt() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/240-pcnt.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, body, version) => {
                let v2 = frames::V2::new(head, body, version);
                let frame = v2.read().unwrap();
                match frame {
                    FrameData::PCNT(frame) => assert_eq!(256, frame.counter),
                    _ => ()
                }
            },
            _ => ()
        }
    }
}

#[test]
fn frame_tbpm() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230-tbpm.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, body, version) => {
                let v2 = frames::V2::new(head, body, version);
                let frame = v2.read().unwrap();
                match frame {
                    FrameData::TBPM(frame) => {
                        assert_eq!("0", frame.text);
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }
}

#[test]
fn v1_encoding() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/v1-iso-8859-1.mp3").unwrap() {
        match m {
            Unit::FrameV1(bytes) => {
                let v1 = frames::V1::new(bytes);
                let frame = v1.read().unwrap();
                assert_eq!("räksmörgås", frame.title);
                assert_eq!("räksmörgås", frame.artist);
                assert_eq!("räksmörgås", frame.album);
                assert_eq!("räksmörgås", frame.comment);
            },
            _ => ()
        }
    }

    for m in MetadataIterator::new("./test-resources/v1-utf8.mp3").unwrap() {
        match m {
            Unit::FrameV1(bytes) => {
                let v1 = frames::V1::new(bytes);
                let frame = v1.read().unwrap();
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.title);
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.artist);
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.album);
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.comment);
            },
            _ => ()
        }
    }
}

#[test]
fn v220() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/v2.2-pic.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, body, version) => {
                let v2 = frames::V2::new(head, body, version);
                let frame = v2.read().unwrap();
                match frame {
                    FrameData::PIC(frame) => {
                        comp_frame(FrameData::PIC(frame), &mut vec!["PNG:Other::61007"]);
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }
}