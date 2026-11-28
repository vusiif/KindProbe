use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus};
use std::ffi::OsStr;

#[test]
fn test_detect_game_roms() {
    let opts = DetectOptions::default();

    // 1. NES ROM
    let nes_bytes = b"NES\x1a\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep_nes = detect_bytes(nes_bytes, &opts).expect("detect nes");
    assert_eq!(rep_nes.format_id.as_deref(), Some("nes"));
    assert_eq!(rep_nes.category, "rom");

    // 2. Nintendo 64 Big-Endian (z64)
    let z64_bytes = b"\x80\x37\x12\x40\x00\x00\x00\x0f";
    let rep_z64 = detect_bytes(z64_bytes, &opts).expect("detect z64");
    assert_eq!(rep_z64.format_id.as_deref(), Some("z64"));
    assert_eq!(rep_z64.category, "rom");
    assert_eq!(
        compare_extension(&rep_z64, Some(OsStr::new("z64"))),
        ExtensionStatus::Match
    );

    // 3. Nintendo 64 Little-Endian (n64)
    let n64_bytes = b"\x40\x12\x37\x80\x0f\x00\x00\x00";
    let rep_n64 = detect_bytes(n64_bytes, &opts).expect("detect n64");
    assert_eq!(rep_n64.format_id.as_deref(), Some("n64"));

    // 4. Game Boy Advance (GBA)
    let mut gba_bytes = vec![0u8; 192];
    gba_bytes[4..8].copy_from_slice(b"\x24\xff\xae\x51");
    let rep_gba = detect_bytes(&gba_bytes, &opts).expect("detect gba");
    assert_eq!(rep_gba.format_id.as_deref(), Some("gba"));

    // 5. Game Boy (GB)
    let mut gb_bytes = vec![0u8; 300];
    gb_bytes[260..268].copy_from_slice(b"\xce\xed\x66\x66\xcc\x0d\x00\x0b");
    let rep_gb = detect_bytes(&gb_bytes, &opts).expect("detect gb");
    assert_eq!(rep_gb.format_id.as_deref(), Some("gb"));
    assert_eq!(
        compare_extension(&rep_gb, Some(OsStr::new("gb"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_gb, Some(OsStr::new("gbc"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_audio_trackers_and_pro_audio() {
    let opts = DetectOptions::default();

    // 1. Scream Tracker 3 (S3M)
    let mut s3m_bytes = vec![0u8; 60];
    s3m_bytes[44..48].copy_from_slice(b"SCRM");
    let rep_s3m = detect_bytes(&s3m_bytes, &opts).expect("detect s3m");
    assert_eq!(rep_s3m.format_id.as_deref(), Some("s3m"));
    assert_eq!(rep_s3m.category, "audio");

    // 2. FastTracker II (XM)
    let xm_bytes = b"Extended Module: Test Song\x1aFastTracker v2.00\x00";
    let rep_xm = detect_bytes(xm_bytes, &opts).expect("detect xm");
    assert_eq!(rep_xm.format_id.as_deref(), Some("xm"));

    // 3. Impulse Tracker (IT)
    let it_bytes = b"IMPMSong Title\x00\x00\x00\x00\x00\x00";
    let rep_it = detect_bytes(it_bytes, &opts).expect("detect it");
    assert_eq!(rep_it.format_id.as_deref(), Some("it"));

    // 4. ProTracker (MOD)
    let mut mod_bytes = vec![0u8; 1084];
    mod_bytes[1080..1084].copy_from_slice(b"M.K.");
    let rep_mod = detect_bytes(&mod_bytes, &opts).expect("detect mod");
    assert_eq!(rep_mod.format_id.as_deref(), Some("mod"));

    // 5. Monkey's Audio (APE)
    let ape_bytes = b"MAC \xe8\x03\x00\x00\x00\x00\x00\x00";
    let rep_ape = detect_bytes(ape_bytes, &opts).expect("detect ape");
    assert_eq!(rep_ape.format_id.as_deref(), Some("ape"));

    // 6. WavPack (WV)
    let wv_bytes = b"wvpk\x20\x00\x00\x00\x04\x04\x00\x00";
    let rep_wv = detect_bytes(wv_bytes, &opts).expect("detect wv");
    assert_eq!(rep_wv.format_id.as_deref(), Some("wv"));
}
