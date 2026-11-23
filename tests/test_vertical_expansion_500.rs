use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};

#[test]
fn test_detect_batch_500_system_and_media() {
    let opts = DetectOptions::default();

    // 1. SquashFS compressed read-only filesystem (sqsh / squashfs)
    let sqsh_data = b"hsqs\x00\x00\x00\x00";
    let rep_sqsh = detect_bytes(sqsh_data, &opts).expect("detect squashfs");
    assert_eq!(rep_sqsh.format_id.as_deref(), Some("squashfs"));
    assert_eq!(rep_sqsh.category, "archive");
    assert_eq!(rep_sqsh.verification, Verification::Signature);
    assert_eq!(compare_extension(&rep_sqsh, Some(OsStr::new("sqsh"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_sqsh, Some(OsStr::new("squashfs"))), ExtensionStatus::Match);

    // 2. NES Sound Format (nsf)
    let nsf_data = b"NESM\x1a\x01\x01\x00";
    let rep_nsf = detect_bytes(nsf_data, &opts).expect("detect nsf");
    assert_eq!(rep_nsf.format_id.as_deref(), Some("nsf"));
    assert_eq!(rep_nsf.category, "audio");
    assert_eq!(compare_extension(&rep_nsf, Some(OsStr::new("nsf"))), ExtensionStatus::Match);

    // 3. LLVM Bitcode (bc)
    let bc_data = b"BC\xc0\xde\x00\x00\x01\x00";
    let rep_bc = detect_bytes(bc_data, &opts).expect("detect bc");
    assert_eq!(rep_bc.format_id.as_deref(), Some("bc"));
    assert_eq!(rep_bc.category, "bytecode");
    assert_eq!(compare_extension(&rep_bc, Some(OsStr::new("bc"))), ExtensionStatus::Match);

    // 4. KML Keyhole Markup Language (kml)
    let kml_data = b"<kml xmlns=\"http://www.opengis.net/kml/2.2\"><Document></Document></kml>";
    let rep_kml = detect_bytes(kml_data, &opts).expect("detect kml");
    assert_eq!(rep_kml.format_id.as_deref(), Some("kml"));
    assert_eq!(rep_kml.category, "geospatial");
    assert_eq!(compare_extension(&rep_kml, Some(OsStr::new("kml"))), ExtensionStatus::Match);

    // 5. OpenSSH Ed25519 Public Key (pub)
    let pub_data = b"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExampleKey pubkey";
    let rep_pub = detect_bytes(pub_data, &opts).expect("detect sshpub");
    assert_eq!(rep_pub.format_id.as_deref(), Some("sshpub"));
    assert_eq!(rep_pub.category, "security");
    assert_eq!(compare_extension(&rep_pub, Some(OsStr::new("pub"))), ExtensionStatus::Match);

    // 6. Apache ORC Columnar Database (orc)
    let orc_data = b"ORC\x00\x00\x01\x00";
    let rep_orc = detect_bytes(orc_data, &opts).expect("detect orc");
    assert_eq!(rep_orc.format_id.as_deref(), Some("orc"));
    assert_eq!(rep_orc.category, "database");
    assert_eq!(compare_extension(&rep_orc, Some(OsStr::new("orc"))), ExtensionStatus::Match);

    // 7. Compressed ISO Image (isz)
    let isz_data = b"IsZ\x01\x00\x00\x00\x00";
    let rep_isz = detect_bytes(isz_data, &opts).expect("detect isz");
    assert_eq!(rep_isz.format_id.as_deref(), Some("isz"));
    assert_eq!(rep_isz.category, "disk");
    assert_eq!(compare_extension(&rep_isz, Some(OsStr::new("isz"))), ExtensionStatus::Match);

    // 8. WARC Web ARChive (warc)
    let warc_data = b"WARC/1.0\r\nWARC-Type: warcinfo\r\n";
    let rep_warc = detect_bytes(warc_data, &opts).expect("detect warc");
    assert_eq!(rep_warc.format_id.as_deref(), Some("warc"));
    assert_eq!(rep_warc.category, "document");
    assert_eq!(compare_extension(&rep_warc, Some(OsStr::new("warc"))), ExtensionStatus::Match);

    // 9. Adobe BDF Bitmap Distribution Font (bdf)
    let bdf_data = b"STARTFONT 2.1\nFONT Sample\n";
    let rep_bdf = detect_bytes(bdf_data, &opts).expect("detect bdf");
    assert_eq!(rep_bdf.format_id.as_deref(), Some("bdf"));
    assert_eq!(rep_bdf.category, "font");
    assert_eq!(compare_extension(&rep_bdf, Some(OsStr::new("bdf"))), ExtensionStatus::Match);

    // 10. Unreal Engine Asset Package (uasset / umap)
    let uasset_data = b"\xc1\x83\x2a\x9e\x00\x00\x00\x00";
    let rep_uasset = detect_bytes(uasset_data, &opts).expect("detect uasset");
    assert_eq!(rep_uasset.format_id.as_deref(), Some("uasset"));
    assert_eq!(rep_uasset.category, "game");
    assert_eq!(compare_extension(&rep_uasset, Some(OsStr::new("uasset"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_uasset, Some(OsStr::new("umap"))), ExtensionStatus::Match);

    // 11. Quite OK Image Format (qoi)
    let qoi_data = b"qoif\x00\x00\x01\x00\x00\x00\x01\x00\x04\x00";
    let rep_qoi = detect_bytes(qoi_data, &opts).expect("detect qoi");
    assert_eq!(rep_qoi.format_id.as_deref(), Some("qoi"));
    assert_eq!(rep_qoi.category, "image");
    assert_eq!(compare_extension(&rep_qoi, Some(OsStr::new("qoi"))), ExtensionStatus::Match);

    // 12. Dolby Digital AC-3 Audio (ac3)
    let ac3_data = b"\x0b\x77\x00\x00";
    let rep_ac3 = detect_bytes(ac3_data, &opts).expect("detect ac3");
    assert_eq!(rep_ac3.format_id.as_deref(), Some("ac3"));
    assert_eq!(rep_ac3.category, "audio");
    assert_eq!(compare_extension(&rep_ac3, Some(OsStr::new("ac3"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_batch_500_engineering_and_protocols() {
    let opts = DetectOptions::default();

    // 13. Stanford PLY 3D Model (ply)
    let ply_data = b"ply\nformat ascii 1.0\n";
    let rep_ply = detect_bytes(ply_data, &opts).expect("detect ply");
    assert_eq!(rep_ply.format_id.as_deref(), Some("ply"));
    assert_eq!(rep_ply.category, "model");
    assert_eq!(compare_extension(&rep_ply, Some(OsStr::new("ply"))), ExtensionStatus::Match);

    // 14. Scream Tracker 3 Music Module (s3m)
    let mut s3m_data = vec![0u8; 44];
    s3m_data.extend_from_slice(b"SCRM\x00\x00\x00\x00");
    let rep_s3m = detect_bytes(&s3m_data, &opts).expect("detect s3m");
    assert_eq!(rep_s3m.format_id.as_deref(), Some("s3m"));
    assert_eq!(rep_s3m.category, "audio");
    assert_eq!(compare_extension(&rep_s3m, Some(OsStr::new("s3m"))), ExtensionStatus::Match);

    // 15. Netpbm Portable Arbitrary Map (pam)
    let pam_data = b"P7\nWIDTH 10\nHEIGHT 10\n";
    let rep_pam = detect_bytes(pam_data, &opts).expect("detect pam");
    assert_eq!(rep_pam.format_id.as_deref(), Some("pam"));
    assert_eq!(rep_pam.category, "image");
    assert_eq!(compare_extension(&rep_pam, Some(OsStr::new("pam"))), ExtensionStatus::Match);

    // 16. HTTP Archive (har)
    let har_data = b"{\"log\":{\"version\":\"1.2\"}}";
    let rep_har = detect_bytes(har_data, &opts).expect("detect har");
    assert_eq!(rep_har.format_id.as_deref(), Some("har"));
    assert_eq!(rep_har.category, "web");
    assert_eq!(compare_extension(&rep_har, Some(OsStr::new("har"))), ExtensionStatus::Match);

    // 17. Synchronized Multimedia Integration Language (smil)
    let smil_data = b"<smil><head></head><body></body></smil>";
    let rep_smil = detect_bytes(smil_data, &opts).expect("detect smil");
    assert_eq!(rep_smil.format_id.as_deref(), Some("smil"));
    assert_eq!(rep_smil.category, "multimedia");
    assert_eq!(compare_extension(&rep_smil, Some(OsStr::new("smil"))), ExtensionStatus::Match);

    // 18. Unified Robot Description Format (urdf)
    let urdf_data = b"<robot name=\"test_robot\"><link name=\"base\"/></robot>";
    let rep_urdf = detect_bytes(urdf_data, &opts).expect("detect urdf");
    assert_eq!(rep_urdf.format_id.as_deref(), Some("urdf"));
    assert_eq!(rep_urdf.category, "robotics");
    assert_eq!(compare_extension(&rep_urdf, Some(OsStr::new("urdf"))), ExtensionStatus::Match);

    // 19. International Patching System (ips)
    let ips_data = b"PATCH\x00\x01\x00\x00";
    let rep_ips = detect_bytes(ips_data, &opts).expect("detect ips");
    assert_eq!(rep_ips.format_id.as_deref(), Some("ips"));
    assert_eq!(rep_ips.category, "rom");
    assert_eq!(compare_extension(&rep_ips, Some(OsStr::new("ips"))), ExtensionStatus::Match);

    // 20. Axon Binary File electrophysiology (abf)
    let abf_data = b"ABF \x00\x00\x01\x00";
    let rep_abf = detect_bytes(abf_data, &opts).expect("detect abf");
    assert_eq!(rep_abf.format_id.as_deref(), Some("abf"));
    assert_eq!(rep_abf.category, "scientific");
    assert_eq!(compare_extension(&rep_abf, Some(OsStr::new("abf"))), ExtensionStatus::Match);

    // 21. MicroDVD Subtitle (sub)
    let sub_data = b"{1}{1}00:00:01/Hello MicroDVD";
    let rep_sub = detect_bytes(sub_data, &opts).expect("detect sub");
    assert_eq!(rep_sub.format_id.as_deref(), Some("sub"));
    assert_eq!(rep_sub.category, "media");
    assert_eq!(compare_extension(&rep_sub, Some(OsStr::new("sub"))), ExtensionStatus::Match);

    // 22. Open Financial Exchange (ofx / qfx)
    let ofx_data = b"OFXHEADER:100\nDATA:OFXSGML\n";
    let rep_ofx = detect_bytes(ofx_data, &opts).expect("detect ofx");
    assert_eq!(rep_ofx.format_id.as_deref(), Some("ofx"));
    assert_eq!(rep_ofx.category, "exchange");
    assert_eq!(compare_extension(&rep_ofx, Some(OsStr::new("ofx"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ofx, Some(OsStr::new("qfx"))), ExtensionStatus::Match);

    // 23. Basis Universal GPU Texture (basis)
    let basis_data = b"sB\x01\x00\x00\x00\x00\x00";
    let rep_basis = detect_bytes(basis_data, &opts).expect("detect basis");
    assert_eq!(rep_basis.format_id.as_deref(), Some("basis"));
    assert_eq!(rep_basis.category, "texture");
    assert_eq!(compare_extension(&rep_basis, Some(OsStr::new("basis"))), ExtensionStatus::Match);

    // 24. AutoCAD Slide Vector Graphic (sld)
    let sld_data = b"AutoCAD Slide\r\n\x1a\x00\x00\x00";
    let rep_sld = detect_bytes(sld_data, &opts).expect("detect sld");
    assert_eq!(rep_sld.format_id.as_deref(), Some("sld"));
    assert_eq!(rep_sld.category, "image");
    assert_eq!(compare_extension(&rep_sld, Some(OsStr::new("sld"))), ExtensionStatus::Match);
}
