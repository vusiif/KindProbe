use std::process::Command;

const BIN_PATH: &str = env!("CARGO_BIN_EXE_kindprobe");

#[test]
fn test_cli_flags() {
    // --help
    let output = Command::new(BIN_PATH)
        .arg("--help")
        .output()
        .expect("run --help");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));

    // --version
    let output = Command::new(BIN_PATH)
        .arg("--version")
        .output()
        .expect("run --version");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kindprobe"));

    // Missing arguments
    let output = Command::new(BIN_PATH)
        .output()
        .expect("run no args");
    assert_eq!(output.status.code(), Some(2));

    // Unknown option
    let output = Command::new(BIN_PATH)
        .arg("--unknown-flag-xyz")
        .output()
        .expect("run unknown flag");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_cli_io_errors_and_json_envelope() {
    // Non-existent file in text mode
    let output = Command::new(BIN_PATH)
        .arg("fixtures/does_not_exist_xyz.bin")
        .output()
        .expect("run missing file");
    assert_eq!(output.status.code(), Some(4));

    // Non-existent file in JSON mode
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures/does_not_exist_xyz.bin")
        .output()
        .expect("run missing file json");
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"schema_version\":1"));
    assert!(stdout.contains("\"io.open_failed\""));

    // Directory path in JSON mode (not regular file)
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures")
        .output()
        .expect("run dir");
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"io.not_regular_file\""));
}

#[test]
fn test_cli_double_dash_leading_dash() {
    // Handling leading dash file name
    let output = Command::new(BIN_PATH)
        .arg("--")
        .arg("fixtures/valid_png.png")
        .output()
        .expect("run double dash");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_cli_non_utf8_arguments_and_paths() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    // 1. Non-UTF-8 option flag should return exit code 2, not panic
    let output = Command::new(BIN_PATH)
        .arg(OsStr::from_bytes(b"--invalid\xff\xfe"))
        .output()
        .expect("run non-utf8 flag");
    assert_eq!(output.status.code(), Some(2));

    // 2. Non-UTF-8 file path should execute without panic
    let non_utf8_path = OsStr::from_bytes(b"/tmp/kindprobe_test_non_utf8_\xff\xfe.png");
    let png_sig = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x5c\x72\x1c\x5d\x00\x00\x00\x00IEND\xae\x42\x60\x82";
    std::fs::write(std::path::Path::new(non_utf8_path), png_sig).expect("write non-utf8 file");

    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg(non_utf8_path)
        .output()
        .expect("run non-utf8 path");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"format_id\":\"png\""));

    let _ = std::fs::remove_file(std::path::Path::new(non_utf8_path));
}
