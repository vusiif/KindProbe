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
    fs::write(
        sub.join("d.bin"),
        [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02],
    )
    .expect("write bin");

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

    let formats: Vec<Option<&str>> = rec_lines.iter().map(|v| v["format_id"].as_str()).collect();
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

#[test]
fn test_phase20_parallel_batch_80_files_jsonl() {
    let root = make_temp_dir("parallel_80");
    for i in 0..80 {
        match i % 4 {
            0 => fs::write(
                root.join(format!("f_{:03}.png", i)),
                b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4",
            )
            .expect("png"),
            1 => fs::write(root.join(format!("f_{:03}.json", i)), br#"{"ok":true}"#).expect("json"),
            2 => fs::write(root.join(format!("f_{:03}.py", i)), b"#!/usr/bin/env python3\n").expect("py"),
            _ => fs::write(root.join(format!("f_{:03}.bin", i)), [0xAA, 0xBB, 0x00, 0xCC]).expect("bin"),
        }
    }

    let out = Command::new(bin_path())
        .args(["--batch", "-j", "4", "--jsonl"])
        .arg(&root)
        .output()
        .expect("run parallel batch");
    assert_eq!(out.status.code(), Some(0));
    let lines: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("valid jsonl"))
        .collect();
    assert_eq!(lines.len(), 80);
    assert_eq!(lines.iter().filter(|v| v["format_id"] == "png").count(), 20);
    assert_eq!(
        lines.iter().filter(|v| v["format_id"] == "json").count(),
        20
    );
    assert_eq!(lines.iter().filter(|v| v["format_id"] == "py").count(), 20);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_batch_invalid_jobs_and_chinese_output() {
    let root = make_temp_dir("zh_batch");
    fs::write(
        root.join("sample.png"),
        b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4",
    )
    .unwrap();

    // 1. -j 0 must fail cleanly naming the -j option
    let out_j0 = Command::new(bin_path())
        .args(["--json", "--batch", "-j", "0"])
        .arg(&root)
        .output()
        .expect("run -j 0");
    assert_eq!(out_j0.status.code(), Some(2));
    let j0_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out_j0.stdout).trim()).unwrap();
    assert_eq!(j0_json["error"]["code"], "error.invalid_arguments");
    assert!(j0_json["error"]["message"].as_str().unwrap().contains("-j"));

    // 2. Chinese text output in batch mode should localize category and summary
    let out_zh = Command::new(bin_path())
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_WIN_LANGID", "2052")
        .args(["--batch"])
        .arg(&root)
        .output()
        .expect("run zh batch");
    assert_eq!(out_zh.status.code(), Some(0));
    let stdout_zh = String::from_utf8_lossy(&out_zh.stdout);
    let stderr_zh = String::from_utf8_lossy(&out_zh.stderr);
    assert!(stdout_zh.contains("png (图像)"));
    assert!(stderr_zh.contains("已扫描 1 个文件"));

    let _ = fs::remove_dir_all(&root);
}
