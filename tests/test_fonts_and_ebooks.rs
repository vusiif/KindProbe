use std::ffi::OsStr;
use std::io::{Cursor, Write};
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_detect_ttf_and_otf() {
    let opts = DetectOptions::default();

    // 1. TrueType font (TTF)
    let mut ttf = vec![0u8; 32];
    ttf[0..4].copy_from_slice(&[0x00, 0x01, 0x00, 0x00]);
    ttf[4..6].copy_from_slice(&12u16.to_be_bytes()); // numTables = 12

    let rep_ttf = detect_bytes(&ttf, &opts).expect("detect ttf");
    assert_eq!(rep_ttf.identification, Identification::Format);
    assert_eq!(rep_ttf.format_id.as_deref(), Some("ttf"));
    assert_eq!(rep_ttf.category, "font");
    assert_eq!(rep_ttf.verification, Verification::Structure);
    assert_eq!(rep_ttf.inspection, Inspection::Complete);
    assert_eq!(rep_ttf.recommended_extension.as_deref(), Some("ttf"));
    assert_eq!(rep_ttf.accepted_extensions, vec!["ttf"]);
    assert!(rep_ttf.evidence.iter().any(|e| e.code == "sig.font.ttf_magic"));
    assert!(rep_ttf.evidence.iter().any(|e| e.code == "sig.font.num_tables"));

    assert_eq!(compare_extension(&rep_ttf, Some(OsStr::new("ttf"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ttf, Some(OsStr::new("otf"))), ExtensionStatus::Mismatch);

    // 2. OpenType font (OTF)
    let mut otf = vec![0u8; 32];
    otf[0..4].copy_from_slice(b"OTTO");
    otf[4..6].copy_from_slice(&15u16.to_be_bytes()); // numTables = 15

    let rep_otf = detect_bytes(&otf, &opts).expect("detect otf");
    assert_eq!(rep_otf.identification, Identification::Format);
    assert_eq!(rep_otf.format_id.as_deref(), Some("otf"));
    assert_eq!(rep_otf.category, "font");
    assert_eq!(rep_otf.verification, Verification::Structure);
    assert_eq!(rep_otf.recommended_extension.as_deref(), Some("otf"));
    assert!(rep_otf.evidence.iter().any(|e| e.code == "sig.font.otf_magic"));
    assert!(rep_otf.evidence.iter().any(|e| e.code == "sig.font.num_tables"));

    assert_eq!(compare_extension(&rep_otf, Some(OsStr::new("otf"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_otf, Some(OsStr::new("ttf"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_woff_and_woff2() {
    let opts = DetectOptions::default();

    // 1. WOFF 1.0
    let mut woff = vec![0u8; 44];
    woff[0..4].copy_from_slice(b"wOFF");
    woff[4..8].copy_from_slice(b"OTTO"); // flavor
    woff[8..12].copy_from_slice(&44u32.to_be_bytes()); // length
    woff[12..14].copy_from_slice(&8u16.to_be_bytes()); // numTables = 8

    let rep_woff = detect_bytes(&woff, &opts).expect("detect woff");
    assert_eq!(rep_woff.identification, Identification::Format);
    assert_eq!(rep_woff.format_id.as_deref(), Some("woff"));
    assert_eq!(rep_woff.category, "font");
    assert_eq!(rep_woff.verification, Verification::Structure);
    assert_eq!(rep_woff.recommended_extension.as_deref(), Some("woff"));
    assert!(rep_woff.evidence.iter().any(|e| e.code == "sig.font.woff_magic"));
    assert!(rep_woff.evidence.iter().any(|e| e.code == "sig.font.woff_header"));

    assert_eq!(compare_extension(&rep_woff, Some(OsStr::new("woff"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_woff, Some(OsStr::new("woff2"))), ExtensionStatus::Mismatch);

    // 2. WOFF 2.0
    let mut woff2 = vec![0u8; 48];
    woff2[0..4].copy_from_slice(b"wOF2");
    woff2[4..8].copy_from_slice(&[0x00, 0x01, 0x00, 0x00]); // TrueType flavor
    woff2[8..12].copy_from_slice(&48u32.to_be_bytes());
    woff2[12..14].copy_from_slice(&10u16.to_be_bytes()); // numTables = 10

    let rep_woff2 = detect_bytes(&woff2, &opts).expect("detect woff2");
    assert_eq!(rep_woff2.identification, Identification::Format);
    assert_eq!(rep_woff2.format_id.as_deref(), Some("woff2"));
    assert_eq!(rep_woff2.category, "font");
    assert_eq!(rep_woff2.verification, Verification::Structure);
    assert_eq!(rep_woff2.recommended_extension.as_deref(), Some("woff2"));
    assert!(rep_woff2.evidence.iter().any(|e| e.code == "sig.font.woff2_magic"));
    assert!(rep_woff2.evidence.iter().any(|e| e.code == "sig.font.woff2_header"));
}

#[test]
fn test_detect_epub_in_zip() {
    let opts = DetectOptions::default();
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);

    let zip_opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mimetype", zip_opts).expect("start mimetype");
    zip.write_all(b"application/epub+zip").expect("write mimetype");

    zip.start_file("META-INF/container.xml", zip_opts).expect("start container");
    zip.write_all(b"<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\"><rootfiles/></container>").expect("write container");

    let cursor = zip.finish().expect("finish zip");
    let data = cursor.into_inner();

    let rep = detect_bytes(&data, &opts).expect("detect epub");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("epub"));
    assert_eq!(rep.category, "document");
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("epub"));
    assert_eq!(rep.accepted_extensions, vec!["epub"]);
    assert!(rep.evidence.iter().any(|e| e.code == "epub.mimetype"));
    assert!(rep.evidence.iter().any(|e| e.code == "epub.container"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("epub"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("zip"))), ExtensionStatus::Mismatch);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pdf"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_odt_in_zip() {
    let opts = DetectOptions::default();
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);

    let zip_opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mimetype", zip_opts).expect("start mimetype");
    zip.write_all(b"application/vnd.oasis.opendocument.text").expect("write mimetype");

    let cursor = zip.finish().expect("finish zip");
    let data = cursor.into_inner();

    let rep = detect_bytes(&data, &opts).expect("detect odt");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("odt"));
    assert_eq!(rep.category, "document");
    assert_eq!(rep.container_id.as_deref(), Some("zip"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("odt"));
    assert_eq!(rep.accepted_extensions, vec!["odt", "ott"]);
    assert!(rep.evidence.iter().any(|e| e.code == "odf.mimetype"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("odt"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("docx"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_mobi_ebook() {
    let opts = DetectOptions::default();

    let mut mobi = vec![0u8; 128];
    // Database name (0..32)
    mobi[0..12].copy_from_slice(b"Sample Book\0");
    // Type and creator at 60..68: BOOKMOBI
    mobi[60..68].copy_from_slice(b"BOOKMOBI");

    let rep = detect_bytes(&mobi, &opts).expect("detect mobi");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("mobi"));
    assert_eq!(rep.category, "document");
    assert_eq!(rep.container_id.as_deref(), Some("palmdoc"));
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("mobi"));
    assert_eq!(rep.accepted_extensions, vec!["mobi", "azw", "prc"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.mobi.header"));

    assert_eq!(compare_extension(&rep, Some(OsStr::new("mobi"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("azw"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("epub"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_fonts_and_ebooks_truncated() {
    let opts = DetectOptions::default();

    // 1. Truncated TTF
    let rep_ttf = detect_bytes(&[0x00, 0x01], &opts).expect("truncated ttf");
    assert_eq!(rep_ttf.identification, Identification::Unknown);
    assert_eq!(rep_ttf.candidates.as_deref(), Some(&["ttf".to_string()][..]));

    // 2. Truncated WOFF
    let rep_woff = detect_bytes(b"wO", &opts).expect("truncated woff");
    assert_eq!(rep_woff.identification, Identification::Unknown);
    assert_eq!(rep_woff.candidates.as_deref(), Some(&["woff".to_string()][..]));

    // 3. Truncated MOBI
    let mut mobi_short = vec![0u8; 64];
    mobi_short[60..64].copy_from_slice(b"BOOK");
    let rep_mobi = detect_bytes(&mobi_short, &opts).expect("truncated mobi");
    assert_eq!(rep_mobi.identification, Identification::Unknown);
    assert_eq!(rep_mobi.candidates.as_deref(), Some(&["mobi".to_string()][..]));
}
