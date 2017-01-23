#[macro_use]
extern crate log;

pub extern crate regex;
extern crate env_logger;
extern crate rust_id3 as id3;

use std::vec::Vec;
use id3::frame::constants::{EventTimingCode, FrameData, FrameHeaderFlag, TimestampFormat};
use id3::metadata::{header, frames, MetaFrame, MetadataIterator, Unit};
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
fn metadata_regex() {
    let frame_id = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}$").unwrap();
    assert!(frame_id.is_match("AAA0"));
    assert!(frame_id.is_match("AAA"));
    assert!(!frame_id.is_match("0AA"));
    assert!(!frame_id.is_match("AA"));
    assert!(frame_id.is_match("COM"));
}

#[test]
fn metadata_empty() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/empty-meta.mp3").unwrap() {
        match m {
            Unit::FrameV1(_) => assert!(false),
            Unit::FrameV2(_, _) => assert!(false),
            _ => ()
        }
    }
}

#[test]
fn metadata_v1() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/v1-v2.mp3").unwrap() {
        match m {
            Unit::FrameV1(frame) => {
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
fn metadata_v1_no_id() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230-no-id3.mp3").unwrap() {
        match m {
            Unit::FrameV1(_) => assert!(false),
            _ => ()
        }
    }
}

#[test]
fn metadata_header() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230.mp3").unwrap() {
        match m {
            Unit::Header(header) => {
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
            Unit::Header(header) => {
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
fn metadata_frame_data() {
    let _ = env_logger::init();

    fn test(path: &str, mut data: Vec<&str>) {
        for m in MetadataIterator::new(path).unwrap() {
            match m {
                Unit::FrameV2(_, frame) => {
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
fn metadata_frame_etco() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230-etco.mp3").unwrap() {
        match m {
            Unit::FrameV2(_, FrameData::ETCO(frame)) => {
                assert_eq! ( &TimestampFormat::Milliseconds, &frame.timestamp_format);

                match frame.event_timing_codes[0] {
                    EventTimingCode::MainPartStart(timestamp) => assert_eq! (timestamp, 152110),
                    _ => assert! (false )
                }
            },
            _ => ()
        }
    }
}

#[test]
fn metadata_frame_pcnt() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/240-pcnt.mp3").unwrap() {
        match m {
            Unit::FrameV2(_, FrameData::PCNT(frame)) => assert_eq!(256, frame.counter),
            _ => ()
        }
    }
}

#[test]
fn metadata_frame_tbpm() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/230-tbpm.mp3").unwrap() {
        match m {
            Unit::FrameV2(_, FrameData::TBPM(frame)) => assert_eq!("0", frame.text),
            _ => ()
        }
    }
}

#[test]
fn metadata_v1_encoding() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/v1-iso-8859-1.mp3").unwrap() {
        match m {
            Unit::FrameV1(frame) => {
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
            Unit::FrameV1(frame) => {
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
fn metadata_v220() {
    let _ = env_logger::init();

    for m in MetadataIterator::new("./test-resources/v2.2-pic.mp3").unwrap() {
        match m {
            Unit::FrameV2(_, FrameData::PIC(frame)) => {
                comp_frame(FrameData::PIC(frame), &mut vec!["PNG:Other::61007"]);
            },
            _ => ()
        }
    }
}

#[test]
fn metadata_v230_compressed() {
    let _ = env_logger::init();

    let iter = MetadataIterator::new("./test-resources/v2.3-compressed-frame.mp3").unwrap();
    let mut i = iter.filter(|m| {
        match m {
            &Unit::FrameV2(ref header, _) => header.has_flag(FrameHeaderFlag::Compression),
            _ => false
        }
    });

    if let Unit::FrameV2(_, FrameData::TIT2(ref frame)) = i.next().unwrap() {
        assert_eq!("Compressed TIT2 Frame", frame.text)
    } else {
        assert!(false);
    }

    assert!(i.next().is_none());
}

#[test]
fn metadata_v230_encrypted() {
    let _ = env_logger::init();

    let iter = MetadataIterator::new("./test-resources/v2.3-encrypted-frame.mp3").unwrap();
    let mut i = iter.filter(|m| {
        match m {
            &Unit::FrameV2(ref head, _) => head.has_flag(FrameHeaderFlag::Encryption),
            _ => false
        }
    });

    if let Unit::FrameV2(_, FrameData::SKIP(_)) = i.next().unwrap() {
        assert!(true);
    } else {
        assert!(false);
    }

    assert!(i.next().is_none());
}

#[test]
fn metadata_v230_ext_header() {
    let _ = env_logger::init();

    // file with extend header bit set but no extended header
    {
        let iter = MetadataIterator::new("./test-resources/v2.3-ext-header-invalid.mp3").unwrap();
        let i = iter.filter(|m| {
            match m {
                &Unit::Header(ref header) => header.has_flag(header::Flag::ExtendedHeader),
                _ => false
            }
        });

        assert!(i.count() == 1);

        let iter = MetadataIterator::new("./test-resources/v2.3-ext-header-invalid.mp3").unwrap();
        let i = iter.filter(|m| {
            match m {
                &Unit::ExtendedHeader(_) => true,
                _ => false
            }
        });

        assert!(i.count() == 0);
    }

    {
        let iter = MetadataIterator::new("./test-resources/v2.3-ext-header.mp3").unwrap();
        let i = iter.filter(|m| {
            match m {
                &Unit::ExtendedHeader(_) => true,
                _ => false
            }
        });

        assert!(i.count() == 1);

        for m in MetadataIterator::new("./test-resources/v2.3-ext-header.mp3").unwrap() {
            match m {
                Unit::FrameV2(_, FrameData::TCON(frame)) => assert_eq!("(0)Blues", frame.text),
                _ => ()
            }
        };
    }
}