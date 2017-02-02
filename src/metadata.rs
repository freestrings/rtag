pub extern crate regex;
extern crate flate2;

use self::flate2::read::ZlibDecoder;

use errors::*;
use std::error::Error;
use frame::*;
use frame::id::*;
use util;

use readable::{
    Readable,
    ReadableFactory
};

use std::cell::RefCell;
use std::fs::File;
use std::io::{
    Cursor,
    Read
};
use std::iter::Iterator;
use std::rc::Rc;
use std::result;
use std::vec::Vec;

type RefHead = Rc<RefCell<Box<Head>>>;
type RefFileReader = Rc<RefCell<Box<Readable<File>>>>;
type RefByteReader = Rc<RefCell<Box<Readable<Cursor<Vec<u8>>>>>>;

#[derive(Debug)]
enum Status {
    Head(RefFileReader),
    ExtendedHeader(RefHead, RefFileReader),
    Frame(RefHead, RefFileReader, RefByteReader),
    None
}

#[derive(Debug)]
pub enum Unit {
    Header(Head),
    // TODO not yet implemented
    ExtendedHeader(Vec<u8>),
    FrameV1(Frame1),
    FrameV2(FrameHeader, FrameData),
    Unknown(String)
}

pub struct Metadata {
    next: Status,
    file_len: u64
}

impl Metadata {
    pub fn new(path: &str) -> result::Result<Self, ParsingError> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_len = metadata.len();
        let readable = file.to_readable();

