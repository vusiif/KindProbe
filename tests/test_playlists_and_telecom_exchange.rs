use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};

#[test]
fn test_detect_playlists_and_cue() {
    let opts = DetectOptions::default();

    // 1. M3U Playlist
    let m3u_data = b"#EXTM3U\n#EXTINF:123,Sample Song\nsample.mp3\n";
    let rep_m3u = detect_bytes(m3u_data, &opts).expect("detect m3u");
    assert_eq!(rep_m3u.identification, Identification::Format);
    assert_eq!(rep_m3u.format_id.as_deref(), Some("m3u"));
    assert_eq!(rep_m3u.verification, Verification::Signature);
    assert_eq!(compare_extension(&rep_m3u, Some(OsStr::new("m3u"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_m3u, Some(OsStr::new("m3u8"))), ExtensionStatus::Match);

    // 2. PLS Playlist
    let pls_data = b"[playlist]\nNumberOfEntries=1\nFile1=http://stream.example.com\n";
    let rep_pls = detect_bytes(pls_data, &opts).expect("detect pls");
    assert_eq!(rep_pls.format_id.as_deref(), Some("pls"));
    assert_eq!(compare_extension(&rep_pls, Some(OsStr::new("pls"))), ExtensionStatus::Match);

    // 3. CUE Sheet
    let cue_data = b"FILE \"album.wav\" WAVE\n  TRACK 01 AUDIO\n    TITLE \"Track 1\"\n";
    let rep_cue = detect_bytes(cue_data, &opts).expect("detect cue");
    assert_eq!(rep_cue.format_id.as_deref(), Some("cue"));
    assert_eq!(compare_extension(&rep_cue, Some(OsStr::new("cue"))), ExtensionStatus::Match);

    // 4. WPL Windows Media Playlist
    let wpl_data = b"<?wpl version=\"1.0\"?>\n<smil><head></head><body></body></smil>";
    let rep_wpl = detect_bytes(wpl_data, &opts).expect("detect wpl");
    assert_eq!(rep_wpl.format_id.as_deref(), Some("wpl"));
    assert_eq!(compare_extension(&rep_wpl, Some(OsStr::new("wpl"))), ExtensionStatus::Match);

    // 5. XSPF XML Shareable Playlist
    let xspf_data = b"<playlist version=\"1\" xmlns=\"http://xspf.org/ns/0/\">\n<trackList/>\n</playlist>";
    let rep_xspf = detect_bytes(xspf_data, &opts).expect("detect xspf");
    assert_eq!(rep_xspf.format_id.as_deref(), Some("xspf"));
    assert_eq!(compare_extension(&rep_xspf, Some(OsStr::new("xspf"))), ExtensionStatus::Match);

    // 6. ASX Advanced Stream Redirector
    let asx_data = b"<asx version=\"3.0\">\n<entry><ref href=\"http://test.asx\"/></entry></asx>";
    let rep_asx = detect_bytes(asx_data, &opts).expect("detect asx");
    assert_eq!(rep_asx.format_id.as_deref(), Some("asx"));
    assert_eq!(compare_extension(&rep_asx, Some(OsStr::new("asx"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_asx, Some(OsStr::new("wax"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_asx, Some(OsStr::new("wvx"))), ExtensionStatus::Match);

    // 7. B4S Winamp Playlist
    let b4s_data = b"Winamp XML Playlist 1.0\n<playlist></playlist>";
    let rep_b4s = detect_bytes(b4s_data, &opts).expect("detect b4s");
    assert_eq!(rep_b4s.format_id.as_deref(), Some("b4s"));
    assert_eq!(compare_extension(&rep_b4s, Some(OsStr::new("b4s"))), ExtensionStatus::Match);

    // 8. JSPF JSON Shareable Playlist
    let jspf_data = b"{\"playlist\":{\"title\":\"Sample Playlist\",\"track\":[]}}";
    let rep_jspf = detect_bytes(jspf_data, &opts).expect("detect jspf");
    assert_eq!(rep_jspf.format_id.as_deref(), Some("jspf"));
    assert_eq!(compare_extension(&rep_jspf, Some(OsStr::new("jspf"))), ExtensionStatus::Match);

    // 9. Zune XML Playlist (zpl)
    let zpl_data = b"<?zpl version=\"2.0\"?>\n<smil><head></head><body><seq></seq></body></smil>";
    let rep_zpl = detect_bytes(zpl_data, &opts).expect("detect zpl");
    assert_eq!(rep_zpl.format_id.as_deref(), Some("zpl"));
    assert_eq!(compare_extension(&rep_zpl, Some(OsStr::new("zpl"))), ExtensionStatus::Match);

    // 10. AIMP Playlist (aimppl)
    let aimp_data = b"AIMP30\x00\x01\x00\x00Sample AIMP Playlist";
    let rep_aimp = detect_bytes(aimp_data, &opts).expect("detect aimppl");
    assert_eq!(rep_aimp.format_id.as_deref(), Some("aimppl"));
    assert_eq!(compare_extension(&rep_aimp, Some(OsStr::new("aimppl"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_telecom_and_firmware_exchange() {
    let opts = DetectOptions::default();

    // 1. Apple Binary Property List (bplist)
    let bplist_data = b"bplist00\xd1\x01\x02\x54root\x08\x0b\x00\x00";
    let rep_bp = detect_bytes(bplist_data, &opts).expect("detect bplist");
    assert_eq!(rep_bp.format_id.as_deref(), Some("bplist"));
    assert_eq!(compare_extension(&rep_bp, Some(OsStr::new("bplist"))), ExtensionStatus::Match);

    // 2. Motorola S-Record Header (srec S0)
    let srec_s0 = b"S00F000068656C6C6F202020202000004C\n";
    let rep_s0 = detect_bytes(srec_s0, &opts).expect("detect srec S0");
    assert_eq!(rep_s0.format_id.as_deref(), Some("srec"));
    assert_eq!(compare_extension(&rep_s0, Some(OsStr::new("srec"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_s0, Some(OsStr::new("s19"))), ExtensionStatus::Match);

    // 3. Motorola S-Record Data (srec S1 & S3)
    let srec_s1 = b"S1130000285C000000000000000000000000000066\n";
    let rep_s1 = detect_bytes(srec_s1, &opts).expect("detect srec S1");
    assert_eq!(rep_s1.format_id.as_deref(), Some("srec"));
    assert_eq!(compare_extension(&rep_s1, Some(OsStr::new("s28"))), ExtensionStatus::Match);

    let srec_s3 = b"S31500000000285C000000000000000000000000000062\n";
    let rep_s3 = detect_bytes(srec_s3, &opts).expect("detect srec S3");
    assert_eq!(rep_s3.format_id.as_deref(), Some("srec"));
    assert_eq!(compare_extension(&rep_s3, Some(OsStr::new("s37"))), ExtensionStatus::Match);

    // 4. HDF4 Scientific Dataset
    let hdf4_data = b"\x0e\x03\x13\x01\x00\x00\x00\x00\x00\x00\x00\x00";
    let rep_h4 = detect_bytes(hdf4_data, &opts).expect("detect hdf4");
    assert_eq!(rep_h4.format_id.as_deref(), Some("hdf4"));
    assert_eq!(compare_extension(&rep_h4, Some(OsStr::new("hdf4"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_h4, Some(OsStr::new("h4"))), ExtensionStatus::Match);

    // 5. vCard Virtual Contact (vcf)
    let vcf_data = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:KindProbe User\r\nEND:VCARD\r\n";
    let rep_vcf = detect_bytes(vcf_data, &opts).expect("detect vcf");
    assert_eq!(rep_vcf.format_id.as_deref(), Some("vcf"));
    assert_eq!(compare_extension(&rep_vcf, Some(OsStr::new("vcf"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_vcf, Some(OsStr::new("vcard"))), ExtensionStatus::Match);

    // 6. iCalendar Schedule (ics)
    let ics_data = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//KindProbe//EN\r\nEND:VCALENDAR\r\n";
    let rep_ics = detect_bytes(ics_data, &opts).expect("detect ics");
    assert_eq!(rep_ics.format_id.as_deref(), Some("ics"));
    assert_eq!(compare_extension(&rep_ics, Some(OsStr::new("ics"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ics, Some(OsStr::new("ical"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ics, Some(OsStr::new("ifb"))), ExtensionStatus::Match);

    // 7. LDIF LDAP Data Interchange
    let ldif_data = b"version: 1\ndn: cn=KindProbe, dc=example, dc=com\n";
    let rep_ldif = detect_bytes(ldif_data, &opts).expect("detect ldif");
    assert_eq!(rep_ldif.format_id.as_deref(), Some("ldif"));
    assert_eq!(compare_extension(&rep_ldif, Some(OsStr::new("ldif"))), ExtensionStatus::Match);

    // 8. Intel HEX (ihex)
    let ihex_data = b":10010000214601360121470136007EFE09D2190140\n:00000001FF\n";
    let rep_ihex = detect_bytes(ihex_data, &opts).expect("detect ihex");
    assert_eq!(rep_ihex.format_id.as_deref(), Some("ihex"));
    assert_eq!(compare_extension(&rep_ihex, Some(OsStr::new("ihex"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ihex, Some(OsStr::new("ihx"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ihex, Some(OsStr::new("hex"))), ExtensionStatus::Match);

    // 9. Quicken Interchange Format (qif)
    let qif_data = b"!Type:Bank\nD03/03/2026\nT-100.00\nPKindProbe Store\n^\n";
    let rep_qif = detect_bytes(qif_data, &opts).expect("detect qif");
    assert_eq!(rep_qif.format_id.as_deref(), Some("qif"));
    assert_eq!(compare_extension(&rep_qif, Some(OsStr::new("qif"))), ExtensionStatus::Match);

    // 10. GEDCOM FamilySearch Genealogy Data (ged)
    let ged_data = b"0 HEAD\n1 SOUR KindProbe\n1 GEDC\n2 VERS 5.5.1\n";
    let rep_ged = detect_bytes(ged_data, &opts).expect("detect gedcom");
    assert_eq!(rep_ged.format_id.as_deref(), Some("gedcom"));
    assert_eq!(compare_extension(&rep_ged, Some(OsStr::new("ged"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ged, Some(OsStr::new("gedcom"))), ExtensionStatus::Match);

    // 11. UN/EDIFACT Electronic Data Interchange (edifact / edi)
    let edi_data = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+261123:0000+1'";
    let rep_edi = detect_bytes(edi_data, &opts).expect("detect edifact");
    assert_eq!(rep_edi.format_id.as_deref(), Some("edifact"));
    assert_eq!(compare_extension(&rep_edi, Some(OsStr::new("edi"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_edi, Some(OsStr::new("edifact"))), ExtensionStatus::Match);
}
