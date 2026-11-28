use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::extension::compare_extension;
use kindprobe::model::ExtensionStatus;
use std::ffi::OsStr;

#[test]
fn test_detect_publishing_and_fonts() {
    let opts = DetectOptions::default();

    // 1. TeX Packed Raster Font (PK)
    let mut pk = vec![0u8; 32];
    pk[0..2].copy_from_slice(b"\xf7\x59");
    let rep_pk = detect_bytes(&pk, &opts).expect("detect pk");
    assert_eq!(rep_pk.format_id.as_deref(), Some("pk"));
    assert_eq!(rep_pk.category, "font");
    assert_eq!(
        compare_extension(&rep_pk, Some(OsStr::new("pk"))),
        ExtensionStatus::Match
    );

    // 2. TeX Device Independent Document (DVI)
    let mut dvi = vec![0u8; 32];
    dvi[0..2].copy_from_slice(b"\xf7\x02");
    let rep_dvi = detect_bytes(&dvi, &opts).expect("detect dvi");
    assert_eq!(rep_dvi.format_id.as_deref(), Some("dvi"));
    assert_eq!(rep_dvi.category, "document");
    assert_eq!(
        compare_extension(&rep_dvi, Some(OsStr::new("dvi"))),
        ExtensionStatus::Match
    );

    // 3. Autodesk Animator FLI
    let mut fli = vec![0u8; 32];
    fli[4..6].copy_from_slice(b"\x11\xaf");
    let rep_fli = detect_bytes(&fli, &opts).expect("detect fli");
    assert_eq!(rep_fli.format_id.as_deref(), Some("fli"));
    assert_eq!(rep_fli.category, "video");
    assert_eq!(
        compare_extension(&rep_fli, Some(OsStr::new("fli"))),
        ExtensionStatus::Match
    );

    // 4. DOS Binary Encapsulated PostScript (EPS)
    let mut eps = vec![0u8; 32];
    eps[0..4].copy_from_slice(b"\xc5\xd0\xd3\xc6");
    let rep_eps = detect_bytes(&eps, &opts).expect("detect eps_bin");
    assert_eq!(rep_eps.format_id.as_deref(), Some("eps_bin"));
    assert_eq!(rep_eps.category, "image");
    assert_eq!(
        compare_extension(&rep_eps, Some(OsStr::new("eps"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_archives_and_security() {
    let opts = DetectOptions::default();

    // 1. Java Pack200 Compressed JAR
    let mut pack200 = vec![0u8; 32];
    pack200[0..4].copy_from_slice(b"\xca\xfe\xd0\x0d");
    let rep_pack = detect_bytes(&pack200, &opts).expect("detect pack200");
    assert_eq!(rep_pack.format_id.as_deref(), Some("pack200"));
    assert_eq!(rep_pack.category, "archive");
    assert_eq!(
        compare_extension(&rep_pack, Some(OsStr::new("pack"))),
        ExtensionStatus::Match
    );

    // 2. InstallShield CAB Archive (ISC)
    let mut isc = vec![0u8; 32];
    isc[0..4].copy_from_slice(b"ISc(");
    let rep_isc = detect_bytes(&isc, &opts).expect("detect isc");
    assert_eq!(rep_isc.format_id.as_deref(), Some("isc"));
    assert_eq!(rep_isc.category, "archive");
    assert_eq!(
        compare_extension(&rep_isc, Some(OsStr::new("isc"))),
        ExtensionStatus::Match
    );

    // 3. Age Encryption Container
    let mut age = vec![0u8; 32];
    age[0..22].copy_from_slice(b"age-encryption.org/v1\n");
    let rep_age = detect_bytes(&age, &opts).expect("detect age");
    assert_eq!(rep_age.format_id.as_deref(), Some("age"));
    assert_eq!(rep_age.category, "security");
    assert_eq!(
        compare_extension(&rep_age, Some(OsStr::new("age"))),
        ExtensionStatus::Match
    );

    // 4. PuTTY Private Key (PPK)
    let mut ppk = vec![0u8; 32];
    ppk[0..20].copy_from_slice(b"PuTTY-User-Key-File-");
    let rep_ppk = detect_bytes(&ppk, &opts).expect("detect ppk");
    assert_eq!(rep_ppk.format_id.as_deref(), Some("ppk"));
    assert_eq!(rep_ppk.category, "security");
    assert_eq!(
        compare_extension(&rep_ppk, Some(OsStr::new("ppk"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_images_textures_and_firmware() {
    let opts = DetectOptions::default();

    // 1. Sun Raster Image (RAS)
    let mut ras = vec![0u8; 32];
    ras[0..4].copy_from_slice(b"\x59\xa6\x6a\x95");
    let rep_ras = detect_bytes(&ras, &opts).expect("detect ras");
    assert_eq!(rep_ras.format_id.as_deref(), Some("ras"));
    assert_eq!(rep_ras.category, "image");
    assert_eq!(
        compare_extension(&rep_ras, Some(OsStr::new("ras"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_ras, Some(OsStr::new("sun"))),
        ExtensionStatus::Match
    );

    // 2. Radiance HDR Image
    let mut hdr = vec![0u8; 32];
    hdr[0..11].copy_from_slice(b"#?RADIANCE\n");
    let rep_hdr = detect_bytes(&hdr, &opts).expect("detect hdr");
    assert_eq!(rep_hdr.format_id.as_deref(), Some("hdr"));
    assert_eq!(rep_hdr.category, "image");
    assert_eq!(
        compare_extension(&rep_hdr, Some(OsStr::new("hdr"))),
        ExtensionStatus::Match
    );

    // 3. PowerVR Texture Container (PVRT)
    let mut pvrt = vec![0u8; 32];
    pvrt[0..4].copy_from_slice(b"PVRT");
    let rep_pvrt = detect_bytes(&pvrt, &opts).expect("detect pvrt");
    assert_eq!(rep_pvrt.format_id.as_deref(), Some("pvrt"));
    assert_eq!(rep_pvrt.category, "image");
    assert_eq!(
        compare_extension(&rep_pvrt, Some(OsStr::new("pvr"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_pvrt, Some(OsStr::new("pvrt"))),
        ExtensionStatus::Match
    );

    // 4. STM32 DfuSe Microcontroller Firmware (DFU)
    let mut dfu = vec![0u8; 32];
    dfu[0..6].copy_from_slice(b"DfuSe\x01");
    let rep_dfu = detect_bytes(&dfu, &opts).expect("detect dfu");
    assert_eq!(rep_dfu.format_id.as_deref(), Some("dfu"));
    assert_eq!(rep_dfu.category, "system");
    assert_eq!(
        compare_extension(&rep_dfu, Some(OsStr::new("dfu"))),
        ExtensionStatus::Match
    );

    // 5. R Data Workspace / RDS
    let mut rdata = vec![0u8; 32];
    rdata[0..5].copy_from_slice(b"RDX2\n");
    let rep_rdata = detect_bytes(&rdata, &opts).expect("detect rdata");
    assert_eq!(rep_rdata.format_id.as_deref(), Some("rdata"));
    assert_eq!(rep_rdata.category, "scientific");
    assert_eq!(
        compare_extension(&rep_rdata, Some(OsStr::new("rdata"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_rdata, Some(OsStr::new("rds"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_rdata, Some(OsStr::new("rda"))),
        ExtensionStatus::Match
    );
}