        Ok(Metadata {
            next: Status::Head(Rc::new(RefCell::new(Box::new(readable)))),
            file_len: file_len
        })
    }

    fn has_frame_id(&mut self, readable: &mut Readable<Cursor<Vec<u8>>>) -> bool {
        match readable.look_string(4) {
            Ok(id) => {
                //
                // http://id3.org/id3v2.4.0-structure > 4. ID3v2 frame overview
                let regex = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}").unwrap();
                let matched = regex.is_match(&id);
                debug!("Frame Id:'{}', reg matched: {}", id, matched);

                matched
            },
            _ => false
        }
    }

    fn head(&mut self, readable_wrap: RefFileReader) -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();

        let head = Head::new(readable.to_readable(10)?)?;
        debug!("{:?}", head);

        let is_extended = head.has_flag(HeadFlag::ExtendedHeader);
        let head_wrap = Rc::new(RefCell::new(Box::new(head.clone())));

        let next = if is_extended {
            Status::ExtendedHeader(head_wrap, readable_wrap.clone())
        } else {
            let head_size = head.size as usize;
            let frame_bytes = if head.has_flag(HeadFlag::Unsynchronisation) {
                let mut bytes = readable.bytes(head_size)?;
                util::to_synchronize(&mut bytes);
                bytes
            } else {
                readable.bytes(head_size)?
            };
            let frame_readable = Cursor::new(frame_bytes).to_readable();
            let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

            Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap)
        };

        self.next = next;

        Ok(Unit::Header(head))
    }

    // optional unit
    fn extended_head(&mut self,
                     head_wrap: RefHead,
                     readable_wrap: RefFileReader) -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();

        let size = match head_wrap.borrow().version {
            //
            // Did not explained for whether big-endian or synchsafe in "http://id3.org/id3v2.3.0".
            3 => readable.u32()?,
            //
            // `Extended header size` stored as a 32 bit synchsafe integer in "2.4.0".
            _ => readable.synchsafe()?
        };
        let extended_bytes = readable.bytes(size as usize)?;
        let head_size = head_wrap.borrow().size as usize;
        let frame_bytes = readable.bytes(head_size)?;
        let frame_readable = Cursor::new(frame_bytes).to_readable();
        let frame_readable_wrap = Rc::new(RefCell::new(Box::new(frame_readable)));

        self.next = Status::Frame(head_wrap, readable_wrap.clone(), frame_readable_wrap);

        Ok(Unit::ExtendedHeader(extended_bytes))
    }

    fn frame1(&mut self, readable: &mut Readable<File>) -> result::Result<Frame1, ParsingError> {
        if self.file_len < 128 {
            return Err(ParsingError::BadData(ParsingErrorKind::InvalidFrameLength));
        }

        readable.skip((self.file_len - 128) as i64)?;

        if readable.string(3)? != "TAG" {
            let _ = readable.skip(-3);
            debug!("{}", util::to_hex(&readable.bytes(3)?));
            return Err(ParsingError::BadData(ParsingErrorKind::InvalidV1FrameId));
        }

        Frame1::new(&mut Cursor::new(readable.all_bytes()?).to_readable())
    }

    // version 2.2
    fn frame2(&mut self,
              head: &Head,
              readable: &mut Readable<Cursor<Vec<u8>>>) -> result::Result<Unit, ParsingError> {
        let id = readable.string(3)?;
        let size = readable.u24()?;
        let frame_header = FrameHeader::new(id.to_string(), head.version, 0, 0);
        let frame_readable = readable.to_readable(size as usize)?;
        let frame_body = frame_data(id.as_str(), head.version, &frame_header, frame_readable)?;

        Ok(Unit::FrameV2(frame_header, frame_body))
    }

    // v2.3
    fn frame3(&mut self, head: &Head, readable: &mut Readable<Cursor<Vec<u8>>>) -> result::Result<Unit, ParsingError> {
        let id = readable.string(4)?;
        let size = readable.u32()?;
        let status_flag = readable.u8()?;
        let encoding_flag = readable.u8()?;
        let frame_header = FrameHeader::new(id.to_string(), head.version, status_flag, encoding_flag);

        let mut extra_size: u32 = 0;
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        let body_bytes = if frame_header.has_flag(FrameHeaderFlag::Compression) {
            let _ = readable.u32()?;
            extra_size = extra_size + 4;

            let actual_size = size - extra_size as u32;
            let body_bytes = readable.bytes(actual_size as usize)?;
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&body_bytes[..]);

            let _ = decoder.read_to_end(&mut out);

            out
        } else {
            let actual_size = size - extra_size as u32;
            readable.bytes(actual_size as usize)?
        };

        let frame_readable = Cursor::new(body_bytes).to_readable();
        let frame_body = frame_data(id.as_str(), head.version, &frame_header, frame_readable)?;

        Ok(Unit::FrameV2(frame_header, frame_body))
    }

    // v2.4
    fn frame4(&mut self,
              head: &Head,
              readable: &mut Readable<Cursor<Vec<u8>>>) -> result::Result<Unit, ParsingError> {
        let id = readable.string(4)?;
        let size = readable.synchsafe()?;
        let status_flag = readable.u8()?;
        let encoding_flag = readable.u8()?;
        let frame_header = FrameHeader::new(id.to_string(), head.version, status_flag, encoding_flag);


        let mut extra_size: u32 = 0;
        if frame_header.has_flag(FrameHeaderFlag::GroupIdentity) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::Encryption) {
            let _ = readable.u8()?;
            extra_size = extra_size + 1;
        }

        if frame_header.has_flag(FrameHeaderFlag::DataLength) {
            let _ = readable.u32()?;
            extra_size = extra_size + 4;
        }

        let actual_size = size - extra_size as u32;
        let mut body_bytes = readable.bytes(actual_size as usize)?;

        if frame_header.has_flag(FrameHeaderFlag::Unsynchronisation) {
            debug!("'{}' is unsynchronised", id);

            let mut out = body_bytes[..].to_vec();
            let sync_size = util::to_synchronize(&mut out);

            //cut to synchrosized size
            out.split_off(sync_size);

            body_bytes = out;
        }

        if frame_header.has_flag(FrameHeaderFlag::Compression) {
            debug!("'{}' is compressed", id);

            let _ = readable.u32()?;

            let real_frame = body_bytes.clone();
            let mut out = vec![];
            let mut decoder = ZlibDecoder::new(&real_frame[..]);
            let _ = decoder.read_to_end(&mut out);

            body_bytes = out;
        }

        let frame_readable = Cursor::new(body_bytes).to_readable();
        let frame_body = frame_data(id.as_str(), head.version, &frame_header, frame_readable)?;

        Ok(Unit::FrameV2(frame_header, frame_body))
    }

    fn frame(&mut self,
             head_wrap: RefHead,
             readable_wrap: RefFileReader,
             frame_readable_wrap: RefByteReader) -> result::Result<Unit, ParsingError> {
        let mut readable = readable_wrap.borrow_mut();
        let mut frame_readable = frame_readable_wrap.borrow_mut();
        //
        // frame v1
        if !self.has_frame_id(&mut frame_readable) {
            self.next = Status::None;
            return Ok(Unit::FrameV1(self.frame1(&mut readable)?));
        }

        //
        // frame v2
        match head_wrap.borrow().version {
            2 => self.frame2(&head_wrap.borrow(), &mut frame_readable),
            3 => self.frame3(&head_wrap.borrow(), &mut frame_readable),
            4 => self.frame4(&head_wrap.borrow(), &mut frame_readable),
            _ => self.frame4(&head_wrap.borrow(), &mut frame_readable)
        }
    }
}

