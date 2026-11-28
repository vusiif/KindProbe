use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification};
use std::ffi::OsStr;

#[test]
fn test_detect_typography_fonts_and_disks() {
    let opts = DetectOptions::default();

    // 1. X11 PCF Font
    let pcf_bytes = b"\x01fcp\x00\x00\x00\x04\x00\x00\x00\x00";
    let rep_pcf = detect_bytes(pcf_bytes, &opts).expect("detect pcf");
    assert_eq!(rep_pcf.identification, Identification::Format);
    assert_eq!(rep_pcf.format_id.as_deref(), Some("pcf"));
    assert_eq!(rep_pcf.category, "font");
    assert_eq!(
        compare_extension(&rep_pcf, Some(OsStr::new("pcf"))),
        ExtensionStatus::Match
    );

    // 2. X11 BDF Font
    let bdf_bytes =
        b"STARTFONT 2.1\nFONT -Misc-Fixed-Medium-R-Normal--14-130-75-75-C-70-ISO8859-1\n";
    let rep_bdf = detect_bytes(bdf_bytes, &opts).expect("detect bdf");
    assert_eq!(rep_bdf.format_id.as_deref(), Some("bdf"));
    assert_eq!(
        compare_extension(&rep_bdf, Some(OsStr::new("bdf"))),
        ExtensionStatus::Match
    );

    // 3. Adobe Font Metrics (AFM)
    let afm_bytes = b"StartFontMetrics 4.1\nFontName Helvetica\n";
    let rep_afm = detect_bytes(afm_bytes, &opts).expect("detect afm");
    assert_eq!(rep_afm.format_id.as_deref(), Some("afm"));
    assert_eq!(
        compare_extension(&rep_afm, Some(OsStr::new("afm"))),
        ExtensionStatus::Match
    );

    // 4. VirtualBox Disk Image (VDI)
    let mut vdi_bytes = vec![0u8; 100];
    vdi_bytes[..4].copy_from_slice(b"<<< ");
    vdi_bytes[10..36].copy_from_slice(b"VirtualBox Disk Image >>>\n");
    let rep_vdi = detect_bytes(&vdi_bytes, &opts).expect("detect vdi");
    assert_eq!(rep_vdi.format_id.as_deref(), Some("vdi"));
    assert_eq!(rep_vdi.category, "disk");
    assert_eq!(
        compare_extension(&rep_vdi, Some(OsStr::new("vdi"))),
        ExtensionStatus::Match
    );

    // 5. MAME Compressed Hunks of Data (CHD)
    let chd_bytes = b"MComprHD\x00\x00\x00\x05\x00\x00\x00\x00";
    let rep_chd = detect_bytes(chd_bytes, &opts).expect("detect chd");
    assert_eq!(rep_chd.format_id.as_deref(), Some("chd"));
    assert_eq!(
        compare_extension(&rep_chd, Some(OsStr::new("chd"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_crypto_and_scientific_lidar() {
    let opts = DetectOptions::default();

    // 1. Java KeyStore (JKS)
    let jks_bytes = b"\xfe\xed\xfe\xed\x00\x00\x00\x02\x00\x00\x00\x01";
    let rep_jks = detect_bytes(jks_bytes, &opts).expect("detect jks");
    assert_eq!(rep_jks.format_id.as_deref(), Some("jks"));
    assert_eq!(rep_jks.category, "security");
    assert_eq!(
        compare_extension(&rep_jks, Some(OsStr::new("jks"))),
        ExtensionStatus::Match
    );

    // 2. PEM Certificate / Key
    let pem_bytes = b"-----BEGIN CERTIFICATE-----\nMIIB...test...\n-----END CERTIFICATE-----\n";
    let rep_pem = detect_bytes(pem_bytes, &opts).expect("detect pem");
    assert_eq!(rep_pem.format_id.as_deref(), Some("pem"));
    assert_eq!(rep_pem.category, "security");
    assert_eq!(
        compare_extension(&rep_pem, Some(OsStr::new("pem"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_pem, Some(OsStr::new("crt"))),
        ExtensionStatus::Match
    );

    // 3. ASPRS LIDAR Point Cloud (LAS)
    let las_bytes = b"LASF\x00\x00\x00\x00\x00\x00\x00\x00\x01\x04";
    let rep_las = detect_bytes(las_bytes, &opts).expect("detect las");
    assert_eq!(rep_las.format_id.as_deref(), Some("las"));
    assert_eq!(rep_las.category, "scientific");
    assert_eq!(
        compare_extension(&rep_las, Some(OsStr::new("las"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_las, Some(OsStr::new("laz"))),
        ExtensionStatus::Match
    );

    // 4. ZPAQ Archive
    let zpaq_bytes = b"zPAQ\x01\x00\x00\x00";
    let rep_zpaq = detect_bytes(zpaq_bytes, &opts).expect("detect zpaq");
    assert_eq!(rep_zpaq.format_id.as_deref(), Some("zpaq"));
    assert_eq!(rep_zpaq.category, "archive");
    assert_eq!(
        compare_extension(&rep_zpaq, Some(OsStr::new("zpaq"))),
        ExtensionStatus::Match
    );
}
