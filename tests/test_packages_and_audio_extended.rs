use std::ffi::OsStr;
use std::io::{Cursor, Write};
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, detect_reader, ExtensionStatus, Identification, Verification};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn test_detect_apk_and_jar_in_zip() {
    let opts = DetectOptions::default();

    // 1. Android APK in ZIP container
    let mut apk_buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut apk_buf);
        let zip_opts = SimpleFileOptions::default();
        zip.start_file("AndroidManifest.xml", zip_opts).unwrap();
        zip.write_all(b"fake android manifest binary xml").unwrap();
        zip.start_file("classes.dex", zip_opts).unwrap();
        zip.write_all(b"fake dex bytecode").unwrap();
        zip.finish().unwrap();
    }
    apk_buf.set_position(0);

    let rep_apk = detect_reader(&mut apk_buf, &opts).expect("detect apk");
    assert_eq!(rep_apk.identification, Identification::Format);
    assert_eq!(rep_apk.format_id.as_deref(), Some("apk"));
    assert_eq!(rep_apk.category, "package");
    assert_eq!(rep_apk.container_id.as_deref(), Some("zip"));
    assert_eq!(rep_apk.verification, Verification::Structure);
    assert!(rep_apk.evidence.iter().any(|e| e.code == "zip.apk.manifest"));
    assert!(rep_apk.evidence.iter().any(|e| e.code == "zip.apk.dex"));
    assert_eq!(compare_extension(&rep_apk, Some(OsStr::new("apk"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_apk, Some(OsStr::new("xapk"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_apk, Some(OsStr::new("jar"))), ExtensionStatus::Mismatch);

    // 2. Java JAR in ZIP container
    let mut jar_buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut jar_buf);
        let zip_opts = SimpleFileOptions::default();
        zip.start_file("META-INF/MANIFEST.MF", zip_opts).unwrap();
        zip.write_all(b"Manifest-Version: 1.0\nCreated-By: 1.8.0\n").unwrap();
        zip.finish().unwrap();
    }
    jar_buf.set_position(0);

    let rep_jar = detect_reader(&mut jar_buf, &opts).expect("detect jar");
    assert_eq!(rep_jar.identification, Identification::Format);
    assert_eq!(rep_jar.format_id.as_deref(), Some("jar"));
    assert_eq!(rep_jar.category, "package");
    assert_eq!(rep_jar.container_id.as_deref(), Some("zip"));
    assert_eq!(compare_extension(&rep_jar, Some(OsStr::new("jar"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_jar, Some(OsStr::new("war"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_ipa_in_zip() {
    let opts = DetectOptions::default();

    let mut ipa_buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut ipa_buf);
        let zip_opts = SimpleFileOptions::default();
        zip.start_file("Payload/KindProbe.app/Info.plist", zip_opts).unwrap();
        zip.write_all(b"<plist version=\"1.0\"></plist>").unwrap();
        zip.finish().unwrap();
    }
    ipa_buf.set_position(0);

    let rep_ipa = detect_reader(&mut ipa_buf, &opts).expect("detect ipa");
    assert_eq!(rep_ipa.identification, Identification::Format);
    assert_eq!(rep_ipa.format_id.as_deref(), Some("ipa"));
    assert_eq!(rep_ipa.category, "package");
    assert_eq!(rep_ipa.container_id.as_deref(), Some("zip"));
    assert!(rep_ipa.evidence.iter().any(|e| e.code == "zip.ipa.payload"));
    assert_eq!(compare_extension(&rep_ipa, Some(OsStr::new("ipa"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ipa, Some(OsStr::new("apk"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_aac_audio() {
    let opts = DetectOptions::default();

    let aac_bytes = [0xFF, 0xF1, 0x50, 0x80];
    let rep = detect_bytes(&aac_bytes, &opts).expect("detect aac");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("aac"));
    assert_eq!(rep.category, "audio");
    assert_eq!(rep.verification, Verification::Structure);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.aac.magic"));
    assert_eq!(compare_extension(&rep, Some(OsStr::new("aac"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("mp3"))), ExtensionStatus::Mismatch);
}

#[test]
fn test_detect_aiff_and_midi() {
    let opts = DetectOptions::default();

    // 1. AIFF Audio ("FORM" .... "AIFF")
    let mut aiff = vec![0u8; 16];
    aiff[0..4].copy_from_slice(b"FORM");
    aiff[4..8].copy_from_slice(&16u32.to_be_bytes());
    aiff[8..12].copy_from_slice(b"AIFF");

    let rep_aiff = detect_bytes(&aiff, &opts).expect("detect aiff");
    assert_eq!(rep_aiff.identification, Identification::Format);
    assert_eq!(rep_aiff.format_id.as_deref(), Some("aiff"));
    assert_eq!(rep_aiff.category, "audio");
    assert_eq!(compare_extension(&rep_aiff, Some(OsStr::new("aif"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_aiff, Some(OsStr::new("aiff"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_aiff, Some(OsStr::new("wav"))), ExtensionStatus::Mismatch);

    // 2. MIDI Audio ("MThd" + length 6)
    let mut midi = vec![0u8; 16];
    midi[0..4].copy_from_slice(b"MThd");
    midi[4..8].copy_from_slice(&6u32.to_be_bytes()); // length = 6

    let rep_mid = detect_bytes(&midi, &opts).expect("detect midi");
    assert_eq!(rep_mid.identification, Identification::Format);
    assert_eq!(rep_mid.format_id.as_deref(), Some("mid"));
    assert_eq!(rep_mid.category, "audio");
    assert_eq!(compare_extension(&rep_mid, Some(OsStr::new("mid"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_mid, Some(OsStr::new("midi"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_chm_document() {
    let opts = DetectOptions::default();

    // CHM ("ITSF" + version 3)
    let mut chm = vec![0u8; 16];
    chm[0..4].copy_from_slice(b"ITSF");
    chm[4..8].copy_from_slice(&3u32.to_le_bytes()); // version = 3

    let rep_chm = detect_bytes(&chm, &opts).expect("detect chm");
    assert_eq!(rep_chm.identification, Identification::Format);
    assert_eq!(rep_chm.format_id.as_deref(), Some("chm"));
    assert_eq!(rep_chm.category, "document");
    assert_eq!(rep_chm.verification, Verification::Structure);
    assert!(rep_chm.evidence.iter().any(|e| e.code == "sig.chm.magic"));
    assert_eq!(compare_extension(&rep_chm, Some(OsStr::new("chm"))), ExtensionStatus::Match);
}
