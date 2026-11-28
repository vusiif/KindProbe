use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_packages() {
    let opts = DetectOptions::default();

    // 1. Debian Package (DEB) - Multi-pattern magic
    let mut deb = vec![0u8; 32];
    deb[0..8].copy_from_slice(b"!<arch>\n");
    deb[8..21].copy_from_slice(b"debian-binary");

    let rep_deb = detect_bytes(&deb, &opts).expect("detect deb");
    assert_eq!(rep_deb.identification, Identification::Format);
    assert_eq!(rep_deb.format_id.as_deref(), Some("deb"));
    assert_eq!(rep_deb.category, "package");
    assert_eq!(rep_deb.container_id.as_deref(), Some("ar"));
    assert_eq!(rep_deb.verification, Verification::Structure);
    assert_eq!(rep_deb.recommended_extension.as_deref(), Some("deb"));
    assert_eq!(rep_deb.accepted_extensions, vec!["deb", "udeb"]);
    assert!(rep_deb.evidence.iter().any(|e| e.code == "sig.deb.magic"));

    assert_eq!(
        compare_extension(&rep_deb, Some(OsStr::new("deb"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_deb, Some(OsStr::new("udeb"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_deb, Some(OsStr::new("rpm"))),
        ExtensionStatus::Mismatch
    );

    // 2. Red Hat Package (RPM)
    let rpm = [0xED, 0xAB, 0xEE, 0xDB, 0x03, 0x00, 0x00, 0x00];
    let rep_rpm = detect_bytes(&rpm, &opts).expect("detect rpm");
    assert_eq!(rep_rpm.identification, Identification::Format);
    assert_eq!(rep_rpm.format_id.as_deref(), Some("rpm"));
    assert_eq!(rep_rpm.category, "package");
    assert_eq!(rep_rpm.verification, Verification::Structure);
    assert!(rep_rpm.evidence.iter().any(|e| e.code == "sig.rpm.magic"));
    assert_eq!(
        compare_extension(&rep_rpm, Some(OsStr::new("rpm"))),
        ExtensionStatus::Match
    );

    // 3. Chrome Extension (CRX)
    let mut crx = vec![0u8; 16];
    crx[0..4].copy_from_slice(b"Cr24");
    crx[4..8].copy_from_slice(&3u32.to_le_bytes()); // version 3
    let rep_crx = detect_bytes(&crx, &opts).expect("detect crx");
    assert_eq!(rep_crx.identification, Identification::Format);
    assert_eq!(rep_crx.format_id.as_deref(), Some("crx"));
    assert_eq!(rep_crx.category, "package");
    assert_eq!(
        compare_extension(&rep_crx, Some(OsStr::new("crx"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_professional_audio() {
    let opts = DetectOptions::default();

    // 1. FLAC audio
    let mut flac = vec![0u8; 16];
    flac[0..4].copy_from_slice(b"fLaC");
    flac[4] = 0x00; // STREAMINFO block type = 0

    let rep_flac = detect_bytes(&flac, &opts).expect("detect flac");
    assert_eq!(rep_flac.identification, Identification::Format);
    assert_eq!(rep_flac.format_id.as_deref(), Some("flac"));
    assert_eq!(rep_flac.category, "audio");
    assert_eq!(rep_flac.verification, Verification::Structure);
    assert!(rep_flac.evidence.iter().any(|e| e.code == "sig.flac.magic"));
    assert_eq!(
        compare_extension(&rep_flac, Some(OsStr::new("flac"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_flac, Some(OsStr::new("ogg"))),
        ExtensionStatus::Mismatch
    );

    // 2. Ogg container
    let mut ogg = vec![0u8; 16];
    ogg[0..4].copy_from_slice(b"OggS");
    ogg[4] = 0x00; // version 0
    ogg[5] = 0x02; // header_type_flag: BOS (Beginning of Stream)

    let rep_ogg = detect_bytes(&ogg, &opts).expect("detect ogg");
    assert_eq!(rep_ogg.identification, Identification::Format);
    assert_eq!(rep_ogg.format_id.as_deref(), Some("ogg"));
    assert_eq!(rep_ogg.category, "audio");
    assert_eq!(rep_ogg.container_id.as_deref(), Some("ogg"));
    assert_eq!(
        compare_extension(&rep_ogg, Some(OsStr::new("ogg"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_ogg, Some(OsStr::new("opus"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_design_and_images() {
    let opts = DetectOptions::default();

    // 1. Adobe Photoshop (PSD)
    let mut psd = vec![0u8; 16];
    psd[0..4].copy_from_slice(b"8BPS");
    psd[4..6].copy_from_slice(&1u16.to_be_bytes()); // version = 1

    let rep_psd = detect_bytes(&psd, &opts).expect("detect psd");
    assert_eq!(rep_psd.identification, Identification::Format);
    assert_eq!(rep_psd.format_id.as_deref(), Some("psd"));
    assert_eq!(rep_psd.category, "image");
    assert_eq!(rep_psd.verification, Verification::Structure);
    assert_eq!(
        compare_extension(&rep_psd, Some(OsStr::new("psd"))),
        ExtensionStatus::Match
    );

    // 2. TIFF Little-Endian
    let mut tiff_le = vec![0u8; 16];
    tiff_le[0..4].copy_from_slice(b"II*\0");
    tiff_le[4..8].copy_from_slice(&8u32.to_le_bytes()); // IFD offset = 8

    let rep_tiff_le = detect_bytes(&tiff_le, &opts).expect("detect tiff le");
    assert_eq!(rep_tiff_le.identification, Identification::Format);
    assert_eq!(rep_tiff_le.format_id.as_deref(), Some("tiff"));
    assert_eq!(
        compare_extension(&rep_tiff_le, Some(OsStr::new("tiff"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_tiff_le, Some(OsStr::new("tif"))),
        ExtensionStatus::Match
    );

    // 3. TIFF Big-Endian
    let mut tiff_be = vec![0u8; 16];
    tiff_be[0..4].copy_from_slice(b"MM\0*");
    tiff_be[4..8].copy_from_slice(&8u32.to_be_bytes()); // IFD offset = 8

    let rep_tiff_be = detect_bytes(&tiff_be, &opts).expect("detect tiff be");
    assert_eq!(rep_tiff_be.format_id.as_deref(), Some("tiff"));

    // 4. Windows Icon (ICO)
    let mut ico = vec![0u8; 8];
    ico[0..4].copy_from_slice(&[0x00, 0x00, 0x01, 0x00]);
    ico[4..6].copy_from_slice(&1u16.to_le_bytes()); // count = 1

    let rep_ico = detect_bytes(&ico, &opts).expect("detect ico");
    assert_eq!(rep_ico.format_id.as_deref(), Some("ico"));
    assert_eq!(
        compare_extension(&rep_ico, Some(OsStr::new("ico"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_ico, Some(OsStr::new("cur"))),
        ExtensionStatus::Mismatch
    );

    // 5. Windows Cursor (CUR)
    let mut cur = vec![0u8; 8];
    cur[0..4].copy_from_slice(&[0x00, 0x00, 0x02, 0x00]);
    cur[4..6].copy_from_slice(&1u16.to_le_bytes()); // count = 1

    let rep_cur = detect_bytes(&cur, &opts).expect("detect cur");
    assert_eq!(rep_cur.format_id.as_deref(), Some("cur"));
    assert_eq!(
        compare_extension(&rep_cur, Some(OsStr::new("cur"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_declarative_truncated() {
    let opts = DetectOptions::default();

    let trunc_flac = detect_bytes(b"fL", &opts).expect("detect truncated flac");
    assert_eq!(trunc_flac.identification, Identification::Unknown);
    assert!(trunc_flac
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));

    let trunc_deb = detect_bytes(b"!<ar", &opts).expect("detect truncated deb");
    assert_eq!(trunc_deb.identification, Identification::Unknown);
    assert!(trunc_deb
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));
}

#[test]
fn test_short_rule_collision_and_length_guards() {
    let opts = DetectOptions::default();

    // E-AC-3 (bsid = 16 in header[5] >> 3) must not be shadowed by AC-3
    let eac3 = [0x0b, 0x77, 0x01, 0x00, 0x00, 0x80, 0x00, 0x00];
    let rep_eac3 = detect_bytes(&eac3, &opts).expect("detect eac3");
    assert_eq!(rep_eac3.format_id.as_deref(), Some("eac3"));
    assert_eq!(
        compare_extension(&rep_eac3, Some(OsStr::new("eac3"))),
        ExtensionStatus::Match
    );

    // Standard AC-3 (bsid = 8) stays ac3
    let ac3 = [0x0b, 0x77, 0x12, 0x34, 0x08, 0x40, 0x00, 0x00];
    let rep_ac3 = detect_bytes(&ac3, &opts).expect("detect ac3");
    assert_eq!(rep_ac3.format_id.as_deref(), Some("ac3"));

    // PK font accepts both .pk and .pkf extensions
    let pk = [0xf7, 0x59, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    let rep_pk = detect_bytes(&pk, &opts).expect("detect pk");
    assert_eq!(rep_pk.format_id.as_deref(), Some("pk"));
    assert_eq!(
        compare_extension(&rep_pk, Some(OsStr::new("pkf"))),
        ExtensionStatus::Match
    );

    // Short 2-byte prefixes report truncated/unknown instead of false-confirming
    let short_jxl = detect_bytes(b"\xff\x0a", &opts).expect("detect short jxl");
    assert_eq!(short_jxl.identification, Identification::Unknown);
    assert!(short_jxl
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));
}
