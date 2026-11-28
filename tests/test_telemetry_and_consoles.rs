use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::ExtensionStatus;
use std::ffi::OsStr;

#[test]
fn test_detect_telemetry_and_pcb() {
    let opts = DetectOptions::default();

    // 1. Vector CAN Binary Log File (BLF)
    let mut blf = vec![0u8; 32];
    blf[0..4].copy_from_slice(b"LOGG");
    let rep_blf = detect_bytes(&blf, &opts).expect("detect blf");
    assert_eq!(rep_blf.format_id.as_deref(), Some("blf"));
    assert_eq!(rep_blf.category, "system");
    assert_eq!(
        compare_extension(&rep_blf, Some(OsStr::new("blf"))),
        ExtensionStatus::Match
    );

    // 2. RS-274X Extended Gerber PCB (GBR)
    let mut gbr = vec![0u8; 32];
    gbr[0..3].copy_from_slice(b"%FS");
    let rep_gbr = detect_bytes(&gbr, &opts).expect("detect gbr");
    assert_eq!(rep_gbr.format_id.as_deref(), Some("gbr"));
    assert_eq!(rep_gbr.category, "model");
    assert_eq!(
        compare_extension(&rep_gbr, Some(OsStr::new("gbr"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_gbr, Some(OsStr::new("ger"))),
        ExtensionStatus::Match
    );

    // 3. Self-Describing CBOR (RFC 8949)
    let mut cbor = vec![0u8; 32];
    cbor[0..3].copy_from_slice(b"\xd9\xd9\xf7");
    let rep_cbor = detect_bytes(&cbor, &opts).expect("detect cbor");
    assert_eq!(rep_cbor.format_id.as_deref(), Some("cbor"));
    assert_eq!(rep_cbor.category, "database");
    assert_eq!(
        compare_extension(&rep_cbor, Some(OsStr::new("cbor"))),
        ExtensionStatus::Match
    );

    // 4. Lzop Compressed Archive (LZO)
    let mut lzo = vec![0u8; 32];
    lzo[0..9].copy_from_slice(b"\x89LZO\x00\r\n\x1a\n");
    let rep_lzo = detect_bytes(&lzo, &opts).expect("detect lzo");
    assert_eq!(rep_lzo.format_id.as_deref(), Some("lzo"));
    assert_eq!(rep_lzo.category, "archive");
    assert_eq!(
        compare_extension(&rep_lzo, Some(OsStr::new("lzo"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_crash_dump_and_crypto() {
    let opts = DetectOptions::default();

    // 1. Windows MiniDump (DMP)
    let mut dmp = vec![0u8; 32];
    dmp[0..6].copy_from_slice(b"MDMP\x93\xa7");
    let rep_dmp = detect_bytes(&dmp, &opts).expect("detect dmp");
    assert_eq!(rep_dmp.format_id.as_deref(), Some("dmp"));
    assert_eq!(rep_dmp.category, "system");
    assert_eq!(
        compare_extension(&rep_dmp, Some(OsStr::new("dmp"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_dmp, Some(OsStr::new("mdmp"))),
        ExtensionStatus::Match
    );

    // 2. GnuPG Keybox (KBX)
    let mut kbx = vec![0u8; 32];
    kbx[8..12].copy_from_slice(b"KBXf");
    let rep_kbx = detect_bytes(&kbx, &opts).expect("detect kbx");
    assert_eq!(rep_kbx.format_id.as_deref(), Some("kbx"));
    assert_eq!(rep_kbx.category, "security");
    assert_eq!(
        compare_extension(&rep_kbx, Some(OsStr::new("kbx"))),
        ExtensionStatus::Match
    );

    // 3. Autodesk Animator FLC
    let mut flc = vec![0u8; 32];
    flc[4..6].copy_from_slice(b"\x12\xaf");
    let rep_flc = detect_bytes(&flc, &opts).expect("detect flc");
    assert_eq!(rep_flc.format_id.as_deref(), Some("flc"));
    assert_eq!(rep_flc.category, "video");
    assert_eq!(
        compare_extension(&rep_flc, Some(OsStr::new("flc"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_flc, Some(OsStr::new("fli"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_consoles_and_textures() {
    let opts = DetectOptions::default();

    // 1. Sega Genesis / Mega Drive (GEN)
    let mut gen = vec![0u8; 300];
    gen[256..260].copy_from_slice(b"SEGA");
    let rep_gen = detect_bytes(&gen, &opts).expect("detect gen");
    assert_eq!(rep_gen.format_id.as_deref(), Some("gen"));
    assert_eq!(rep_gen.category, "rom");
    assert_eq!(
        compare_extension(&rep_gen, Some(OsStr::new("gen"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_gen, Some(OsStr::new("smd"))),
        ExtensionStatus::Match
    );

    // 2. Commodore 64 Tape Image (T64)
    let mut t64 = vec![0u8; 32];
    t64[0..19].copy_from_slice(b"C64 tape image file");
    let rep_t64 = detect_bytes(&t64, &opts).expect("detect t64");
    assert_eq!(rep_t64.format_id.as_deref(), Some("t64"));
    assert_eq!(rep_t64.category, "rom");
    assert_eq!(
        compare_extension(&rep_t64, Some(OsStr::new("t64"))),
        ExtensionStatus::Match
    );

    // 3. Creative Voice File (VOC)
    let mut voc = vec![0u8; 32];
    voc[0..20].copy_from_slice(b"Creative Voice File\x1a");
    let rep_voc = detect_bytes(&voc, &opts).expect("detect voc");
    assert_eq!(rep_voc.format_id.as_deref(), Some("voc"));
    assert_eq!(rep_voc.category, "audio");
    assert_eq!(
        compare_extension(&rep_voc, Some(OsStr::new("voc"))),
        ExtensionStatus::Match
    );

    // 4. Sony PlayStation TIM2 Texture (TM2)
    let mut tm2 = vec![0u8; 32];
    tm2[0..4].copy_from_slice(b"TIM2");
    let rep_tm2 = detect_bytes(&tm2, &opts).expect("detect tm2");
    assert_eq!(rep_tm2.format_id.as_deref(), Some("tm2"));
    assert_eq!(rep_tm2.category, "image");
    assert_eq!(
        compare_extension(&rep_tm2, Some(OsStr::new("tm2"))),
        ExtensionStatus::Match
    );

    // 5. GameCube / Wii Texture Palette Library (TPL)
    let mut tpl = vec![0u8; 32];
    tpl[0..4].copy_from_slice(b"\x00\x20\xaf\x30");
    let rep_tpl = detect_bytes(&tpl, &opts).expect("detect tpl");
    assert_eq!(rep_tpl.format_id.as_deref(), Some("tpl"));
    assert_eq!(rep_tpl.category, "image");
    assert_eq!(
        compare_extension(&rep_tpl, Some(OsStr::new("tpl"))),
        ExtensionStatus::Match
    );
}