impl Iterator for Metadata {
    type Item = Unit;

    fn next(&mut self) -> Option<(Self::Item)> {
        match self.next {
            Status::Head(_) => debug!("next: Head"),
            Status::ExtendedHeader(_, _) => debug!("next: ExtendedHeader"),
            Status::Frame(_, _, _) => debug!("next: Frame"),
            Status::None => debug!("next: None"),
        };

        fn head(next: &Status) -> Option<RefFileReader> {
            match next {
                &Status::Head(ref readable) => Some(readable.clone()),
                _ => None
            }
        }

        fn extended_head(next: &Status) -> Option<(RefHead, RefFileReader)> {
            match next {
                &Status::ExtendedHeader(ref head, ref readable) =>
                    Some((head.clone(), readable.clone())),
                _ => None
            }
        }

        fn frame(next: &Status) -> Option<(RefHead, RefFileReader, RefByteReader)> {
            match next {
                &Status::Frame(ref head, ref readable, ref frame_readable) =>
                    Some((head.clone(), readable.clone(), frame_readable.clone())),
                _ => None
            }
        }

        let head = head(&self.next);
        let extended_header = extended_head(&self.next);
        let frame = frame(&self.next);

        match self.next {
            Status::Head(_) => match self.head(head.unwrap()) {
                Ok(data) => Some(data),
                Err(msg) => {
                    debug!("Stop on 'Head': {}", msg);
                    None
                }
            },
            Status::ExtendedHeader(_, _) => {
                let (head, readable) = extended_header.unwrap();
                match self.extended_head(head, readable) {
                    Ok(data) => Some(data),
                    Err(msg) => {
                        debug!("Stop on 'Extended Head': {}", msg);
                        None
                    }
                }
            },
            Status::Frame(_, _, _) => {
                let (head, readable, frame_readable) = frame.unwrap();
                match self.frame(head, readable, frame_readable) {
                    Ok(data) => {
                        Some(data)
                    },
                    Err(msg) => {
                        debug!("Ignored 'Frame': {}", msg);
                        Some(Unit::Unknown(msg.description().to_string()))
                    }
                }
            }
            _ => None
        }
    }
}

