use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};
use std::ffi::OsStr;

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
    assert_eq!(
        compare_extension(&rep_flv, Some(OsStr::new("flv"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_flv, Some(OsStr::new("mp4"))),
        ExtensionStatus::Mismatch
    );

    // 2. AMR audio
    let amr_bytes = b"#!AMR\n\x3c\x12\x34\x56\x78";
    let rep_amr = detect_bytes(amr_bytes, &opts).expect("detect amr");
    assert_eq!(rep_amr.format_id.as_deref(), Some("amr"));
    assert_eq!(rep_amr.category, "audio");
    assert_eq!(
        compare_extension(&rep_amr, Some(OsStr::new("amr"))),
        ExtensionStatus::Match
    );

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

    // 6. IVF raw video
    let ivf_bytes = b"DKIF\x00\x00\x20\x00VP80\x40\x01\xf0\x00";
    let rep_ivf = detect_bytes(ivf_bytes, &opts).expect("detect ivf");
    assert_eq!(rep_ivf.format_id.as_deref(), Some("ivf"));
    assert_eq!(rep_ivf.category, "video");
    assert_eq!(
        compare_extension(&rep_ivf, Some(OsStr::new("ivf"))),
        ExtensionStatus::Match
    );

    // 7. Bink Video (bik / bk2)
    let bik_bytes = b"BIKi\x00\x01\x00\x00";
    let rep_bik = detect_bytes(bik_bytes, &opts).expect("detect bik");
    assert_eq!(rep_bik.format_id.as_deref(), Some("bik"));
    assert_eq!(rep_bik.category, "video");
    assert_eq!(
        compare_extension(&rep_bik, Some(OsStr::new("bik"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_bik, Some(OsStr::new("bk2"))),
        ExtensionStatus::Match
    );

    // 8. Smacker Video (smk)
    let smk_bytes = b"SMK2\x00\x01\x00\x00";
    let rep_smk = detect_bytes(smk_bytes, &opts).expect("detect smk");
    assert_eq!(rep_smk.format_id.as_deref(), Some("smk"));
    assert_eq!(rep_smk.category, "video");
    assert_eq!(
        compare_extension(&rep_smk, Some(OsStr::new("smk"))),
        ExtensionStatus::Match
    );

    // 9. NUT Open Container (nut)
    let nut_bytes = b"NUTopencontainer\0\x00\x00\x00";
    let rep_nut = detect_bytes(nut_bytes, &opts).expect("detect nut");
    assert_eq!(rep_nut.format_id.as_deref(), Some("nut"));
    assert_eq!(rep_nut.category, "video");
    assert_eq!(
        compare_extension(&rep_nut, Some(OsStr::new("nut"))),
        ExtensionStatus::Match
    );

    // 10. Tom's Lossless Audio (tak)
    let tak_bytes = b"tBaK\x00\x00\x00\x00";
    let rep_tak = detect_bytes(tak_bytes, &opts).expect("detect tak");
    assert_eq!(rep_tak.format_id.as_deref(), Some("tak"));
    assert_eq!(rep_tak.category, "audio");
    assert_eq!(
        compare_extension(&rep_tak, Some(OsStr::new("tak"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_generated_image_formats() {
    let opts = DetectOptions::default();

    // 1. GIMP XCF
    let xcf_bytes = b"gimp xcf v011\x00\x00\x00\x01\x00";
    let rep_xcf = detect_bytes(xcf_bytes, &opts).expect("detect xcf");
    assert_eq!(rep_xcf.format_id.as_deref(), Some("xcf"));
    assert_eq!(rep_xcf.category, "image");
    assert_eq!(
        compare_extension(&rep_xcf, Some(OsStr::new("xcf"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_xcf, Some(OsStr::new("png"))),
        ExtensionStatus::Mismatch
    );

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
    assert_eq!(
        detect_bytes(b"P4\n10 10\n", &opts)
            .unwrap()
            .format_id
            .as_deref(),
        Some("pbm")
    );
    assert_eq!(
        detect_bytes(b"P5\n10 10\n255\n", &opts)
            .unwrap()
            .format_id
            .as_deref(),
        Some("pgm")
    );
    assert_eq!(
        detect_bytes(b"P6\n10 10\n255\n", &opts)
            .unwrap()
            .format_id
            .as_deref(),
        Some("ppm")
    );

    // 7. SMPTE Digital Picture Exchange (DPX)
    let dpx_bytes = b"SDPX\x00\x00\x08\x00";
    let rep_dpx = detect_bytes(dpx_bytes, &opts).expect("detect dpx");
    assert_eq!(rep_dpx.format_id.as_deref(), Some("dpx"));
    assert_eq!(rep_dpx.category, "image");
    assert_eq!(
        compare_extension(&rep_dpx, Some(OsStr::new("dpx"))),
        ExtensionStatus::Match
    );

    // 8. Kodak Cineon (cin)
    let cin_bytes = b"\x80\x2a\x5f\xd7\x00\x00\x00\x00";
    let rep_cin = detect_bytes(cin_bytes, &opts).expect("detect cin");
    assert_eq!(rep_cin.format_id.as_deref(), Some("cin"));
    assert_eq!(rep_cin.category, "image");
    assert_eq!(
        compare_extension(&rep_cin, Some(OsStr::new("cin"))),
        ExtensionStatus::Match
    );

    // 9. Amiga IFF Interleaved Bitmap (ilbm / lbm)
    let ilbm_bytes = b"FORM\x00\x00\x01\x00ILBM\x00\x00\x00\x00";
    let rep_ilbm = detect_bytes(ilbm_bytes, &opts).expect("detect ilbm");
    assert_eq!(rep_ilbm.format_id.as_deref(), Some("ilbm"));
    assert_eq!(rep_ilbm.category, "image");
    assert_eq!(
        compare_extension(&rep_ilbm, Some(OsStr::new("ilbm"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_ilbm, Some(OsStr::new("lbm"))),
        ExtensionStatus::Match
    );

    // 10. Computer Graphics Metafile (cgm)
    let cgm_bytes = b"BEGMF01";
    let rep_cgm = detect_bytes(cgm_bytes, &opts).expect("detect cgm");
    assert_eq!(rep_cgm.format_id.as_deref(), Some("cgm"));
    assert_eq!(rep_cgm.category, "image");
    assert_eq!(
        compare_extension(&rep_cgm, Some(OsStr::new("cgm"))),
        ExtensionStatus::Match
    );
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
    assert_eq!(
        compare_extension(&rep_dcm, Some(OsStr::new("dcm"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_dcm, Some(OsStr::new("dicm"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_dcm, Some(OsStr::new("pdf"))),
        ExtensionStatus::Mismatch
    );

    // 2. PostScript
    let ps_bytes = b"%!PS-Adobe-3.0\n%%BoundingBox: 0 0 612 792";
    let rep_ps = detect_bytes(ps_bytes, &opts).expect("detect postscript");
    assert_eq!(rep_ps.format_id.as_deref(), Some("postscript"));
    assert_eq!(rep_ps.category, "document");
    assert_eq!(
        compare_extension(&rep_ps, Some(OsStr::new("ps"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_ps, Some(OsStr::new("eps"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_ps, Some(OsStr::new("ai"))),
        ExtensionStatus::Match
    );

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

    // 7. Roxio Toast Disc Image (toast)
    let toast_bytes = b"ER\x02\x00\x00\x00\x00\x00";
    let rep_toast = detect_bytes(toast_bytes, &opts).expect("detect toast");
    assert_eq!(rep_toast.format_id.as_deref(), Some("toast"));
    assert_eq!(rep_toast.category, "disk");
    assert_eq!(
        compare_extension(&rep_toast, Some(OsStr::new("toast"))),
        ExtensionStatus::Match
    );

    // 8. PostScript Font ASCII (pfa)
    let pfa_bytes = b"%!PS-AdobeFont-1.0: TestFont 001.001";
    let rep_pfa = detect_bytes(pfa_bytes, &opts).expect("detect pfa");
    assert_eq!(rep_pfa.format_id.as_deref(), Some("pfa"));
    assert_eq!(rep_pfa.category, "font");
    assert_eq!(
        compare_extension(&rep_pfa, Some(OsStr::new("pfa"))),
        ExtensionStatus::Match
    );

    // 9. GNU gettext message catalog (mo / gmo)
    let mo_bytes = b"\xde\x12\x04\x95\x00\x00\x00\x00";
    let rep_mo = detect_bytes(mo_bytes, &opts).expect("detect mo");
    assert_eq!(rep_mo.format_id.as_deref(), Some("mo"));
    assert_eq!(rep_mo.category, "translation");
    assert_eq!(
        compare_extension(&rep_mo, Some(OsStr::new("mo"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_mo, Some(OsStr::new("gmo"))),
        ExtensionStatus::Match
    );

    // 10. SQL database script
    let sql_bytes = b"CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255));\n";
    let rep_sql = detect_bytes(sql_bytes, &opts).expect("detect sql");
    assert_eq!(rep_sql.format_id.as_deref(), Some("sql"));
    assert_eq!(rep_sql.category, "database");
    assert_eq!(
        compare_extension(&rep_sql, Some(OsStr::new("sql"))),
        ExtensionStatus::Match
    );

    // 11. Geomview Object File Format 3D mesh (off)
    let off_bytes = b"OFF\n8 6 12\n";
    let rep_off = detect_bytes(off_bytes, &opts).expect("detect off");
    assert_eq!(rep_off.format_id.as_deref(), Some("off"));
    assert_eq!(rep_off.category, "model");
    assert_eq!(
        compare_extension(&rep_off, Some(OsStr::new("off"))),
        ExtensionStatus::Match
    );

    // 12. FictionBook 2 eBook (fb2)
    let fb2_bytes = b"<FictionBook xmlns=\"http://www.gribuser.ru/xml/fictionbook/2.0\"><description></description></FictionBook>";
    let rep_fb2 = detect_bytes(fb2_bytes, &opts).expect("detect fb2");
    assert_eq!(rep_fb2.format_id.as_deref(), Some("fb2"));
    assert_eq!(rep_fb2.category, "document");
    assert_eq!(
        compare_extension(&rep_fb2, Some(OsStr::new("fb2"))),
        ExtensionStatus::Match
    );

    // 13. PHP Source Script (php / phtml)
    let php_bytes = b"<?php echo 'Hello KindProbe'; ?>";
    let rep_php = detect_bytes(php_bytes, &opts).expect("detect php");
    assert_eq!(rep_php.format_id.as_deref(), Some("php"));
    assert_eq!(rep_php.category, "source");
    assert_eq!(
        compare_extension(&rep_php, Some(OsStr::new("php"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_php, Some(OsStr::new("phtml"))),
        ExtensionStatus::Match
    );

    // 14. Protocol Buffers (proto)
    let proto_bytes = b"syntax = \"proto3\";\npackage kindprobe;\nmessage Detection {}\n";
    let rep_proto = detect_bytes(proto_bytes, &opts).expect("detect proto");
    assert_eq!(rep_proto.format_id.as_deref(), Some("proto"));
    assert_eq!(rep_proto.category, "schema");
    assert_eq!(
        compare_extension(&rep_proto, Some(OsStr::new("proto"))),
        ExtensionStatus::Match
    );

    // 15. Ren'Py Game Archive (rpa)
    let rpa_bytes = b"RPA-3.0 01234567 89abcdef\n";
    let rep_rpa = detect_bytes(rpa_bytes, &opts).expect("detect rpa");
    assert_eq!(rep_rpa.format_id.as_deref(), Some("rpa"));
    assert_eq!(rep_rpa.category, "archive");
    assert_eq!(
        compare_extension(&rep_rpa, Some(OsStr::new("rpa"))),
        ExtensionStatus::Match
    );

    // 16. Minisign signature (minisig)
    let minisig_bytes = b"untrusted comment: signature from minisign secret key\nRWQ123456789";
    let rep_minisig = detect_bytes(minisig_bytes, &opts).expect("detect minisig");
    assert_eq!(rep_minisig.format_id.as_deref(), Some("minisig"));
    assert_eq!(rep_minisig.category, "security");
    assert_eq!(
        compare_extension(&rep_minisig, Some(OsStr::new("minisig"))),
        ExtensionStatus::Match
    );

    // 17. Server Normal Format Font (snf)
    let snf_bytes = b"\x04\x10\x00\x00\x00\x00\x00\x00";
    let rep_snf = detect_bytes(snf_bytes, &opts).expect("detect snf");
    assert_eq!(rep_snf.format_id.as_deref(), Some("snf"));
    assert_eq!(rep_snf.category, "font");
    assert_eq!(
        compare_extension(&rep_snf, Some(OsStr::new("snf"))),
        ExtensionStatus::Match
    );

    // 18. UPS Patch (ups)
    let ups_bytes = b"UPS1\x12\x34\x56\x78";
    let rep_ups = detect_bytes(ups_bytes, &opts).expect("detect ups");
    assert_eq!(rep_ups.format_id.as_deref(), Some("ups"));
    assert_eq!(rep_ups.category, "rom");
    assert_eq!(
        compare_extension(&rep_ups, Some(OsStr::new("ups"))),
        ExtensionStatus::Match
    );

    // 19. Crystallographic Information File (cif)
    let cif_bytes = b"data_global\n_cell_length_a 10.50\n_cell_length_b 10.50\n";
    let rep_cif = detect_bytes(cif_bytes, &opts).expect("detect cif");
    assert_eq!(rep_cif.format_id.as_deref(), Some("cif"));
    assert_eq!(rep_cif.category, "scientific");
    assert_eq!(
        compare_extension(&rep_cif, Some(OsStr::new("cif"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_generated_truncated_detection() {
    let opts = DetectOptions::default();

    // Prefix matches but shorter than required magic
    let trunc_flv = detect_bytes(b"FL", &opts).expect("detect trunc flv");
    assert_eq!(trunc_flv.identification, Identification::Unknown);
    assert!(trunc_flv
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));
    assert_eq!(
        trunc_flv.candidates.as_deref(),
        Some(&["flv".to_string()][..])
    );

    let trunc_amr = detect_bytes(b"#!AM", &opts).expect("detect trunc amr");
    assert_eq!(trunc_amr.identification, Identification::Unknown);
    assert!(trunc_amr
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));
    assert_eq!(
        trunc_amr.candidates.as_deref(),
        Some(&["amr".to_string()][..])
    );

    let trunc_lz = detect_bytes(b"LZ", &opts).expect("detect trunc lz");
    assert_eq!(trunc_lz.identification, Identification::Unknown);
    assert!(trunc_lz
        .warnings
        .iter()
        .any(|w| w.code == "input.truncated"));
    assert_eq!(
        trunc_lz.candidates.as_deref(),
        Some(&["lz".to_string()][..])
    );
}
