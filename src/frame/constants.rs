pub mod id {
    pub const AENC_STR: &'static str = "AENC";
    pub const APIC_STR: &'static str = "APIC";
    pub const ASPI_STR: &'static str = "ASPI";
    pub const COMM_STR: &'static str = "COMM";
    pub const COMR_STR: &'static str = "COMR";
    pub const ENCR_STR: &'static str = "ENCR";
    pub const EQU2_STR: &'static str = "EQU2";
    // 2.3 only
    pub const EQUA_STR: &'static str = "EQUA";
    pub const ETCO_STR: &'static str = "ETCO";
    pub const GEOB_STR: &'static str = "GEOB";
    pub const GRID_STR: &'static str = "GRID";
    // 2.3 only
    pub const IPLS_STR: &'static str = "IPLS";
    pub const LINK_STR: &'static str = "LINK";
    pub const MCDI_STR: &'static str = "MCDI";
    pub const MLLT_STR: &'static str = "MLLT";
    pub const OWNE_STR: &'static str = "OWNE";
    pub const PRIV_STR: &'static str = "PRIV";
    pub const PCNT_STR: &'static str = "PCNT";
    pub const POPM_STR: &'static str = "POPM";
    pub const POSS_STR: &'static str = "POSS";
    pub const RBUF_STR: &'static str = "RBUF";
    // 2.3 only
    pub const RVAD_STR: &'static str = "RVAD";
    pub const RVA2_STR: &'static str = "RVA2";
    pub const RVRB_STR: &'static str = "RVRB";
    pub const SEEK_STR: &'static str = "SEEK";
    pub const SIGN_STR: &'static str = "SIGN";
    pub const SYLT_STR: &'static str = "SYLT";
    pub const SYTC_STR: &'static str = "SYTC";
    pub const TALB_STR: &'static str = "TALB";
    pub const TBPM_STR: &'static str = "TBPM";
    pub const TCOM_STR: &'static str = "TCOM";
    pub const TCON_STR: &'static str = "TCON";
    pub const TCOP_STR: &'static str = "TCOP";
    // 2.3 only
    pub const TDAT_STR: &'static str = "TDAT";
    pub const TDEN_STR: &'static str = "TDEN";
    pub const TDLY_STR: &'static str = "TDLY";
    pub const TDOR_STR: &'static str = "TDOR";
    pub const TDRC_STR: &'static str = "TDRC";
    pub const TDTG_STR: &'static str = "TDTG";
    pub const TDRL_STR: &'static str = "TDRL";
    pub const TENC_STR: &'static str = "TENC";
    pub const TEXT_STR: &'static str = "TEXT";
    pub const TFLT_STR: &'static str = "TFLT";
    // 2.3 only
    pub const TIME_STR: &'static str = "TIME";
    pub const TIPL_STR: &'static str = "TIPL";
    pub const TIT1_STR: &'static str = "TIT1";
    pub const TIT2_STR: &'static str = "TIT2";
    pub const TIT3_STR: &'static str = "TIT3";
    pub const TKEY_STR: &'static str = "TKEY";
    pub const TLAN_STR: &'static str = "TLAN";
    pub const TLEN_STR: &'static str = "TLEN";
    pub const TMCL_STR: &'static str = "TMCL";
    pub const TMED_STR: &'static str = "TMED";
    pub const TMOO_STR: &'static str = "TMOO";
    pub const TOAL_STR: &'static str = "TOAL";
    pub const TOFN_STR: &'static str = "TOFN";
    pub const TOLY_STR: &'static str = "TOLY";
    pub const TOPE_STR: &'static str = "TOPE";
    pub const TORY_STR: &'static str = "TORY";
    pub const TOWN_STR: &'static str = "TOWN";
    pub const TPE1_STR: &'static str = "TPE1";
    pub const TPE2_STR: &'static str = "TPE2";
    pub const TPE3_STR: &'static str = "TPE3";
    pub const TPE4_STR: &'static str = "TPE4";
    pub const TPOS_STR: &'static str = "TPOS";
    pub const TPRO_STR: &'static str = "TPRO";
    pub const TPUB_STR: &'static str = "TPUB";
    pub const TRCK_STR: &'static str = "TRCK";
    pub const TRDA_STR: &'static str = "TRDA";
    pub const TRSN_STR: &'static str = "TRSN";
    pub const TRSO_STR: &'static str = "TRSO";
    // 2.3 only
    pub const TSIZ_STR: &'static str = "TSIZ";
    pub const TSOA_STR: &'static str = "TSOA";
    pub const TSOP_STR: &'static str = "TSOP";
    pub const TSOT_STR: &'static str = "TSOT";
    pub const TSRC_STR: &'static str = "TSRC";
    pub const TSSE_STR: &'static str = "TSSE";
    // 2.3 only
    pub const TYER_STR: &'static str = "TYER";
    pub const TSST_STR: &'static str = "TSST";
    pub const TXXX_STR: &'static str = "TXXX";
    pub const UFID_STR: &'static str = "UFID";
    pub const USER_STR: &'static str = "USER";
    pub const USLT_STR: &'static str = "USLT";
    pub const WCOM_STR: &'static str = "WCOM";
    pub const WCOP_STR: &'static str = "WCOP";
    pub const WOAF_STR: &'static str = "WOAF";
    pub const WOAR_STR: &'static str = "WOAR";
    pub const WOAS_STR: &'static str = "WOAS";
    pub const WORS_STR: &'static str = "WORS";
    pub const WPAY_STR: &'static str = "WPAY";
    pub const WPUB_STR: &'static str = "WPUB";
    pub const WXXX_STR: &'static str = "WXXX";
}