fn frame_data(id: &str,
              version: u8,
              frame_header: &FrameHeader,
              mut readable: Readable<Cursor<Vec<u8>>>) -> result::Result<FrameData, ParsingError> {
    if frame_header.has_flag(FrameHeaderFlag::Encryption) {
        return Ok(FrameData::SKIP("Encrypted frame".to_string()));
    };

    let frame_data = match id.as_ref() {
        BUF_STR => FrameData::BUF(BUF::read(&mut readable)?),
        CNT_STR => FrameData::PCNT(PCNT::read(&mut readable)?),
        COM_STR => FrameData::COMM(COMM::read(&mut readable)?),
        CRA_STR => FrameData::AENC(AENC::read(&mut readable)?),
        CRM_STR => FrameData::CRM(CRM::read(&mut readable)?),
        ETC_STR => FrameData::ETCO(ETCO::read(&mut readable)?),
        EQU_STR => FrameData::EQUA(EQUA::read(&mut readable)?),
        GEO_STR => FrameData::GEOB(GEOB::read(&mut readable)?),
        IPL_STR => FrameData::IPLS(IPLS::read(&mut readable)?),
        LNK_STR => FrameData::LINK(LINK::read(&mut readable, version)?),
        MCI_STR => FrameData::MCDI(MCDI::read(&mut readable)?),
        MLL_STR => FrameData::MLLT(MLLT::read(&mut readable)?),
        PIC_STR => FrameData::PIC(PIC::read(&mut readable)?),
        POP_STR => FrameData::POPM(POPM::read(&mut readable)?),
        REV_STR => FrameData::RVRB(RVRB::read(&mut readable)?),
        RVA_STR => FrameData::RVAD(RVA2::read(&mut readable)?),
        SLT_STR => FrameData::SYLT(SYLT::read(&mut readable)?),
        STC_STR => FrameData::SYTC(SYTC::read(&mut readable)?),
        TAL_STR => FrameData::TALB(TEXT::read(&mut readable, id)?),
        TBP_STR => FrameData::TBPM(TEXT::read(&mut readable, id)?),
        TCM_STR => FrameData::TCOM(TEXT::read(&mut readable, id)?),
        TCO_STR => FrameData::TCON(TEXT::read(&mut readable, id)?),
        TCR_STR => FrameData::TCOP(TEXT::read(&mut readable, id)?),
        TDA_STR => FrameData::TDAT(TEXT::read(&mut readable, id)?),
        TDY_STR => FrameData::TDLY(TEXT::read(&mut readable, id)?),
        TEN_STR => FrameData::TENC(TEXT::read(&mut readable, id)?),
        TFT_STR => FrameData::TFLT(TEXT::read(&mut readable, id)?),
        TIM_STR => FrameData::TIME(TEXT::read(&mut readable, id)?),
        TKE_STR => FrameData::TKEY(TEXT::read(&mut readable, id)?),
        TLA_STR => FrameData::TLAN(TEXT::read(&mut readable, id)?),
        TLE_STR => FrameData::TLEN(TEXT::read(&mut readable, id)?),
        TMT_STR => FrameData::TMED(TEXT::read(&mut readable, id)?),
        TOA_STR => FrameData::TMED(TEXT::read(&mut readable, id)?),
        TOF_STR => FrameData::TOFN(TEXT::read(&mut readable, id)?),
        TOL_STR => FrameData::TOLY(TEXT::read(&mut readable, id)?),
        TOR_STR => FrameData::TORY(TEXT::read(&mut readable, id)?),
        TOT_STR => FrameData::TOAL(TEXT::read(&mut readable, id)?),
        TP1_STR => FrameData::TPE1(TEXT::read(&mut readable, id)?),
        TP2_STR => FrameData::TPE2(TEXT::read(&mut readable, id)?),
        TP3_STR => FrameData::TPE3(TEXT::read(&mut readable, id)?),
        TP4_STR => FrameData::TPE4(TEXT::read(&mut readable, id)?),
        TPA_STR => FrameData::TPOS(TEXT::read(&mut readable, id)?),
        TPB_STR => FrameData::TPUB(TEXT::read(&mut readable, id)?),
        TRC_STR => FrameData::TSRC(TEXT::read(&mut readable, id)?),
        TRD_STR => FrameData::TRDA(TEXT::read(&mut readable, id)?),
        TRK_STR => FrameData::TRCK(TEXT::read(&mut readable, id)?),
        TSI_STR => FrameData::TSIZ(TEXT::read(&mut readable, id)?),
        TSS_STR => FrameData::TSSE(TEXT::read(&mut readable, id)?),
        TT1_STR => FrameData::TIT1(TEXT::read(&mut readable, id)?),
        TT2_STR => FrameData::TIT2(TEXT::read(&mut readable, id)?),
        TT3_STR => FrameData::TIT3(TEXT::read(&mut readable, id)?),
        TXT_STR => FrameData::TEXT(TEXT::read(&mut readable, id)?),
        TXX_STR => FrameData::TXXX(TXXX::read(&mut readable)?),
        TYE_STR => FrameData::TYER(TEXT::read(&mut readable, id)?),
        UFI_STR => FrameData::UFID(UFID::read(&mut readable)?),
        ULT_STR => FrameData::USLT(USLT::read(&mut readable)?),
        WAF_STR => FrameData::WOAF(LINK::read(&mut readable, version)?),
        WAR_STR => FrameData::WOAR(LINK::read(&mut readable, version)?),
        WAS_STR => FrameData::WOAS(LINK::read(&mut readable, version)?),
        WCM_STR => FrameData::WCOM(LINK::read(&mut readable, version)?),
        WCP_STR => FrameData::WCOP(LINK::read(&mut readable, version)?),
        WPB_STR => FrameData::WPUB(LINK::read(&mut readable, version)?),
        WXX_STR => FrameData::WXXX(WXXX::read(&mut readable)?),
        AENC_STR => FrameData::AENC(AENC::read(&mut readable)?),
        APIC_STR => FrameData::APIC(APIC::read(&mut readable)?),
        ASPI_STR => FrameData::ASPI(ASPI::read(&mut readable)?),
        COMM_STR => FrameData::COMM(COMM::read(&mut readable)?),
        COMR_STR => FrameData::COMR(COMR::read(&mut readable)?),
        ENCR_STR => FrameData::ENCR(ENCR::read(&mut readable)?),
        EQUA_STR => FrameData::EQUA(EQUA::read(&mut readable)?),
        EQU2_STR => FrameData::EQU2(EQU2::read(&mut readable)?),
        ETCO_STR => FrameData::ETCO(ETCO::read(&mut readable)?),
        GEOB_STR => FrameData::GEOB(GEOB::read(&mut readable)?),
        GRID_STR => FrameData::GRID(GRID::read(&mut readable)?),
        IPLS_STR => FrameData::IPLS(IPLS::read(&mut readable)?),
        LINK_STR => FrameData::LINK(LINK::read(&mut readable, version)?),
        MCDI_STR => FrameData::MCDI(MCDI::read(&mut readable)?),
        MLLT_STR => FrameData::MLLT(MLLT::read(&mut readable)?),
        OWNE_STR => FrameData::OWNE(OWNE::read(&mut readable)?),
        PRIV_STR => FrameData::PRIV(PRIV::read(&mut readable)?),
        PCNT_STR => FrameData::PCNT(PCNT::read(&mut readable)?),
        POPM_STR => FrameData::POPM(POPM::read(&mut readable)?),
        POSS_STR => FrameData::POSS(POSS::read(&mut readable)?),
        RBUF_STR => FrameData::RBUF(RBUF::read(&mut readable)?),
        RVAD_STR => FrameData::RVAD(RVA2::read(&mut readable)?),
        RVA2_STR => FrameData::RVA2(RVA2::read(&mut readable)?),
        RVRB_STR => FrameData::RVRB(RVRB::read(&mut readable)?),
        SEEK_STR => FrameData::SEEK(SEEK::read(&mut readable)?),
        SIGN_STR => FrameData::SIGN(SIGN::read(&mut readable)?),
        SYLT_STR => FrameData::SYLT(SYLT::read(&mut readable)?),
        SYTC_STR => FrameData::SYTC(SYTC::read(&mut readable)?),
        UFID_STR => FrameData::UFID(UFID::read(&mut readable)?),
        USER_STR => FrameData::USER(USER::read(&mut readable)?),
        USLT_STR => FrameData::USLT(USLT::read(&mut readable)?),
        TALB_STR => FrameData::TALB(TEXT::read(&mut readable, id)?),
        TBPM_STR => FrameData::TBPM(TEXT::read(&mut readable, id)?),
        TCOM_STR => FrameData::TCOM(TEXT::read(&mut readable, id)?),
        TCON_STR => FrameData::TCON(TEXT::read(&mut readable, id)?),
        TCOP_STR => FrameData::TCOP(TEXT::read(&mut readable, id)?),
        TDAT_STR => FrameData::TDAT(TEXT::read(&mut readable, id)?),
        TDEN_STR => FrameData::TDEN(TEXT::read(&mut readable, id)?),
        TDLY_STR => FrameData::TDLY(TEXT::read(&mut readable, id)?),
        TDOR_STR => FrameData::TDOR(TEXT::read(&mut readable, id)?),
        TDRC_STR => FrameData::TDRC(TEXT::read(&mut readable, id)?),
        TDRL_STR => FrameData::TDRL(TEXT::read(&mut readable, id)?),
        TDTG_STR => FrameData::TDTG(TEXT::read(&mut readable, id)?),
        TENC_STR => FrameData::TENC(TEXT::read(&mut readable, id)?),
        TEXT_STR => FrameData::TEXT(TEXT::read(&mut readable, id)?),
        TIME_STR => FrameData::TIME(TEXT::read(&mut readable, id)?),
        TFLT_STR => FrameData::TFLT(TEXT::read(&mut readable, id)?),
        TIPL_STR => FrameData::TIPL(TEXT::read(&mut readable, id)?),
        TIT1_STR => FrameData::TIT1(TEXT::read(&mut readable, id)?),
        TIT2_STR => FrameData::TIT2(TEXT::read(&mut readable, id)?),
        TIT3_STR => FrameData::TIT3(TEXT::read(&mut readable, id)?),
        TKEY_STR => FrameData::TKEY(TEXT::read(&mut readable, id)?),
        TLAN_STR => FrameData::TLAN(TEXT::read(&mut readable, id)?),
        TLEN_STR => FrameData::TLEN(TEXT::read(&mut readable, id)?),
        TMCL_STR => FrameData::TMCL(TEXT::read(&mut readable, id)?),
        TMED_STR => FrameData::TMED(TEXT::read(&mut readable, id)?),
        TMOO_STR => FrameData::TMOO(TEXT::read(&mut readable, id)?),
        TOAL_STR => FrameData::TOAL(TEXT::read(&mut readable, id)?),
        TOFN_STR => FrameData::TOFN(TEXT::read(&mut readable, id)?),
        TOLY_STR => FrameData::TOLY(TEXT::read(&mut readable, id)?),
        TOPE_STR => FrameData::TOPE(TEXT::read(&mut readable, id)?),
        TORY_STR => FrameData::TORY(TEXT::read(&mut readable, id)?),
        TOWN_STR => FrameData::TOWN(TEXT::read(&mut readable, id)?),
        TPE1_STR => FrameData::TPE1(TEXT::read(&mut readable, id)?),
        TPE2_STR => FrameData::TPE2(TEXT::read(&mut readable, id)?),
        TPE3_STR => FrameData::TPE3(TEXT::read(&mut readable, id)?),
        TPE4_STR => FrameData::TPE4(TEXT::read(&mut readable, id)?),
        TPOS_STR => FrameData::TPOS(TEXT::read(&mut readable, id)?),
        TPRO_STR => FrameData::TPRO(TEXT::read(&mut readable, id)?),
        TPUB_STR => FrameData::TPUB(TEXT::read(&mut readable, id)?),
        TRCK_STR => FrameData::TRCK(TEXT::read(&mut readable, id)?),
        TRDA_STR => FrameData::TRDA(TEXT::read(&mut readable, id)?),
        TRSN_STR => FrameData::TRSN(TEXT::read(&mut readable, id)?),
        TSIZ_STR => FrameData::TSIZ(TEXT::read(&mut readable, id)?),
        TRSO_STR => FrameData::TRSO(TEXT::read(&mut readable, id)?),
        TSOA_STR => FrameData::TSOA(TEXT::read(&mut readable, id)?),
        TSOP_STR => FrameData::TSOP(TEXT::read(&mut readable, id)?),
        TSOT_STR => FrameData::TSOT(TEXT::read(&mut readable, id)?),
        TSRC_STR => FrameData::TSRC(TEXT::read(&mut readable, id)?),
        TSSE_STR => FrameData::TSSE(TEXT::read(&mut readable, id)?),
        TYER_STR => FrameData::TYER(TEXT::read(&mut readable, id)?),
        TSST_STR => FrameData::TSST(TEXT::read(&mut readable, id)?),
        TXXX_STR => FrameData::TXXX(TXXX::read(&mut readable)?),
        WCOM_STR => FrameData::WCOM(LINK::read(&mut readable, version)?),
        WCOP_STR => FrameData::WCOP(LINK::read(&mut readable, version)?),
        WOAF_STR => FrameData::WOAF(LINK::read(&mut readable, version)?),
        WOAR_STR => FrameData::WOAR(LINK::read(&mut readable, version)?),
        WOAS_STR => FrameData::WOAS(LINK::read(&mut readable, version)?),
        WORS_STR => FrameData::WORS(LINK::read(&mut readable, version)?),
        WPAY_STR => FrameData::WPAY(LINK::read(&mut readable, version)?),
        WPUB_STR => FrameData::WPUB(LINK::read(&mut readable, version)?),
        WXXX_STR => FrameData::WXXX(WXXX::read(&mut readable)?),
        _ => {
            warn!("No frame id found!! '{}'", id);
            FrameData::TEXT(TEXT::read(&mut readable, id)?)
        }
    };

    Ok(frame_data)
}