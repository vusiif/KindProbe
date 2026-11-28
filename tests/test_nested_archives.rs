use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::process::Command;

use kindprobe::budget::DetectOptions;
use kindprobe::detect_bytes;
use kindprobe::model::{DetectionReport, ExtensionStatus, Identification, Verification};

const BIN_PATH: &str = env!("CARGO_BIN_EXE_kindprobe");

fn make_dummy_tar_block() -> Vec<u8> {
    let mut block = vec![0u8; 512];
    // File name at 0..100
    block[0..13].copy_from_slice(b"test_file.txt");
    // File mode at 100..108
    block[100..107].copy_from_slice(b"0000644");
    // Size at 124..136
    block[124..135].copy_from_slice(b"00000000000");
    // Magic at 257..263
    block[257..263].copy_from_slice(b"ustar\0");
    // Version at 263..265
    block[263..265].copy_from_slice(b"00");

    // Calculate checksum treating 148..156 as spaces (0x20)
    let mut sum: u32 = 0;
    for (i, &b) in block.iter().enumerate() {
        sum += if (148..156).contains(&i) {
            0x20
        } else {
            b as u32
        };
    }
    let chk_str = format!("{:06o}\0 ", sum);
    block[148..156].copy_from_slice(chk_str.as_bytes());
    block
}

fn create_tar_gz_bytes() -> Vec<u8> {
    let tar_block = make_dummy_tar_block();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_block).expect("write tar block");
    encoder.finish().expect("finish gzip")
}

fn create_plain_gz_bytes() -> Vec<u8> {
    let plain_data = b"Hello, this is pure text compressed inside gzip, not a tar archive.";
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(plain_data).expect("write plain text");
    encoder.finish().expect("finish gzip")
}

#[test]
fn test_plain_gzip_vs_nested_tar_gz() {
    let opts = DetectOptions::default();

    // 1. Pure Gzip
    let plain_gz = create_plain_gz_bytes();
    let rep_plain = detect_bytes(&plain_gz, &opts).expect("detect plain gz");
    assert_eq!(rep_plain.identification, Identification::Format);
    assert_eq!(rep_plain.format_id.as_deref(), Some("gzip"));
    assert_eq!(rep_plain.recommended_extension.as_deref(), Some("gz"));
    assert!(!rep_plain
        .evidence
        .iter()
        .any(|e| e.code == "sig.tar.embedded"));

    // 2. Nested tar.gz
    let tar_gz = create_tar_gz_bytes();
    let rep_tar_gz = detect_bytes(&tar_gz, &opts).expect("detect tar gz");
    assert_eq!(rep_tar_gz.identification, Identification::Format);
    assert_eq!(rep_tar_gz.format_id.as_deref(), Some("tar.gz"));
    assert_eq!(rep_tar_gz.container_id.as_deref(), Some("gzip"));
    assert_eq!(rep_tar_gz.category, "archive");
    assert_eq!(rep_tar_gz.verification, Verification::Structure);
    assert_eq!(rep_tar_gz.recommended_extension.as_deref(), Some("tar.gz"));
    assert!(rep_tar_gz
        .accepted_extensions
        .contains(&"tar.gz".to_string()));
    assert!(rep_tar_gz.accepted_extensions.contains(&"tgz".to_string()));
    assert!(rep_tar_gz.accepted_extensions.contains(&"gz".to_string()));
    assert!(rep_tar_gz
        .evidence
        .iter()
        .any(|e| e.code == "sig.tar.embedded"));
    assert!(rep_tar_gz
        .evidence
        .iter()
        .any(|e| e.code == "sig.tar.ustar"));
    assert!(rep_tar_gz
        .evidence
        .iter()
        .any(|e| e.code == "sig.tar.checksum"));
}

