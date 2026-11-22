use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::ExtensionStatus;

#[test]
fn test_detect_texture_formats() {
    let opts = DetectOptions::default();

    // 1. DDS (DirectDraw Surface)
    let mut dds = vec![0u8; 128];
    dds[0..4].copy_from_slice(b"DDS ");
    let rep = detect_bytes(&dds, &opts).expect("detect dds");
    assert_eq!(rep.format_id.as_deref(), Some("dds"));
    assert_eq!(rep.category, "image");
    assert_eq!(rep.recommended_extension.as_deref(), Some("dds"));
    assert_eq!(compare_extension(&rep, Some(OsStr::new("dds"))), ExtensionStatus::Match);

    // 2. KTX 1.0 & KTX 2.0
    let mut ktx1 = vec![0u8; 64];
    ktx1[0..12].copy_from_slice(b"\xabKTX 11\xbb\r\n\x1a\n");
    let rep = detect_bytes(&ktx1, &opts).expect("detect ktx1");
    assert_eq!(rep.format_id.as_deref(), Some("ktx"));

    let mut ktx2 = vec![0u8; 64];
    ktx2[0..12].copy_from_slice(b"\xabKTX 20\xbb\r\n\x1a\n");
    let rep = detect_bytes(&ktx2, &opts).expect("detect ktx2");
    assert_eq!(rep.format_id.as_deref(), Some("ktx2"));

    // 3. ASTC
    let mut astc = vec![0u8; 64];
    astc[0..4].copy_from_slice(&[0x13, 0xab, 0xa1, 0x5c]);
    let rep = detect_bytes(&astc, &opts).expect("detect astc");
    assert_eq!(rep.format_id.as_deref(), Some("astc"));

    // 4. PVR 3.0
    let mut pvr = vec![0u8; 64];
    pvr[0..4].copy_from_slice(b"PVR\x03");
    let rep = detect_bytes(&pvr, &opts).expect("detect pvr");
    assert_eq!(rep.format_id.as_deref(), Some("pvr"));
}

#[test]
fn test_detect_multimedia_streams() {
    let opts = DetectOptions::default();

    // 1. ASF / WMV / WMA
    let mut asf = vec![0u8; 64];
    asf[0..16].copy_from_slice(&[
        0x30, 0x26, 0xb2, 0x75, 0x8e, 0x66, 0xcf, 0x11, 0xa6, 0xd9, 0x00, 0xaa, 0x00, 0x62, 0xce, 0x6c,
    ]);
    let rep = detect_bytes(&asf, &opts).expect("detect asf");
    assert_eq!(rep.format_id.as_deref(), Some("asf"));
    assert_eq!(rep.category, "video");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("wmv"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("wma"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("asf"))), ExtensionStatus::Match);

    // 2. RealMedia (.rm / .rmvb)
    let mut rm = vec![0u8; 32];
    rm[0..4].copy_from_slice(b".RMF");
    let rep = detect_bytes(&rm, &opts).expect("detect rm");
    assert_eq!(rep.format_id.as_deref(), Some("rm"));
    assert_eq!(compare_extension(&rep, Some(OsStr::new("rmvb"))), ExtensionStatus::Match);

    // 3. Flash SWF (FWS, CWS, ZWS)
    for magic in [b"FWS", b"CWS", b"ZWS"] {
        let mut swf = vec![0u8; 32];
        swf[0..3].copy_from_slice(magic);
        let rep = detect_bytes(&swf, &opts).expect("detect swf");
        assert_eq!(rep.format_id.as_deref(), Some("swf"));
    }

    // 4. DTS Audio
    let mut dts = vec![0u8; 32];
    dts[0..4].copy_from_slice(&[0x7f, 0xfe, 0x80, 0x01]);
    let rep = detect_bytes(&dts, &opts).expect("detect dts");
    assert_eq!(rep.format_id.as_deref(), Some("dts"));
    assert_eq!(rep.category, "audio");
}

#[test]
fn test_detect_extended_database_and_archives() {
    let opts = DetectOptions::default();

    // 1. Microsoft Access MDB and ACCDB
    let mut mdb = vec![0u8; 64];
    mdb[0..19].copy_from_slice(b"\x00\x01\x00\x00Standard Jet DB");
    let rep = detect_bytes(&mdb, &opts).expect("detect mdb");
    assert_eq!(rep.format_id.as_deref(), Some("mdb"));

    let mut accdb = vec![0u8; 64];
    accdb[0..19].copy_from_slice(b"\x00\x01\x00\x00Standard ACE DB");
    let rep = detect_bytes(&accdb, &opts).expect("detect accdb");
    assert_eq!(rep.format_id.as_deref(), Some("accdb"));

    // 2. Game archives (DOOM WAD & Quake PAK)
    let mut wad = vec![0u8; 32];
    wad[0..4].copy_from_slice(b"IWAD");
    let rep = detect_bytes(&wad, &opts).expect("detect wad");
    assert_eq!(rep.format_id.as_deref(), Some("wad"));

    let mut pak = vec![0u8; 32];
    pak[0..4].copy_from_slice(b"PACK");
    let rep = detect_bytes(&pak, &opts).expect("detect pak");
    assert_eq!(rep.format_id.as_deref(), Some("pak"));

    // 3. Documents & Help (WinHelp HLP & Microsoft Reader LIT)
    let mut hlp = vec![0u8; 32];
    hlp[0..4].copy_from_slice(&[0x3f, 0x5f, 0x03, 0x00]);
    let rep = detect_bytes(&hlp, &opts).expect("detect hlp");
    assert_eq!(rep.format_id.as_deref(), Some("hlp"));

    let mut lit = vec![0u8; 32];
    lit[0..8].copy_from_slice(b"ITOLITLS");
    let rep = detect_bytes(&lit, &opts).expect("detect lit");
    assert_eq!(rep.format_id.as_deref(), Some("lit"));
}
