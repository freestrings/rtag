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

mod bytes;
mod header;
mod frame;
mod frame_constants;
mod reader;

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::vec;
    use ::id3v2::reader::FrameIterator;
    use ::id3v2::frame_constants::FrameData;

    fn _id_compare(file_path: &'static str, mut ids: vec::Vec<&str>) {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path(file_path).unwrap();
        let mut frame_reader = ::id3v2::reader::FrameReader::new(&mut readable).unwrap();
        ids.reverse();
        for frame in frame_reader {
            debug!("{}: {:?}", frame.get_id(), ids);
            assert_eq!(ids.pop().unwrap(), frame.get_id())
        }
        assert_eq!(ids.len(), 0);
    }

    fn _data_compare(file_path: &'static str, mut data: vec::Vec<&str>) {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path(file_path).unwrap();
        let mut frame_reader = ::id3v2::reader::FrameReader::new(&mut readable).unwrap();
        data.reverse();

        for frame in frame_reader {
            debug!("{}: {:?}", frame.get_id(), frame.get_data());
            let comp_data = data.pop().unwrap();
            match frame.get_data().unwrap() {
                FrameData::COMM(frame) => {
                    assert_eq!(comp_data, format!("{}:{}:{}",
                                                  frame.get_language(),
                                                  frame.get_short_description(),
                                                  frame.get_actual_text()));
                },
                FrameData::TALB(frame) |
                FrameData::TBPM(frame) |
                FrameData::TCOM(frame) |
                FrameData::TCON(frame) |
                FrameData::TCOP(frame) |
                FrameData::TDAT(frame) |
                FrameData::TDEN(frame) |
                FrameData::TDLY(frame) |
                FrameData::TDOR(frame) |
                FrameData::TDRC(frame) |
                FrameData::TDRL(frame) |
                FrameData::TDTG(frame) |
                FrameData::TENC(frame) |
                FrameData::TEXT(frame) |
                FrameData::TIME(frame) |
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
                FrameData::TORY(frame) |
                FrameData::TOWN(frame) |
                FrameData::TPE1(frame) |
                FrameData::TPE2(frame) |
                FrameData::TPE3(frame) |
                FrameData::TPE4(frame) |
                FrameData::TPOS(frame) |
                FrameData::TPRO(frame) |
                FrameData::TPUB(frame) |
                FrameData::TRCK(frame) |
                FrameData::TRDA(frame) |
                FrameData::TRSN(frame) |
                FrameData::TSIZ(frame) |
                FrameData::TRSO(frame) |
                FrameData::TSOA(frame) |
                FrameData::TSOP(frame) |
                FrameData::TSOT(frame) |
                FrameData::TSRC(frame) |
                FrameData::TSSE(frame) |
                FrameData::TYER(frame) |
                FrameData::TSST(frame)
                => {
                    assert_eq!(comp_data, frame.get_text())
                },
                FrameData::TXXX(frame) => {
                    assert_eq!(comp_data, format!("{}:{}",
                                                  frame.get_description(),
                                                  frame.get_value()))
                },

                FrameData::LINK(frame) |

                FrameData::WCOM(frame) |
                FrameData::WCOP(frame) |
                FrameData::WOAF(frame) |
                FrameData::WOAR(frame) |
                FrameData::WOAS(frame) |
                FrameData::WORS(frame) |
                FrameData::WPAY(frame) |
                FrameData::WPUB(frame) => {
                    assert_eq!(comp_data, format!("{}:{}",
                                                  frame.get_url(),
                                                  frame.get_additional_data()));
                },
                FrameData::WXXX(frame) => {
                    assert_eq!(comp_data, format!("{}:{}",
                                                  frame.get_url(),
                                                  frame.get_description()));
                }
                _ => ()
            };
        }

        assert_eq!(data.len(), 0);
    }

    #[test]
    fn idv3_230_header() {
        let _ = env_logger::init();
        let mut readable = ::readable::factory::from_path("./test-resources/230.mp3").unwrap();
        let bytes = readable.as_bytes(10).unwrap();
        let header = ::id3v2::header::Header::new(bytes).unwrap();
        assert_eq!(header.get_version(), 3);
        assert_eq!(header.get_minor_version(), 0);
        assert_eq!(header.has_flag(::id3v2::header::HeaderFlag::Unsynchronisation), false);
        assert_eq!(header.has_flag(::id3v2::header::HeaderFlag::ExtendedHeader), false);
        assert_eq!(header.has_flag(::id3v2::header::HeaderFlag::ExperimentalIndicator), false);
        assert_eq!(header.get_size(), 1171);
    }

    #[test]
    fn idv3_240_header() {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path("./test-resources/240.mp3").unwrap();
        let bytes = readable.as_bytes(10).unwrap();
        let header = ::id3v2::header::Header::new(bytes).unwrap();
        assert_eq!(header.get_version(), 4);
        assert_eq!(header.get_minor_version(), 0);
        assert_eq!(header.has_flag(::id3v2::header::HeaderFlag::Unsynchronisation), false);
        assert_eq!(header.has_flag(::id3v2::header::HeaderFlag::ExtendedHeader), false);
        assert_eq!(header.has_flag(::id3v2::header::HeaderFlag::ExperimentalIndicator), false);
        assert_eq!(header.get_size(), 165126);
    }

    #[test]
    fn idv3_230_frame_id() {
        _id_compare("./test-resources/id3v1-id3v2.mp3", vec!["TIT2", "TPE1", "TALB", "TPE2", "TCON", "COMM", "TRCK", "TPOS"]);
    }

    #[test]
    fn idv3_230_frame_id2() {
        _id_compare("./test-resources/230.mp3", vec!["TALB", "TCON", "TIT2", "TLEN", "TPE1", "TRCK", "COMM", "TYER"]);
    }

    #[test]
    fn idv3_240_frame_id1() {
        _id_compare("./test-resources/240.mp3", vec!["TDRC", "TRCK", "TPOS", "TPE1", "TALB", "TPE2", "TIT2", "TSRC", "TCON", "COMM"]);
    }

    #[test]
    fn id3_240_frame_id2() {
        _id_compare("./test-resources/id3v1-id3v2-albumimage.mp3", vec!["TENC", "WXXX", "TCOP", "TOPE", "TCOM", "COMM", "TPE1", "TALB", "COMM", "TRCK", "TDRC", "TCON", "TIT2", "APIC", "WCOM", "WCOP", "WOAR", "WOAF", "WOAS", "WORS", "WPAY", "WPUB"]);
    }

    #[test]
    fn idv3_230_frame_data1() {
        _data_compare("./test-resources/id3v1-id3v2.mp3", vec!["타이틀", "Artist", "アルバム", "Album Artist", "Heavy Metal", "eng::!@#$", "1", "0"]);
    }

    #[test]
    fn idv3_230_frame_data2() {
        _data_compare("./test-resources/230.mp3", vec!["앨범", "Rock", "Tㅏi틀", "0", "아티st", "1", "eng::!!!@@#$@$^#$%^\\n123", "2017"]);
    }

    #[test]
    fn idv3_240_frame_data1() {
        _data_compare("./test-resources/240.mp3", vec!["2017", "1", "1", "아티스트", "Album", "Artist/아티스트", "타이틀", "ABAB", "Alternative", "eng::~~"]);
    }

    #[test]
    fn id3v2_etco() {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path("./test-resources/230-etco.mp3").unwrap();
        let mut frame_reader = ::id3v2::reader::FrameReader::new(&mut readable).unwrap();
        if let Some(frame) = frame_reader.next() {
            assert_eq!("ETCO", frame.get_id());

            match frame.get_data().unwrap() {
                ::id3v2::frame_constants::FrameData::ETCO(frame) => {
                    let timestamp_format = frame.get_timestamp_format();
                    assert_eq!(timestamp_format, &::id3v2::frame_constants::TimestampFormat::Milliseconds);

                    let event_timing_codes = frame.get_event_timing_codes();
                    match event_timing_codes[0] {
                        ::id3v2::frame_constants::EventTimingCode::MainPartStart(timestamp) => assert_eq!(timestamp, 152110),
                        _ => assert!(false)
                    }
                },
                _ => assert!(false)
            }
        } else {
            assert!(false)
        }
    }

    #[test]
    fn id3v2_pcnt() {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path("./test-resources/240-pcnt.mp3").unwrap();
        let mut frame_reader = ::id3v2::reader::FrameReader::new(&mut readable).unwrap();
        if let Ok(frame) = frame_reader.next_frame() {
            assert_eq!("PCNT", frame.get_id());

            match frame.get_data().unwrap() {
                ::id3v2::frame_constants::FrameData::PCNT(frame) => {
                    let counter = frame.get_counter();
                    assert_eq!(counter, 256);
                },
                _ => assert!(false)
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn id3v2_tbpm() {
        _id_compare("./test-resources/230-tbpm.mp3", vec!["TRCK", "TBPM", "TCON", "TPE1", "TALB", "TIT2"]);
        _data_compare("./test-resources/230-tbpm.mp3", vec!["26", "0", "JPop", "aiko", "aikosingles", "花火"]);
    }
}