#[derive(Debug)]
pub enum TextEncoding {
    ISO8859_1,
    UTF16LE,
    UTF16BE,
    UTF8
}

use ::frame::*;

#[derive(Debug)]
pub enum FrameData {
    AENC(AENC),
    APIC(APIC),
    ASPI(ASPI),
    COMM(COMM),
    COMR(COMR),
    ENCR(ENCR),
    // 2.3 only
    EQUA(EQUA),
    EQU2(EQU2),
    ETCO(ETCO),
    GEOB(GEOB),
    GRID(GRID),
    // 2.3 only
    IPLS(IPLS),
    LINK(LINK),
    MCDI(MCDI),
    MLLT(MLLT),
    OWNE(OWNE),
    PRIV(PRIV),
    PCNT(PCNT),
    POPM(POPM),
    POSS(POSS),
    RBUF(RBUF),
    // 2.3 only
    RVAD(RVA2),
    RVA2(RVA2),
    RVRB(RVRB),
    SEEK(SEEK),
    SIGN(SIGN),
    SYLT(SYLT),
    SYTC(SYTC),
    TALB(TEXT),
    TBPM(TEXT),
    TCOM(TEXT),
    TCON(TEXT),
    TCOP(TEXT),
    // 2.3 only
    TDAT(TEXT),
    TDEN(TEXT),
    TDLY(TEXT),
    TDOR(TEXT),
    TDRC(TEXT),
    TDRL(TEXT),
    TDTG(TEXT),
    TENC(TEXT),
    TEXT(TEXT),
    TFLT(TEXT),
    // 2.3 only
    TIME(TEXT),
    TIPL(TEXT),
    TIT1(TEXT),
    TIT2(TEXT),
    TIT3(TEXT),
    TKEY(TEXT),
    TLAN(TEXT),
    TLEN(TEXT),
    TMCL(TEXT),
    TMED(TEXT),
    TMOO(TEXT),
    TOAL(TEXT),
    TOFN(TEXT),
    TOLY(TEXT),
    TOPE(TEXT),
    TORY(TEXT),
    TOWN(TEXT),
    TPE1(TEXT),
    TPE2(TEXT),
    TPE3(TEXT),
    TPE4(TEXT),
    TPOS(TEXT),
    TPRO(TEXT),
    TPUB(TEXT),
    TRCK(TEXT),
    // 2.3 only
    TRDA(TEXT),
    TRSN(TEXT),
    TRSO(TEXT),
    // 2.3 only
    TSIZ(TEXT),
    TSOA(TEXT),
    TSOP(TEXT),
    TSOT(TEXT),
    TSRC(TEXT),
    TSSE(TEXT),
    // 2.3 only
    TYER(TEXT),
    TSST(TEXT),
    TXXX(TXXX),
    UFID(UFID),
    USER(USER),
    USLT(USLT),
    WCOM(LINK),
    WCOP(LINK),
    WOAF(LINK),
    WOAR(LINK),
    WOAS(LINK),
    WORS(LINK),
    WPAY(LINK),
    WPUB(LINK),
    WXXX(WXXX),
    INVALID(String)
}

#[derive(Debug, PartialEq)]
pub enum PictureType {
    Other,
    FileIcon,
    OtherFileIcon,
    CoverFront,
    CoverBack,
    LeafletPage,
    Media,
    LeadArtist,
    Artist,
    Conductor,
    Band,
    Composer,
    Lyricist,
    RecordingLocation,
    DuringRecording,
    DuringPerformance,
    MovieScreenCapture,
    BrightColouredFish,
    Illustration,
    BandLogotype,
    PublisherLogoType
}

#[derive(Debug, PartialEq)]
pub enum ReceivedAs {
    Other,
    StandardCDAlbum,
    CompressedAudioOnCD,
    FileOverInternet,
    StreamOverInternet,
    AsNoteSheets,
    AsNoteSheetsInBook,
    MusicOnMedia,
    NonMusicalMerchandise
}

#[derive(Debug)]
pub enum InterpolationMethod {
    Band,
    Linear
}

#[derive(Debug, PartialEq)]
pub enum ContentType {
    Other,
    Lyrics,
    TextTranscription,
    MovementName,
    Events,
    Chord,
    Trivia,
    UrlsToWebpages,
    UrlsToImages
}

#[derive(Debug, PartialEq)]
pub enum TimestampFormat {
    MpecFrames,
    Milliseconds
}

#[derive(Debug)]
pub enum EventTimingCode {
    Padding(u32),
    EndOfInitialSilence(u32),
    IntroStart(u32),
    MainPartStart(u32),
    OutroStart(u32),
    OutroEnd(u32),
    VerseStart(u32),
    RefrainStart(u32),
    InterludeStart(u32),
    ThemeStart(u32),
    VariationStart(u32),
    KeyChange(u32),
    TimeChange(u32),
    MomentaryUnwantedNoise(u32),
    SustainedNoise(u32),
    SustainedNoiseEnd(u32),
    IntroEnd(u32),
    MainPartEnd(u32),
    VerseEnd(u32),
    RefrainEnd(u32),
    ThemeEnd(u32),
    Profanity(u32),
    ProfanityEnd(u32),
    ReservedForFutureUse(u32),
    NotPredefinedSynch(u32),
    AudioEnd(u32),
    AudioFileEnds(u32),
    OneMoreByteOfEventsFollows(u32)
}

#[derive(Debug, PartialEq)]
pub enum FrameHeaderFlag {
    TagAlter,
    FileAlter,
    ReadOnly,
    Compression,
    Encryption,
    GroupIdentity,
    //2.4 only
    Unsynchronisation,
    //2.4 only
    DataLength
}