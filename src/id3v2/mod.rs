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
mod reader;
mod tag;

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::vec;
    use ::id3v2::reader::FrameIterator;

    fn _id_compare(file_path: &'static str, mut ids: vec::Vec<&str>) {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path(file_path).unwrap();
        let mut frame_reader = ::id3v2::reader::FrameReader::new(&mut readable).unwrap();
        ids.reverse();
        loop {
            if frame_reader.has_next_frame() {

                if let Ok(frame) = frame_reader.next_frame() {
                    debug!("{}: {:?}", frame.get_id(), ids);
                    assert_eq!(ids.pop().unwrap(), frame.get_id())
                }
            } else {
                break;
            }
        }
    }

    fn _data_compare(file_path: &'static str, mut data: vec::Vec<&str>) {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path(file_path).unwrap();
        let mut frame_reader = ::id3v2::reader::FrameReader::new(&mut readable).unwrap();
        data.reverse();
        loop {
            if frame_reader.has_next_frame() {
                if let Ok(frame) = frame_reader.next_frame() {
                    debug!("{}: {:?}", frame.get_id(), frame.get_data());

                    match frame.get_data().unwrap() {
                        ::id3v2::tag::frame_constants::FrameData::COMM(frame) => assert_eq!(data.pop().unwrap(), format!("{}\u{0}{}{}", frame.get_language(), frame.get_short_description(), frame.get_actual_text())),
                        ::id3v2::tag::frame_constants::FrameData::TALB(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TBPM(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TCOM(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TCON(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TCOP(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TDEN(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TDLY(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TDOR(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TDRC(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TDRL(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TDTG(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TENC(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TEXT(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TFLT(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TIPL(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TIT1(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TIT2(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TIT3(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TKEY(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TLAN(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TLEN(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TMCL(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TMED(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TMOO(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TOAL(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TOFN(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TOLY(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TOPE(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TOWN(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPE1(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPE2(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPE3(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPE4(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPOS(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPRO(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TPUB(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TRCK(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TRSN(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TRSO(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TSOA(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TSOP(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TSOT(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TSRC(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TSSE(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TSST(frame) => assert_eq!(data.pop().unwrap(), frame.get_text()),
                        ::id3v2::tag::frame_constants::FrameData::TXXX(frame) => assert_eq!(data.pop().unwrap(), format!("{}:{}", frame.get_description(), frame.get_value())),
                        _ => ()
                    };
                }
            } else {
                break;
            }
        }
    }

    #[test]
    fn idv3_230_header() {
        let _ = env_logger::init();
        let mut readable = ::readable::factory::from_path("./resources/230.mp3").unwrap();
        let bytes = readable.as_bytes(10).unwrap();
        let header = ::id3v2::tag::header::Header::new(bytes);
        assert_eq!(header.get_version(), 3);
        assert_eq!(header.get_minor_version(), 0);
        assert_eq!(header.has_flag(::id3v2::tag::header::HeaderFlag::Unsynchronisation), false);
        assert_eq!(header.has_flag(::id3v2::tag::header::HeaderFlag::ExtendedHeader), false);
        assert_eq!(header.has_flag(::id3v2::tag::header::HeaderFlag::ExperimentalIndicator), false);
        assert_eq!(header.get_size(), 1171);
    }

    #[test]
    fn idv3_240_header() {
        let _ = env_logger::init();

        let mut readable = ::readable::factory::from_path("./resources/240.mp3").unwrap();
        let bytes = readable.as_bytes(10).unwrap();
        let header = ::id3v2::tag::header::Header::new(bytes);
        assert_eq!(header.get_version(), 4);
        assert_eq!(header.get_minor_version(), 0);
        assert_eq!(header.has_flag(::id3v2::tag::header::HeaderFlag::Unsynchronisation), false);
        assert_eq!(header.has_flag(::id3v2::tag::header::HeaderFlag::ExtendedHeader), false);
        assert_eq!(header.has_flag(::id3v2::tag::header::HeaderFlag::ExperimentalIndicator), false);
        assert_eq!(header.get_size(), 165126);
    }

    #[test]
    fn idv3_230_frame_id() {
        _id_compare("./resources/id3v1-id3v2.mp3", vec!["TIT2", "TPE1", "TALB", "TPE2", "TCON", "COMM", "TRCK", "TPOS"]);
    }

    #[test]
    fn idv3_230_frame_id2() {
        _id_compare("./resources/230.mp3", vec!["TALB", "TCON", "TIT2", "TLEN", "TPE1", "TRCK", "COMM", "TYER"]);
    }

    #[test]
    fn idv3_240_frame_id1() {
        _id_compare("./resources/240.mp3", vec!["TDRC", "TRCK", "TPOS", "TPE1", "TALB", "TPE2", "TIT2", "TSRC", "TCON", "COMM"]);
    }

    #[test]
    fn id3_240_frame_id2() {
        _id_compare("./resources/id3v1-id3v2-albumimage.mp3", vec!["TENC", "WXXX", "TCOP", "TOPE", "TCOM", "COMM", "TPE1", "TALB", "COMM", "TRCK", "TDRC", "TCON", "TIT2", "APIC", "WCOM", "WCOP", "WOAR", "WOAF", "WOAS", "WORS", "WPAY", "WPUB"]);
    }

    #[test]
    fn idv3_230_frame_data1() {
        _data_compare("./resources/id3v1-id3v2.mp3", vec!["타이틀", "Artist", "アルバム", "Album Artist", "Heavy Metal", "eng\u{0}!@#$", "1", "0"]);
    }

    #[test]
    fn idv3_230_frame_data2() {
        _data_compare("./resources/230.mp3", vec!["\u{feff}앨범", "Rock", "\u{feff}Tㅏi틀", "0", "\u{feff}아티st", "1", "eng\u{0}!!!@@#$@$^#$%^\\n123", "2017"]);
    }

    #[test]
    fn idv3_240_frame_data1() {
        _data_compare("./resources/240.mp3", vec!["2017", "1", "1", "아티스트", "Album", "Artist/아티스트", "타이틀", "ABAB", "Alternative", "eng\u{0}~~"]);
    }

}