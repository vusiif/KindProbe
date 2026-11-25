use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_kindprobe"))
}

fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kindprobe_p19_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn test_phase19_batch_recursive_jsonl() {
    let root = make_temp_dir("batch_rec");
    let sub = root.join("nested");
    fs::create_dir_all(&sub).expect("create sub");

    fs::write(root.join("a.png"), b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4").expect("write png");
    fs::write(root.join("b.json"), br#"{"hello":"world","ok":true}"#).expect("write json");
    fs::write(sub.join("c.py"), b"#!/usr/bin/env python3\nprint('hi')\n").expect("write py");
    fs::write(sub.join("d.bin"), [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02]).expect("write bin");

    // Non-recursive --batch <root> --jsonl should see a.png & b.json only (2 files)
    let out_flat = Command::new(bin_path())
        .args(["--batch", "--jsonl"])
        .arg(&root)
        .output()
        .expect("run flat batch");
    assert_eq!(out_flat.status.code(), Some(0));
    let flat_lines: Vec<serde_json::Value> = String::from_utf8_lossy(&out_flat.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("valid jsonl"))
        .collect();
    assert_eq!(flat_lines.len(), 2);

    // Recursive -r --jsonl should see all 4 files
    let out_rec = Command::new(bin_path())
        .args(["-r", "--jsonl"])
        .arg(&root)
        .output()
        .expect("run recursive batch");
    assert_eq!(out_rec.status.code(), Some(0));
    let rec_lines: Vec<serde_json::Value> = String::from_utf8_lossy(&out_rec.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("valid jsonl"))
        .collect();
    assert_eq!(rec_lines.len(), 4);

    let formats: Vec<Option<&str>> = rec_lines
        .iter()
        .map(|v| v["format_id"].as_str())
        .collect();
    assert!(formats.contains(&Some("png")));
    assert!(formats.contains(&Some("json")));
    assert!(formats.contains(&Some("py")));
    assert!(formats.contains(&None));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_phase19_batch_missing_path_returns_exit_4() {
    let missing = std::env::temp_dir().join("kindprobe_p19_definitely_missing_123456");
    let out = Command::new(bin_path())
        .args(["--batch", "--jsonl"])
        .arg(&missing)
        .output()
        .expect("run missing batch");
    assert_eq!(out.status.code(), Some(4));
    let line = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(line.trim()).expect("valid json error");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error_code"], "io.not_found");
}
