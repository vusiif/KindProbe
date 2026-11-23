use std::ffi::OsStr;
use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Verification};

#[test]
fn test_detect_subtitles_and_lyrics() {
    let opts = DetectOptions::default();

    // 1. WebVTT modern web video text tracks
    let vtt_data = b"WEBVTT\n\n00:00:01.000 --> 00:00:04.000\nHello World\n";
    let rep_vtt = detect_bytes(vtt_data, &opts).expect("detect vtt");
    assert_eq!(rep_vtt.identification, Identification::Format);
    assert_eq!(rep_vtt.format_id.as_deref(), Some("vtt"));
    assert_eq!(rep_vtt.verification, Verification::Signature);
    assert_eq!(compare_extension(&rep_vtt, Some(OsStr::new("vtt"))), ExtensionStatus::Match);

    // 2. Advanced SubStation Alpha (ASS & SSA)
    let ass_data = b"[Script Info]\nTitle: KindProbe Subtitle\nScriptType: v4.00+\n";
    let rep_ass = detect_bytes(ass_data, &opts).expect("detect ass");
    assert_eq!(rep_ass.format_id.as_deref(), Some("ass"));
    assert_eq!(compare_extension(&rep_ass, Some(OsStr::new("ass"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ass, Some(OsStr::new("ssa"))), ExtensionStatus::Match);

    // 3. SubRip Subtitle (SRT CRLF & LF)
    let srt_crlf = b"1\r\n00:00:01,000 --> 00:00:04,000\r\nSubtitle line 1\r\n\r\n";
    let rep_srt1 = detect_bytes(srt_crlf, &opts).expect("detect srt crlf");
    assert_eq!(rep_srt1.format_id.as_deref(), Some("srt"));
    assert_eq!(compare_extension(&rep_srt1, Some(OsStr::new("srt"))), ExtensionStatus::Match);

    let srt_lf = b"1\n00:00:01,000 --> 00:00:04,000\nSubtitle line 1\n\n";
    let rep_srt2 = detect_bytes(srt_lf, &opts).expect("detect srt lf");
    assert_eq!(rep_srt2.format_id.as_deref(), Some("srt"));
    assert_eq!(compare_extension(&rep_srt2, Some(OsStr::new("srt"))), ExtensionStatus::Match);

    // 4. SAMI Synchronized Accessible Media Interchange
    let sami_data = b"<SAMI>\n<HEAD><TITLE>KindProbe</TITLE></HEAD><BODY></BODY></SAMI>";
    let rep_smi = detect_bytes(sami_data, &opts).expect("detect sami");
    assert_eq!(rep_smi.format_id.as_deref(), Some("smi"));
    assert_eq!(compare_extension(&rep_smi, Some(OsStr::new("smi"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_smi, Some(OsStr::new("sami"))), ExtensionStatus::Match);

    // 5. LRC Synchronized Lyrics
    let lrc_data = b"[ti:Test Song]\n[ar:Artist]\n[00:01.00]Song lyrics line 1\n";
    let rep_lrc = detect_bytes(lrc_data, &opts).expect("detect lrc");
    assert_eq!(rep_lrc.format_id.as_deref(), Some("lrc"));
    assert_eq!(compare_extension(&rep_lrc, Some(OsStr::new("lrc"))), ExtensionStatus::Match);

    // 6. YouTube Subtitle (sbv)
    let sbv_data = b"0:00:01.000,0:00:04.000\nHello YouTube subtitle\n";
    let rep_sbv = detect_bytes(sbv_data, &opts).expect("detect sbv");
    assert_eq!(rep_sbv.format_id.as_deref(), Some("sbv"));
    assert_eq!(compare_extension(&rep_sbv, Some(OsStr::new("sbv"))), ExtensionStatus::Match);

    // 7. W3C Timed Text Markup Language (ttml / dfxp)
    let ttml_data = b"<tt xmlns=\"http://www.w3.org/ns/ttml\"><body><div><p>Subtitle</p></div></body></tt>";
    let rep_ttml = detect_bytes(ttml_data, &opts).expect("detect ttml");
    assert_eq!(rep_ttml.format_id.as_deref(), Some("ttml"));
    assert_eq!(compare_extension(&rep_ttml, Some(OsStr::new("ttml"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_ttml, Some(OsStr::new("dfxp"))), ExtensionStatus::Match);
}

#[test]
fn test_detect_cad_models_and_navigation() {
    let opts = DetectOptions::default();

    // 1. STL 3D Stereolithography mesh
    let stl_data = b"solid cube\n  facet normal 0 0 0\n  endfacet\nendsolid cube\n";
    let rep_stl = detect_bytes(stl_data, &opts).expect("detect stl");
    assert_eq!(rep_stl.format_id.as_deref(), Some("stl"));
    assert_eq!(compare_extension(&rep_stl, Some(OsStr::new("stl"))), ExtensionStatus::Match);

    // 2. VRML 3D Model
    let vrml_data = b"#VRML V2.0 utf8\nGroup {\n  children []\n}\n";
    let rep_vrml = detect_bytes(vrml_data, &opts).expect("detect vrml");
    assert_eq!(rep_vrml.format_id.as_deref(), Some("vrml"));
    assert_eq!(compare_extension(&rep_vrml, Some(OsStr::new("wrl"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_vrml, Some(OsStr::new("vrml"))), ExtensionStatus::Match);

    // 3. WebAssembly Text (WAT)
    let wat_data = b"(module\n  (func $main (result i32)\n    i32.const 42))\n";
    let rep_wat = detect_bytes(wat_data, &opts).expect("detect wat");
    assert_eq!(rep_wat.format_id.as_deref(), Some("wat"));
    assert_eq!(compare_extension(&rep_wat, Some(OsStr::new("wat"))), ExtensionStatus::Match);

    // 4. NMEA 0183 GPS Sentence
    let nmea_data = b"$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n";
    let rep_nmea = detect_bytes(nmea_data, &opts).expect("detect nmea");
    assert_eq!(rep_nmea.format_id.as_deref(), Some("nmea"));
    assert_eq!(compare_extension(&rep_nmea, Some(OsStr::new("nmea"))), ExtensionStatus::Match);
    assert_eq!(compare_extension(&rep_nmea, Some(OsStr::new("gps"))), ExtensionStatus::Match);

    // 5. BibTeX Reference
    let bib_data = b"@article{kindprobe2026,\n  author = {Deepmind},\n  title = {KindProbe Architecture}\n}\n";
    let rep_bib = detect_bytes(bib_data, &opts).expect("detect bib");
    assert_eq!(rep_bib.format_id.as_deref(), Some("bib"));
    assert_eq!(compare_extension(&rep_bib, Some(OsStr::new("bib"))), ExtensionStatus::Match);

    // 6. OpenStreetMap XML Geodata (osm)
    let osm_data = b"<osm version=\"0.6\" generator=\"KindProbe\"><node id=\"1\" lat=\"51.5\" lon=\"-0.1\"/></osm>";
    let rep_osm = detect_bytes(osm_data, &opts).expect("detect osm");
    assert_eq!(rep_osm.format_id.as_deref(), Some("osm"));
    assert_eq!(compare_extension(&rep_osm, Some(OsStr::new("osm"))), ExtensionStatus::Match);

    // 7. MapInfo Table (tab)
    let tab_data = b"!table\n!version 300\n!charset WindowsLatin1\nDefinition Table\n";
    let rep_tab = detect_bytes(tab_data, &opts).expect("detect tab");
    assert_eq!(rep_tab.format_id.as_deref(), Some("tab"));
    assert_eq!(compare_extension(&rep_tab, Some(OsStr::new("tab"))), ExtensionStatus::Match);

    // 8. TopoJSON Geospatial Topology (topojson)
    let topo_data = b"{\"type\":\"Topology\",\"objects\":{},\"arcs\":[]}";
    let rep_topo = detect_bytes(topo_data, &opts).expect("detect topojson");
    assert_eq!(rep_topo.format_id.as_deref(), Some("topojson"));
    assert_eq!(compare_extension(&rep_topo, Some(OsStr::new("topojson"))), ExtensionStatus::Match);
}
