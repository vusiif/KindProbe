use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus};
use std::ffi::OsStr;

#[test]
fn test_detect_database_and_security() {
    let opts = DetectOptions::default();

    // 1. Apache Arrow / Feather IPC
    let arrow_bytes = b"ARROW1\x00\x00\x00\x00\x00\x00";
    let rep_arrow = detect_bytes(arrow_bytes, &opts).expect("detect arrow");
    assert_eq!(rep_arrow.format_id.as_deref(), Some("arrow"));
    assert_eq!(rep_arrow.category, "database");
    assert_eq!(
        compare_extension(&rep_arrow, Some(OsStr::new("arrow"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_arrow, Some(OsStr::new("feather"))),
        ExtensionStatus::Match
    );

    // 2. KeePass 2.x Password Database (KDBX)
    let kdbx_bytes = b"\x03\xd9\xa2\x9a\x67\xfb\x4b\xb5\x00\x00\x03\x00";
    let rep_kdbx = detect_bytes(kdbx_bytes, &opts).expect("detect kdbx");
    assert_eq!(rep_kdbx.format_id.as_deref(), Some("kdbx"));
    assert_eq!(rep_kdbx.category, "database");

    // 3. KeePass 1.x Password Database (KDB)
    let kdb_bytes = b"\x03\xd9\xa2\x9a\x65\xfb\x4b\xb5\x00\x00\x01\x00";
    let rep_kdb = detect_bytes(kdb_bytes, &opts).expect("detect kdb");
    assert_eq!(rep_kdb.format_id.as_deref(), Some("kdb"));

    // 4. Realm Database
    let realm_bytes = b"T-D-B!\x00\x00\x09\x00\x00\x00";
    let rep_realm = detect_bytes(realm_bytes, &opts).expect("detect realm");
    assert_eq!(rep_realm.format_id.as_deref(), Some("realm"));
}

#[test]
fn test_detect_packages_and_netpbm_extended() {
    let opts = DetectOptions::default();

    // 1. Snappy Framed (SZ)
    let sz_bytes = b"\xff\x06\x00\x00sNaPpY\x01\x04\x00\x00";
    let rep_sz = detect_bytes(sz_bytes, &opts).expect("detect snappy");
    assert_eq!(rep_sz.format_id.as_deref(), Some("snappy"));
    assert_eq!(rep_sz.category, "archive");
    assert_eq!(
        compare_extension(&rep_sz, Some(OsStr::new("sz"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_sz, Some(OsStr::new("snappy"))),
        ExtensionStatus::Match
    );

    // 2. Unix Compress (.Z)
    let z_bytes = b"\x1f\x9d\x90\x00\x00";
    let rep_z = detect_bytes(z_bytes, &opts).expect("detect z");
    assert_eq!(rep_z.format_id.as_deref(), Some("z"));

    // 3. LHA / LZH Archive
    let lha_bytes = b"\x20\x00-lh5-\x00\x00\x00\x00";
    let rep_lha = detect_bytes(lha_bytes, &opts).expect("detect lha");
    assert_eq!(rep_lha.format_id.as_deref(), Some("lha"));
    assert_eq!(
        compare_extension(&rep_lha, Some(OsStr::new("lzh"))),
        ExtensionStatus::Match
    );

    // 4. Netpbm PAM (P7)
    let pam_bytes =
        b"P7\nWIDTH 1\nHEIGHT 1\nDEPTH 1\nMAXVAL 255\nTUPLTYPE BLACKANDWHITE\nENDHDR\n\xff";
    let rep_pam = detect_bytes(pam_bytes, &opts).expect("detect pam");
    assert_eq!(rep_pam.format_id.as_deref(), Some("pam"));
    assert_eq!(rep_pam.category, "image");

    // 5. Silicon Graphics Image (SGI / RGB)
    let sgi_bytes = b"\x01\xda\x00\x01\x00\x02\x00\x01\x00\x01";
    let rep_sgi = detect_bytes(sgi_bytes, &opts).expect("detect sgi");
    assert_eq!(rep_sgi.format_id.as_deref(), Some("sgi"));
    assert_eq!(
        compare_extension(&rep_sgi, Some(OsStr::new("rgb"))),
        ExtensionStatus::Match
    );
}
