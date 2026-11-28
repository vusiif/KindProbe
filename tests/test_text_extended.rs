use kindprobe::budget::DetectOptions;
use kindprobe::extension::compare_extension;
use kindprobe::{detect_bytes, ExtensionStatus, Identification, Inspection, Verification};
use std::ffi::OsStr;

#[test]
fn test_detect_json() {
    let opts = DetectOptions::default();

    // 1. JSON object with UTF-8 BOM and leading whitespace
    let mut json_obj = vec![0xEF, 0xBB, 0xBF];
    json_obj.extend_from_slice(b"   \n  {\"name\": \"kindprobe\", \"active\": true}\n");

    let rep = detect_bytes(&json_obj, &opts).expect("detect json object");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("json"));
    assert_eq!(rep.category, "text");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.inspection, Inspection::Complete);
    assert_eq!(rep.recommended_extension.as_deref(), Some("json"));
    assert_eq!(rep.accepted_extensions, vec!["json"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.text.json"));

    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("json"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("xml"))),
        ExtensionStatus::Mismatch
    );

    // 2. JSON array
    let json_arr = b"[1, 2, 3, {\"key\": \"value\"}]";
    let rep_arr = detect_bytes(json_arr, &opts).expect("detect json array");
    assert_eq!(rep_arr.format_id.as_deref(), Some("json"));
}

