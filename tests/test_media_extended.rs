use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};

#[test]
fn test_detect_mp4_ftyp_success() {
    let mut data = Vec::new();
    // ftyp box: size = 20 (0x14), type = b"ftyp", major_brand = b"isom", minor = 0x00000200, compatible = b"mp41"
    data.extend_from_slice(&(20u32.to_be_bytes()));
    data.extend_from_slice(b"ftyp");
    data.extend_from_slice(b"isom");
    data.extend_from_slice(&(0x200u32.to_be_bytes()));
    data.extend_from_slice(b"mp41");

    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect mp4");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("mp4"));
    assert_eq!(rep.category, "video");
    assert_eq!(rep.container_id.as_deref(), Some("isobmff"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("mp4"));
    assert_eq!(rep.accepted_extensions, vec!["mp4", "m4v", "m4a"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.mp4.ftyp"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.mp4.brand"));

    // Extension comparison
    assert_eq!(compare_extension(&rep, Some(OsStr::new("mp4"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("m4v"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pdf"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_mov_quicktime_brand() {
    let mut data = Vec::new();
    // ftyp box with major_brand "qt  "
    data.extend_from_slice(&(20u32.to_be_bytes()));
    data.extend_from_slice(b"ftyp");
    data.extend_from_slice(b"qt  ");
    data.extend_from_slice(&(0x200u32.to_be_bytes()));
    data.extend_from_slice(b"qt  ");

    let rep = detect_bytes(&data, &DetectOptions::default()).expect("detect mov");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("mov"));
    assert_eq!(rep.category, "video");
    assert_eq!(rep.container_id.as_deref(), Some("isobmff"));
    assert_eq!(rep.recommended_extension.as_deref(), Some("mov"));
    assert_eq!(rep.accepted_extensions, vec!["mov", "qt"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.mov.brand"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("mov"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("qt"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("mp4"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_webm_and_mkv() {
    // 1. WebM: EBML header + DocType "webm"
    let mut webm = Vec::new();
    webm.extend_from_slice(&[0x1A, 0x45, 0xDF, 0xA3]); // EBML ID
    webm.extend_from_slice(&[0x9F]); // EBML Header size = 31
    // DocType element: ID 0x42 0x82, length VINT 0x84 (4 bytes), string "webm"
    webm.extend_from_slice(&[0x42, 0x82, 0x84]);
    webm.extend_from_slice(b"webm");

    let rep_webm = detect_bytes(&webm, &DetectOptions::default()).expect("detect webm");
    assert_eq!(rep_webm.identification, Identification::Format);
    assert_eq!(rep_webm.format_id.as_deref(), Some("webm"));
    assert_eq!(rep_webm.category, "video");
    assert_eq!(rep_webm.container_id.as_deref(), Some("matroska"));
    assert_eq!(rep_webm.verification, Verification::Structure);
    assert_eq!(rep_webm.recommended_extension.as_deref(), Some("webm"));
    assert!(rep_webm.evidence.iter().any(|e| e.code == "sig.ebml.doctype_webm"));

    // 2. MKV: EBML header + DocType "matroska"
    let mut mkv = Vec::new();
    mkv.extend_from_slice(&[0x1A, 0x45, 0xDF, 0xA3]);
    mkv.extend_from_slice(&[0x9F]);
    // DocType element: ID 0x42 0x82, length VINT 0x88 (8 bytes), string "matroska"
    mkv.extend_from_slice(&[0x42, 0x82, 0x88]);
    mkv.extend_from_slice(b"matroska");

    let rep_mkv = detect_bytes(&mkv, &DetectOptions::default()).expect("detect mkv");
    assert_eq!(rep_mkv.identification, Identification::Format);
    assert_eq!(rep_mkv.format_id.as_deref(), Some("mkv"));
    assert_eq!(rep_mkv.category, "video");
    assert_eq!(rep_mkv.container_id.as_deref(), Some("matroska"));
    assert_eq!(rep_mkv.verification, Verification::Structure);
    assert_eq!(rep_mkv.recommended_extension.as_deref(), Some("mkv"));
    assert!(rep_mkv.evidence.iter().any(|e| e.code == "sig.ebml.doctype_mkv"));
}

#[test]
fn test_detect_wav_riff_success() {
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36u32.to_le_bytes())); // file size - 8
    wav.extend_from_slice(b"WAVE");
    // fmt subchunk: "fmt " + size 16 (0x10) + 16 bytes format data
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&(16u32.to_le_bytes()));
    wav.extend_from_slice(&[1, 0]); // AudioFormat = 1 (PCM)
    wav.extend_from_slice(&[2, 0]); // NumChannels = 2
    wav.extend_from_slice(&44100u32.to_le_bytes()); // SampleRate
    wav.extend_from_slice(&176400u32.to_le_bytes()); // ByteRate
    wav.extend_from_slice(&[4, 0]); // BlockAlign = 4
    wav.extend_from_slice(&[16, 0]); // BitsPerSample = 16

    let rep = detect_bytes(&wav, &DetectOptions::default()).expect("detect wav");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("wav"));
    assert_eq!(rep.category, "audio");
    assert_eq!(rep.container_id.as_deref(), Some("riff"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("wav"));
    assert_eq!(rep.accepted_extensions, vec!["wav", "wave"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.riff.header"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.wav.fmt_chunk"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("wav"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("wave"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("mp3"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_webp_riff_success() {
    let mut webp = Vec::new();
    webp.extend_from_slice(b"RIFF");
    webp.extend_from_slice(&(20u32.to_le_bytes()));
    webp.extend_from_slice(b"WEBP");
    webp.extend_from_slice(b"VP8 ");
    webp.extend_from_slice(&(8u32.to_le_bytes()));
    webp.extend_from_slice(&[0u8; 8]);

    let rep = detect_bytes(&webp, &DetectOptions::default()).expect("detect webp");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("webp"));
    assert_eq!(rep.category, "image");
    assert_eq!(rep.container_id.as_deref(), Some("riff"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("webp"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.webp.vp8"));
}

#[test]
fn test_detect_mp3_id3v2_and_raw_sync() {
    // 1. MP3 with ID3v2 tag and frame sync
    let mut id3_mp3 = Vec::new();
    id3_mp3.extend_from_slice(b"ID3");
    id3_mp3.push(3); // major 2.3
    id3_mp3.push(0); // revision
    id3_mp3.push(0); // flags
    id3_mp3.extend_from_slice(&[0, 0, 0, 10]); // synchsafe size = 10
    id3_mp3.extend_from_slice(&[0u8; 10]); // tag data
    // MPEG Audio Layer III frame sync: 0xFF 0xFB (11111111 11111011)
    id3_mp3.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]);

    let rep = detect_bytes(&id3_mp3, &DetectOptions::default()).expect("detect id3 mp3");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("mp3"));
    assert_eq!(rep.category, "audio");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("mp3"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.id3.tag"));
    assert!(rep.evidence.iter().any(|e| e.code == "sig.mp3.frame_sync"));

    // 2. Raw MP3 starting directly with frame sync
    let raw_mp3 = vec![0xFF, 0xFB, 0x90, 0x64, 0x00, 0x00];
    let rep_raw = detect_bytes(&raw_mp3, &DetectOptions::default()).expect("detect raw mp3");
    assert_eq!(rep_raw.identification, Identification::Format);
    assert_eq!(rep_raw.format_id.as_deref(), Some("mp3"));
    assert_eq!(rep_raw.verification, Verification::Signature);
}

#[test]
fn test_detect_media_truncated() {
    let opts = DetectOptions::default();

    // 1. Truncated ftyp
    let rep_mp4 = detect_bytes(b"\0\0\0\x14ft", &opts).expect("truncated mp4");
    assert_eq!(rep_mp4.identification, Identification::Unknown);
    assert_eq!(rep_mp4.candidates.as_deref(), Some(&["mp4".to_string()][..]));

    // 2. Truncated EBML
    let rep_ebml = detect_bytes(&[0x1A, 0x45], &opts).expect("truncated ebml");
    assert_eq!(rep_ebml.identification, Identification::Unknown);
    assert_eq!(rep_ebml.candidates.as_deref(), Some(&["webm".to_string()][..]));

    // 3. Truncated RIFF
    let rep_riff = detect_bytes(b"RIFF\x10\0", &opts).expect("truncated riff");
    assert_eq!(rep_riff.identification, Identification::Unknown);
    assert_eq!(rep_riff.candidates.as_deref(), Some(&["wav".to_string()][..]));

    // 4. Truncated ID3
    let rep_id3 = detect_bytes(b"ID", &opts).expect("truncated id3");
    assert_eq!(rep_id3.identification, Identification::Unknown);
    assert_eq!(rep_id3.candidates.as_deref(), Some(&["mp3".to_string()][..]));
}
