use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Verification};

#[test]
fn test_detect_astronomy_and_space() {
    let opts = DetectOptions::default();

    // 1. SPICE SPK Ephemeris (spk)
    let spk_data = b"DAFT/SPK \x00\x00\x00\x00";
    let rep = detect_bytes(spk_data, &opts).expect("detect spk");
    assert_eq!(rep.format_id.as_deref(), Some("spk"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(rep.verification, Verification::Signature);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("spk"))), ExtensionStatus::Match);

    // 2. Advanced Scientific Data Format (asdf)
    let asdf_data = b"#ASDF 1.0.0\n#ASDF_STANDARD 1.5.0\n";
    let rep = detect_bytes(asdf_data, &opts).expect("detect asdf");
    assert_eq!(rep.format_id.as_deref(), Some("asdf"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("asdf"))), ExtensionStatus::Match);

    // 3. VOTable XML (votable / vot)
    let vot_data = b"<VOTABLE version=\"1.4\"><RESOURCE></RESOURCE></VOTABLE>";
    let rep = detect_bytes(vot_data, &opts).expect("detect votable");
    assert_eq!(rep.format_id.as_deref(), Some("votable"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("vot"))), ExtensionStatus::Match);

    // 4. Two-Line Element (tle)
    let tle_data = b"1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\n";
    let rep = detect_bytes(tle_data, &opts).expect("detect tle");
    assert_eq!(rep.format_id.as_deref(), Some("tle"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("tle"))), ExtensionStatus::Match);

    // 5. Orbit Ephemeris Message (oem)
    let oem_data = b"CCSDS_OEM_VERS = 2.0\nCREATION_DATE = 2026-11-23\n";
    let rep = detect_bytes(oem_data, &opts).expect("detect oem");
    assert_eq!(rep.format_id.as_deref(), Some("oem"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("oem"))), ExtensionStatus::Match);

    // 6. Attitude Ephemeris Message (aem)
    let aem_data = b"CCSDS_AEM_VERS = 1.0\nCREATION_DATE = 2026-11-23\n";
    let rep = detect_bytes(aem_data, &opts).expect("detect aem");
    assert_eq!(rep.format_id.as_deref(), Some("aem"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("aem"))), ExtensionStatus::Match);

    // 7. RINEX Observation (rnx)
    let mut rnx_data = vec![b' '; 80];
    rnx_data[20..36].copy_from_slice(b"OBSERVATION DATA");
    let rep = detect_bytes(&rnx_data, &opts).expect("detect rinex_obs");
    assert_eq!(rep.format_id.as_deref(), Some("rinex_obs"));
    assert_eq!(rep.category, "scientific");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("rnx"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_video_streams_and_rf() {
    let opts = DetectOptions::default();

    // 1. H.265 / HEVC Stream (h265 / hevc)
    let h265_data = b"\x00\x00\x00\x01\x40\x01\x0c\x01\xff\xff";
    let rep = detect_bytes(h265_data, &opts).expect("detect h265");
    assert_eq!(rep.format_id.as_deref(), Some("h265"));
    assert_eq!(rep.category, "video");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("h265"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("hevc"))), ExtensionStatus::Match);

    // 2. H.264 / AVC Stream (h264 / 264)
    let h264_data = b"\x00\x00\x00\x01\x67\x42\x00\x1f\xe9\x01";
    let rep = detect_bytes(h264_data, &opts).expect("detect h264");
    assert_eq!(rep.format_id.as_deref(), Some("h264"));
    assert_eq!(rep.category, "video");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("h264"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep, Some(OsStr::new("264"))), ExtensionStatus::Match);

    // 3. SMPTE VC-1 Stream (vc1)
    let vc1_data = b"\x00\x00\x01\x0f\xc5\x00\x00\x00";
    let rep = detect_bytes(vc1_data, &opts).expect("detect vc1");
    assert_eq!(rep.format_id.as_deref(), Some("vc1"));
    assert_eq!(rep.category, "video");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("vc1"))), ExtensionStatus::Match);

    // 4. Dolby AC-4 Audio (ac4)
    let ac4_data = b"\xac\x40\xff\xff\x00\x00";
    let rep = detect_bytes(ac4_data, &opts).expect("detect ac4");
    assert_eq!(rep.format_id.as_deref(), Some("ac4"));
    assert_eq!(rep.category, "audio");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("ac4"))), ExtensionStatus::Match);

    // 5. AVS / AVS+ Stream (avs)
    let avs_data = b"\x00\x00\x01\xb0\x01\x00\x00\x00";
    let rep = detect_bytes(avs_data, &opts).expect("detect avs");
    assert_eq!(rep.format_id.as_deref(), Some("avs"));
    assert_eq!(rep.category, "video");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("avs"))), ExtensionStatus::Match);

    // 6. Sony Wave64 (w64)
    let w64_data = b"riff\x2e\x91\xcf\x11\xa5\xd6\x28\xdb\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep = detect_bytes(w64_data, &opts).expect("detect w64");
    assert_eq!(rep.format_id.as_deref(), Some("w64"));
    assert_eq!(rep.category, "audio");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("w64"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_geospatial_vector_cad() {
    let opts = DetectOptions::default();

    // 1. OGC GML (gml)
    let gml_data = b"<gml:FeatureCollection xmlns:gml=\"http://www.opengis.net/gml\"></gml:FeatureCollection>";
    let rep = detect_bytes(gml_data, &opts).expect("detect gml");
    assert_eq!(rep.format_id.as_deref(), Some("gml"));
    assert_eq!(rep.category, "gis");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("gml"))), ExtensionStatus::Match);

    // 2. OpenStreetMap PBF (pbf)
    let pbf_data = b"\x00\x00\x00\x0dOSMHeader\x08\x00\x00\x00\x00";
    let rep = detect_bytes(pbf_data, &opts).expect("detect pbf");
    assert_eq!(rep.format_id.as_deref(), Some("pbf"));
    assert_eq!(rep.category, "gis");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("pbf"))), ExtensionStatus::Match);

    // 3. MapInfo Workspace (wor)
    let wor_data = b"!workspace\n!version 300\n";
    let rep = detect_bytes(wor_data, &opts).expect("detect wor");
    assert_eq!(rep.format_id.as_deref(), Some("wor"));
    assert_eq!(rep.category, "gis");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("wor"))), ExtensionStatus::Match);

    // 4. Bentley MicroStation DGN (dgn)
    let dgn_data = b"\x08\x05\x19\x00\x00\x00\x00\x00";
    let rep = detect_bytes(dgn_data, &opts).expect("detect dgn");
    assert_eq!(rep.format_id.as_deref(), Some("dgn"));
    assert_eq!(rep.category, "cad");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("dgn"))), ExtensionStatus::Match);

    // 5. LandXML (landxml)
    let land_data = b"<LandXML date=\"2026-11-23\"></LandXML>";
    let rep = detect_bytes(land_data, &opts).expect("detect landxml");
    assert_eq!(rep.format_id.as_deref(), Some("landxml"));
    assert_eq!(rep.category, "cad");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("landxml"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_fonts_and_embedded_rtos() {
    let opts = DetectOptions::default();

    // 1. FontForge SplineFontDB (sfd)
    let sfd_data = b"SplineFontDB: 3.2\nFontName: Test\n";
    let rep = detect_bytes(sfd_data, &opts).expect("detect sfd");
    assert_eq!(rep.format_id.as_deref(), Some("sfd"));
    assert_eq!(rep.category, "font");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("sfd"))), ExtensionStatus::Match);

    // 2. Microsoft UF2 (uf2)
    let uf2_data = b"\x55\x46\x32\x0a\x57\x51\x5d\x9e\x00\x00\x00\x00";
    let rep = detect_bytes(uf2_data, &opts).expect("detect uf2");
    assert_eq!(rep.format_id.as_deref(), Some("uf2"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("uf2"))), ExtensionStatus::Match);

    // 3. Espressif ESP32 Image (esp32)
    let esp_data = b"\xe9\x01\x02\x00\x00\x00\x00\x00";
    let rep = detect_bytes(esp_data, &opts).expect("detect esp32_img");
    assert_eq!(rep.format_id.as_deref(), Some("esp32_img"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("esp32"))), ExtensionStatus::Match);

    // 4. TI-TXT MSP430 Firmware (titxt)
    let ti_data = b"@F000\r\n31 40 00 03\r\nq\r\n";
    let rep = detect_bytes(ti_data, &opts).expect("detect ti_txt");
    assert_eq!(rep.format_id.as_deref(), Some("ti_txt"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("titxt"))), ExtensionStatus::Match);

    // 5. Allwinner eGON Bootloader (egon)
    let egon_data = b"\x00\x00\x00\x00eGON.BT0\x00\x00\x00\x00";
    let rep = detect_bytes(egon_data, &opts).expect("detect allwinner_egon");
    assert_eq!(rep.format_id.as_deref(), Some("allwinner_egon"));
    assert_eq!(rep.category, "firmware");
    assert_eq!(compare_extension(&rep, Some(OsStr::new("egon"))), ExtensionStatus::Match);
}