#[test]
fn test_detect_xml_and_html() {
    let opts = DetectOptions::default();

    // 1. XML document
    let xml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root><item id=\"1\"/></root>";
    let rep_xml = detect_bytes(xml, &opts).expect("detect xml");
    assert_eq!(rep_xml.identification, Identification::Format);
    assert_eq!(rep_xml.format_id.as_deref(), Some("xml"));
    assert_eq!(rep_xml.category, "text");
    assert_eq!(rep_xml.verification, Verification::Structure);
    assert!(rep_xml.evidence.iter().any(|e| e.code == "sig.text.xml"));
    assert_eq!(
        compare_extension(&rep_xml, Some(OsStr::new("xml"))),
        ExtensionStatus::Match
    );

    // 2. HTML document
    let html = b"<!DOCTYPE html>\n<html><head><title>KindProbe</title></head><body><h1>Hello</h1></body></html>";
    let rep_html = detect_bytes(html, &opts).expect("detect html");
    assert_eq!(rep_html.identification, Identification::Format);
    assert_eq!(rep_html.format_id.as_deref(), Some("html"));
    assert_eq!(rep_html.category, "text");
    assert_eq!(rep_html.verification, Verification::Structure);
    assert!(rep_html.evidence.iter().any(|e| e.code == "sig.text.html"));
    assert_eq!(
        compare_extension(&rep_html, Some(OsStr::new("html"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_html, Some(OsStr::new("htm"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_html, Some(OsStr::new("json"))),
        ExtensionStatus::Mismatch
    );
}

#[test]
fn test_detect_config_yaml_and_toml() {
    let opts = DetectOptions::default();

    // 1. YAML document with document separator "---"
    let yaml = b"---\nname: kindprobe\nversion: 1.0\nenabled: true\n";
    let rep_yaml = detect_bytes(yaml, &opts).expect("detect yaml");
    assert_eq!(rep_yaml.identification, Identification::Format);
    assert_eq!(rep_yaml.format_id.as_deref(), Some("yaml"));
    assert_eq!(rep_yaml.category, "text");
    assert_eq!(rep_yaml.verification, Verification::Structure);
    assert!(rep_yaml.evidence.iter().any(|e| e.code == "sig.text.yaml"));
    assert_eq!(
        compare_extension(&rep_yaml, Some(OsStr::new("yaml"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_yaml, Some(OsStr::new("yml"))),
        ExtensionStatus::Match
    );

    // 2. TOML document with [section]
    let toml = b"[package]\nname = \"kindprobe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    let rep_toml = detect_bytes(toml, &opts).expect("detect toml");
    assert_eq!(rep_toml.identification, Identification::Format);
    assert_eq!(rep_toml.format_id.as_deref(), Some("toml"));
    assert_eq!(rep_toml.category, "text");
    assert_eq!(rep_toml.verification, Verification::Structure);
    assert!(rep_toml.evidence.iter().any(|e| e.code == "sig.text.toml"));
    assert_eq!(
        compare_extension(&rep_toml, Some(OsStr::new("toml"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_csv_and_tsv() {
    let opts = DetectOptions::default();

    // 1. CSV (3 consistent columns)
    let csv = b"id,name,role\n1,Alice,Engineer\n2,Bob,Architect\n3,Charlie,Tester\n";
    let rep_csv = detect_bytes(csv, &opts).expect("detect csv");
    assert_eq!(rep_csv.identification, Identification::Format);
    assert_eq!(rep_csv.format_id.as_deref(), Some("csv"));
    assert_eq!(rep_csv.category, "text");
    assert_eq!(rep_csv.verification, Verification::Structure);
    assert!(rep_csv.evidence.iter().any(|e| e.code == "sig.text.csv"));
    assert_eq!(
        compare_extension(&rep_csv, Some(OsStr::new("csv"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_csv, Some(OsStr::new("tsv"))),
        ExtensionStatus::Mismatch
    );

    // 2. TSV (3 consistent tab-separated columns)
    let tsv = b"id\tname\trole\n1\tAlice\tEngineer\n2\tBob\tArchitect\n";
    let rep_tsv = detect_bytes(tsv, &opts).expect("detect tsv");
    assert_eq!(rep_tsv.identification, Identification::Format);
    assert_eq!(rep_tsv.format_id.as_deref(), Some("tsv"));
    assert_eq!(rep_tsv.category, "text");
    assert_eq!(rep_tsv.verification, Verification::Structure);
    assert!(rep_tsv.evidence.iter().any(|e| e.code == "sig.text.tsv"));
    assert_eq!(
        compare_extension(&rep_tsv, Some(OsStr::new("tsv"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep_tsv, Some(OsStr::new("tab"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_detect_plain_text() {
    let opts = DetectOptions::default();

    let plain = b"This is just a simple plain text message without any structured syntax.\nIt has multiple sentences.";
    let rep = detect_bytes(plain, &opts).expect("detect plain text");
    assert_eq!(rep.identification, Identification::Format);
    assert_eq!(rep.format_id.as_deref(), Some("txt"));
    assert_eq!(rep.category, "text");
    assert_eq!(rep.verification, Verification::Structure);
    assert_eq!(rep.recommended_extension.as_deref(), Some("txt"));
    assert_eq!(rep.accepted_extensions, vec!["txt", "text", "log"]);
    assert!(rep.evidence.iter().any(|e| e.code == "sig.text.plain"));

    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("txt"))),
        ExtensionStatus::Match
    );
    assert_eq!(
        compare_extension(&rep, Some(OsStr::new("log"))),
        ExtensionStatus::Match
    );
}

#[test]
fn test_binary_not_falsely_detected_as_text() {
    let opts = DetectOptions::default();

    // Binary data with null bytes
    let bin = b"Some random ASCII looking prefix \x00\x01\x02\xFF\xFE other junk";
    let rep = detect_bytes(bin, &opts).expect("detect binary");
    assert_eq!(rep.identification, Identification::Unknown);
    assert!(rep.warnings.iter().any(|w| w.code == "format.unknown"));
}

#[test]
fn test_text_heuristic_and_short_pattern_false_positive_fixes() {
    let opts = DetectOptions::default();

    // 1. Markdown containing C code snippet must stay md
    let md_with_c = b"# Build Guide\n\n```c\n#include <stdio.h>\n```\n";
    let rep_md = detect_bytes(md_with_c, &opts).unwrap();
    assert_eq!(rep_md.format_id.as_deref(), Some("md"));

    // 2. [TOC] without key=value must not become toml
    let toc_txt = b"[TOC]\n\nSection overview without assignments\n";
    let rep_toc = detect_bytes(toc_txt, &opts).unwrap();
    assert_eq!(rep_toc.format_id.as_deref(), Some("txt"));

    // 3. Markdown with YAML frontmatter must not stay stuck as yaml
    let md_frontmatter = b"---\ntitle: Guide\n---\n\n# Heading\n\n```bash\necho ok\n```\n";
    let rep_fm = detect_bytes(md_frontmatter, &opts).unwrap();
    assert_ne!(rep_fm.format_id.as_deref(), Some("yaml"));

    // 4. Plain XML root element without prolog/xmlns must detect as xml
    let custom_xml = b"<customRoot><child id=\"1\"/></customRoot>";
    let rep_xml = detect_bytes(custom_xml, &opts).unwrap();
    assert_eq!(rep_xml.format_id.as_deref(), Some("xml"));

    // 5. English text starting with S0/S1/S3 or Cr must not falsely match srec/crn
    let rep_s1_text = detect_bytes(b"S1: Introduction and overview of data\n", &opts).unwrap();
    assert_eq!(rep_s1_text.format_id.as_deref(), Some("txt"));

    let rep_cr_text =
        detect_bytes(b"Create a new configuration file before starting.\n", &opts).unwrap();
    assert_eq!(rep_cr_text.format_id.as_deref(), Some("txt"));

    // 6. Plain ASCII text starting with MM / BD / :U / offset-8 LP must not hit 3ds / hfv / sg2 / eot
    for sample in [
        b"MM: Meeting minutes and action items\n".as_slice(),
        b"BDD scenario description for authentication\n".as_slice(),
        b":User documentation notes on protocols\n".as_slice(),
        b"12345678LP plain ASCII content line\n".as_slice(),
        b"850ms latency observed in eu-west\n".as_slice(),
        b"ORC columnar storage overview notes\n".as_slice(),
        b":10:30 meeting scheduled with team\n".as_slice(),
        b"SMK toolkit documentation notes\n".as_slice(),
        b"$GPS_MODE=high precision config\n".as_slice(),
        b"BIKING trail route summary notes\n".as_slice(),
        b"GXT parser design documentation\n".as_slice(),
        b"%FS placeholder text note here\n".as_slice(),
    ] {
        let rep = detect_bytes(sample, &opts).unwrap();
        assert_eq!(
            rep.format_id.as_deref(),
            Some("txt"),
            "unexpected match for {:?}",
            std::str::from_utf8(sample)
        );
    }

    // C header starting with #define must not be shadowed by XBM image rule
    let c_hdr = b"#define KINDPROBE_LIMIT 128\n#include <stdint.h>\nint add(int a, int b) { return a + b; }\n";
    let rep_c = detect_bytes(c_hdr, &opts).unwrap();
    assert_eq!(rep_c.format_id.as_deref(), Some("c"));
}