#[test]
fn test_tar_gz_extension_diagnosis_matrix() {
    let opts = DetectOptions::default();
    let tar_gz_bytes = create_tar_gz_bytes();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp_dir = std::env::temp_dir();

    // 1. File with .tar.gz -> ExtensionStatus::Match
    let path_targz = tmp_dir.join(format!("pkg_{}.tar.gz", nonce));
    std::fs::write(&path_targz, &tar_gz_bytes).unwrap();
    let rep = kindprobe::detect_path(&path_targz, &opts).unwrap();
    assert_eq!(rep.extension_status, ExtensionStatus::Match);
    let _ = std::fs::remove_file(&path_targz);

    // 2. File with .tgz -> ExtensionStatus::Match
    let path_tgz = tmp_dir.join(format!("pkg_{}.tgz", nonce));
    std::fs::write(&path_tgz, &tar_gz_bytes).unwrap();
    let rep = kindprobe::detect_path(&path_tgz, &opts).unwrap();
    assert_eq!(rep.extension_status, ExtensionStatus::Match);
    let _ = std::fs::remove_file(&path_tgz);

    // 3. File with .gz -> ExtensionStatus::Match
    let path_gz = tmp_dir.join(format!("pkg_{}.gz", nonce));
    std::fs::write(&path_gz, &tar_gz_bytes).unwrap();
    let rep = kindprobe::detect_path(&path_gz, &opts).unwrap();
    assert_eq!(rep.extension_status, ExtensionStatus::Match);
    let _ = std::fs::remove_file(&path_gz);

    // 4. File with .zip -> ExtensionStatus::Mismatch
    let path_zip = tmp_dir.join(format!("pkg_{}.zip", nonce));
    std::fs::write(&path_zip, &tar_gz_bytes).unwrap();
    let rep = kindprobe::detect_path(&path_zip, &opts).unwrap();
    assert_eq!(rep.extension_status, ExtensionStatus::Mismatch);
    let _ = std::fs::remove_file(&path_zip);
}

#[test]
fn test_tar_gz_cli_e2e_display() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bundle_{}.tar.gz", nonce));
    let tar_gz_bytes = create_tar_gz_bytes();
    std::fs::write(&path, tar_gz_bytes).unwrap();

    // Chinese display
    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_WIN_LANGID", "2052")
        .arg(&path)
        .output()
        .expect("run cli zh");
    assert_eq!(out_zh.status.code(), Some(0));
    let stdout_zh = String::from_utf8_lossy(&out_zh.stdout);
    assert!(stdout_zh.contains("识别结果: 格式已确认 (tar.gz)"));
    assert!(stdout_zh.contains("容器: gzip"));
    assert!(stdout_zh.contains("类别: 归档压缩"));
    assert!(stdout_zh.contains("推荐扩展名: .tar.gz"));
    assert!(stdout_zh.contains("扩展名状态: 匹配"));
    assert!(stdout_zh.contains("sig.tar.embedded"));
    assert!(stdout_zh.contains("在压缩数据流中检测到内嵌 POSIX ustar 归档"));

    // English display
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_WIN_LANGID", "1033")
        .arg(&path)
        .output()
        .expect("run cli en");
    assert_eq!(out_en.status.code(), Some(0));
    let stdout_en = String::from_utf8_lossy(&out_en.stdout);
    assert!(stdout_en.contains("Identification: Format (tar.gz)"));
    assert!(stdout_en.contains("Container: gzip"));
    assert!(stdout_en.contains("Category: archive"));
    assert!(stdout_en.contains("Recommended Extension: .tar.gz"));
    assert!(stdout_en.contains("Extension Status: Match"));
    assert!(stdout_en.contains("sig.tar.embedded"));

    // JSON mode
    let out_json = Command::new(BIN_PATH)
        .arg("--json")
        .arg(&path)
        .output()
        .expect("run cli json");
    assert_eq!(out_json.status.code(), Some(0));
    let stdout_json = String::from_utf8_lossy(&out_json.stdout);
    let report: DetectionReport =
        serde_json::from_str(stdout_json.trim()).expect("deserialize json");
    assert_eq!(report.format_id.as_deref(), Some("tar.gz"));
    assert_eq!(report.container_id.as_deref(), Some("gzip"));
    assert_eq!(report.recommended_extension.as_deref(), Some("tar.gz"));
    assert!(report.evidence.iter().any(|e| e.code == "sig.tar.embedded"));

    let _ = std::fs::remove_file(&path);
}
