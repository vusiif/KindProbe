use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::ExtensionStatus;
use std::ffi::OsStr;

#[test]
fn test_detect_bytecode_and_static_libs() {
    let opts = DetectOptions::default();

    // 1. Android Dalvik Executable (DEX)
    let mut dex = vec![0u8; 32];
    dex[0..8].copy_from_slice(b"dex\n035\x00");
    let rep = detect_bytes(&dex, &opts).expect("detect dex");
    assert_eq!(rep.format_id.as_deref(), Some("dex"));
    assert_eq!(rep.category, "executable");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("dex"))),
        ExtensionStatus::Match
    );

    // 2. Lua Bytecode (LUAC)
    let mut luac = vec![0u8; 32];
    luac[0..4].copy_from_slice(b"\x1bLua");
    let rep = detect_bytes(&luac, &opts).expect("detect luac");
    assert_eq!(rep.format_id.as_deref(), Some("luac"));
    assert_eq!(rep.category, "executable");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("luac"))),
        ExtensionStatus::Match
    );

    // 3. Unix Archive / Static Library (AR / .a / .lib)
    let mut ar = vec![0u8; 32];
    ar[0..8].copy_from_slice(b"!<arch>\x0a");
    let rep = detect_bytes(&ar, &opts).expect("detect ar");
    assert_eq!(rep.format_id.as_deref(), Some("ar"));
    assert_eq!(rep.category, "archive");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("a"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("lib"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_vector_graphics_and_modern_images() {
    let opts = DetectOptions::default();

    // 1. Windows Metafile (WMF)
    let mut wmf = vec![0u8; 32];
    wmf[0..4].copy_from_slice(&[0xd7, 0xcd, 0xc6, 0x9a]);
    let rep = detect_bytes(&wmf, &opts).expect("detect wmf");
    assert_eq!(rep.format_id.as_deref(), Some("wmf"));
    assert_eq!(rep.category, "image");

    // 2. Enhanced Metafile (EMF)
    let mut emf = vec![0u8; 64];
    emf[0..4].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    emf[40..44].copy_from_slice(b" EMF");
    let rep = detect_bytes(&emf, &opts).expect("detect emf");
    assert_eq!(rep.format_id.as_deref(), Some("emf"));
    assert_eq!(rep.category, "image");

    // 3. QOI (Quite OK Image)
    let mut qoi = vec![0u8; 32];
    qoi[0..4].copy_from_slice(b"qoif");
    let rep = detect_bytes(&qoi, &opts).expect("detect qoi");
    assert_eq!(rep.format_id.as_deref(), Some("qoi"));

    // 4. JPEG XL (JXL raw and container)
    let mut jxl_raw = vec![0u8; 32];
    jxl_raw[0..2].copy_from_slice(&[0xff, 0x0a]);
    let rep = detect_bytes(&jxl_raw, &opts).expect("detect jxl raw");
    assert_eq!(rep.format_id.as_deref(), Some("jxl"));

    let mut jxl_box = vec![0u8; 32];
    jxl_box[0..12].copy_from_slice(b"\x00\x00\x00\x0cJXL \r\n\x87\n");
    let rep = detect_bytes(&jxl_box, &opts).expect("detect jxl box");
    assert_eq!(rep.format_id.as_deref(), Some("jxl"));

    // 5. FLIF (Free Lossless Image Format)
    let mut flif = vec![0u8; 32];
    flif[0..4].copy_from_slice(b"FLIF");
    let rep = detect_bytes(&flif, &opts).expect("detect flif");
    assert_eq!(rep.format_id.as_deref(), Some("flif"));
}

#[test]
fn test_detect_bvh_and_snoop() {
    let opts = DetectOptions::default();

    // 1. Biovision Hierarchy Motion Capture (BVH)
    let mut bvh = vec![0u8; 32];
    bvh[0..9].copy_from_slice(b"HIERARCHY");
    let rep = detect_bytes(&bvh, &opts).expect("detect bvh");
    assert_eq!(rep.format_id.as_deref(), Some("bvh"));
    assert_eq!(rep.category, "model");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("bvh"))),
        ExtensionStatus::Match
    );

    // 2. RFC 1761 Snoop Network Packet Capture (Snoop)
    let mut snoop = vec![0u8; 32];
    snoop[0..8].copy_from_slice(b"snoop\x00\x00\x00");
    let rep = detect_bytes(&snoop, &opts).expect("detect snoop");
    assert_eq!(rep.format_id.as_deref(), Some("snoop"));
    assert_eq!(rep.category, "network");
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("snoop"))),
        ExtensionStatus::Match
    );
}
