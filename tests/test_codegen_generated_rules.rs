use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};

#[test]
fn test_detect_generated_media_formats() {
    let opts = DetectOptions::default();

    // 1. FLV video
    let flv_bytes = b"FLV\x01\x05\x00\x00\x00\x09\x00\x00\x00\x00";
    let rep_flv = detect_bytes(flv_bytes, &opts).expect("detect flv");
    assert_eq!(rep_flv.identification, Identification::Format);
    assert_eq!(rep_flv.format_id.as_deref(), Some("flv"));
    assert_eq!(rep_flv.category, "video");
    assert_eq!(rep_flv.verification, Verification::Signature);
    assert!(rep_flv.evidence.iter().any(|e| e.code == "rule.flv.magic"));
    assert_eq!(compare_extension(&rep_flv, Some(OsStr::new("flv"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_flv, Some(OsStr::new("mp4"))), ExtensionStatus::Mismatch);

    // 2. AMR audio
    let amr_bytes = b"#!AMR\n\x3c\x12\x34\x56\x78";
    let rep_amr = detect_bytes(amr_bytes, &opts).expect("detect amr");
    assert_eq!(rep_amr.format_id.as_deref(), Some("amr"));
    assert_eq!(rep_amr.category, "audio");
    assert_eq!(compare_extension(&rep_amr, Some(OsStr::new("amr"))), ExtensionStatus::Match);

    // 3. Apple CAF audio
    let caf_bytes = b"caff\x00\x01\x00\x00desc\x00\x00\x00\x20";
    let rep_caf = detect_bytes(caf_bytes, &opts).expect("detect caf");
    assert_eq!(rep_caf.format_id.as_deref(), Some("caf"));
    assert_eq!(rep_caf.category, "audio");

    // 4. Dirac video
    let drc_bytes = b"BBCD\x00\x01\x00\x00";
    let rep_drc = detect_bytes(drc_bytes, &opts).expect("detect drc");
    assert_eq!(rep_drc.format_id.as_deref(), Some("drc"));
    assert_eq!(rep_drc.category, "video");

    // 5. MNG video / animation
    let mng_bytes = b"\x8aMNG\r\n\x1a\n\x00\x00\x00\x08MHDR\x00\x00\x01\x00";
    let rep_mng = detect_bytes(mng_bytes, &opts).expect("detect mng");
    assert_eq!(rep_mng.format_id.as_deref(), Some("mng"));
    assert_eq!(rep_mng.category, "video");
}

#[test]
fn test_detect_generated_image_formats() {
    let opts = DetectOptions::default();

    // 1. GIMP XCF
    let xcf_bytes = b"gimp xcf v011\x00\x00\x00\x01\x00";
    let rep_xcf = detect_bytes(xcf_bytes, &opts).expect("detect xcf");
    assert_eq!(rep_xcf.format_id.as_deref(), Some("xcf"));
    assert_eq!(rep_xcf.category, "image");
    assert_eq!(compare_extension(&rep_xcf, Some(OsStr::new("xcf"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_xcf, Some(OsStr::new("png"))), ExtensionStatus::Mismatch);

    // 2. OpenEXR
    let exr_bytes = b"\x76\x2f\x31\x01\x02\x00\x00\x00channels\x00";
    let rep_exr = detect_bytes(exr_bytes, &opts).expect("detect exr");
    assert_eq!(rep_exr.format_id.as_deref(), Some("exr"));
    assert_eq!(rep_exr.category, "image");

    // 3. FITS
    let fits_bytes = b"SIMPLE  =                    T / file conforms to FITS standard";
    let rep_fits = detect_bytes(fits_bytes, &opts).expect("detect fits");
    assert_eq!(rep_fits.format_id.as_deref(), Some("fits"));
    assert_eq!(rep_fits.category, "image");

    // 4. Apple ICNS
    let icns_bytes = b"icns\x00\x00\x01\x00";
    let rep_icns = detect_bytes(icns_bytes, &opts).expect("detect icns");
    assert_eq!(rep_icns.format_id.as_deref(), Some("icns"));

    // 5. JPEG 2000
    let jp2_bytes = b"\x00\x00\x00\x0cjP  \r\n\x87\n\x00\x00\x00\x14ftypjp2 ";
    let rep_jp2 = detect_bytes(jp2_bytes, &opts).expect("detect jp2");
    assert_eq!(rep_jp2.format_id.as_deref(), Some("jp2"));

    // 6. Netpbm PBM, PGM, PPM
    assert_eq!(detect_bytes(b"P4\n10 10\n", &opts).unwrap().format_id.as_deref(), Some("pbm"));
    assert_eq!(detect_bytes(b"P5\n10 10\n255\n", &opts).unwrap().format_id.as_deref(), Some("pgm"));
    assert_eq!(detect_bytes(b"P6\n10 10\n255\n", &opts).unwrap().format_id.as_deref(), Some("ppm"));
}

#[test]
fn test_detect_generated_documents_and_archives() {
    let opts = DetectOptions::default();

    // 1. DICOM medical image (128 bytes preamble + DICM)
    let mut dicom_bytes = vec![0u8; 128];
    dicom_bytes.extend_from_slice(b"DICM\x02\x00\x00\x00");
    let rep_dcm = detect_bytes(&dicom_bytes, &opts).expect("detect dicom");
    assert_eq!(rep_dcm.format_id.as_deref(), Some("dicom"));
    assert_eq!(rep_dcm.category, "document");
    assert_eq!(compare_extension(&rep_dcm, Some(OsStr::new("dcm"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_dcm, Some(OsStr::new("dicm"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_dcm, Some(OsStr::new("pdf"))), ExtensionStatus::Mismatch);

    // 2. PostScript
    let ps_bytes = b"%!PS-Adobe-3.0\n%%BoundingBox: 0 0 612 792";
    let rep_ps = detect_bytes(ps_bytes, &opts).expect("detect postscript");
    assert_eq!(rep_ps.format_id.as_deref(), Some("postscript"));
    assert_eq!(rep_ps.category, "document");
    assert_eq!(compare_extension(&rep_ps, Some(OsStr::new("ps"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ps, Some(OsStr::new("eps"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ps, Some(OsStr::new("ai"))), ExtensionStatus::Match);

    // 3. RTF document
    let rtf_bytes = b"{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Arial;}}Hello World!}";
    let rep_rtf = detect_bytes(rtf_bytes, &opts).expect("detect rtf");
    assert_eq!(rep_rtf.format_id.as_deref(), Some("rtf"));
    assert_eq!(rep_rtf.category, "document");

    // 4. CPIO archive
    let cpio_bytes = b"070707000000";
    let rep_cpio = detect_bytes(cpio_bytes, &opts).expect("detect cpio");
    assert_eq!(rep_cpio.format_id.as_deref(), Some("cpio"));
    assert_eq!(rep_cpio.category, "archive");

    // 5. Lzip compressed
    let lz_bytes = b"LZIP\x01\x00\x00\x00";
    let rep_lz = detect_bytes(lz_bytes, &opts).expect("detect lzip");
    assert_eq!(rep_lz.format_id.as_deref(), Some("lz"));
    assert_eq!(rep_lz.category, "archive");

    // 6. Microsoft Cabinet
    let cab_bytes = b"MSCF\x00\x00\x00\x00";
    let rep_cab = detect_bytes(cab_bytes, &opts).expect("detect cab");
    assert_eq!(rep_cab.format_id.as_deref(), Some("cab"));
    assert_eq!(rep_cab.category, "archive");
}

#[test]
fn test_generated_truncated_detection() {
    let opts = DetectOptions::default();

    // Prefix matches but shorter than required magic
    let trunc_flv = detect_bytes(b"FL", &opts).expect("detect trunc flv");
    assert_eq!(trunc_flv.identification, Identification::Unknown);
    assert!(trunc_flv.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(trunc_flv.candidates.as_deref(), Some(&["flv".to_string()][..]));

    let trunc_amr = detect_bytes(b"#!AM", &opts).expect("detect trunc amr");
    assert_eq!(trunc_amr.identification, Identification::Unknown);
    assert!(trunc_amr.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(trunc_amr.candidates.as_deref(), Some(&["amr".to_string()][..]));

    let trunc_lz = detect_bytes(b"LZ", &opts).expect("detect trunc lz");
    assert_eq!(trunc_lz.identification, Identification::Unknown);
    assert!(trunc_lz.warnings.iter().any(|w| w.code == "input.truncated"));
    assert_eq!(trunc_lz.candidates.as_deref(), Some(&["lz".to_string()][..]));
}
