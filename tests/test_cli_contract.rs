use std::process::Command;
use kindprobe::model::{DetectionReport, ErrorEnvelope, Identification, Inspection};

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
    // Non-existent file in text mode -> exit code 4
    let output = Command::new(BIN_PATH)
        .arg("fixtures/does_not_exist_xyz.bin")
        .output()
        .expect("run missing file");
    assert_eq!(output.status.code(), Some(4));

    // Non-existent file in JSON mode -> exit code 4 + full ErrorEnvelope validation
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures/does_not_exist_xyz.bin")
        .output()
        .expect("run missing file json");
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let env: ErrorEnvelope = serde_json::from_str(stdout.trim())
        .expect("must deserialize full ErrorEnvelope on missing file");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.error.code, "io.open_failed");
    assert!(!env.error.message.is_empty());

    // Directory path in JSON mode (not regular file) -> exit code 4 + full ErrorEnvelope validation
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures")
        .output()
        .expect("run dir");
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let env: ErrorEnvelope = serde_json::from_str(stdout.trim())
        .expect("must deserialize full ErrorEnvelope on non regular file");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.error.code, "io.not_regular_file");
    assert!(!env.error.message.is_empty());
}

#[test]
fn test_cli_exit_codes_and_envelopes_full_coverage() {
    // 1. Exit code 0: Format confirmed and inspection complete (valid_png.png)
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures/valid_png.png")
        .output()
        .expect("run valid_png");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: DetectionReport = serde_json::from_str(stdout.trim())
        .expect("must deserialize full DetectionReport on exit code 0");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.format_id.as_deref(), Some("png"));
    assert_eq!(report.identification, Identification::Format);
    assert_eq!(report.inspection, Inspection::Complete);

    // 2. Exit code 2: CLI argument error (unknown option or missing arg)
    // In text mode:
    let output = Command::new(BIN_PATH)
        .arg("--unknown-flag-xyz")
        .output()
        .expect("run unknown flag text");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized option"));

    let output = Command::new(BIN_PATH)
        .output()
        .expect("run missing arg text");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing file argument"));

    // In JSON mode: emits ErrorEnvelope on stdout with exit code 2
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("--unknown-flag-xyz")
        .output()
        .expect("run unknown flag json");
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let env: ErrorEnvelope = serde_json::from_str(stdout.trim())
        .expect("must deserialize full ErrorEnvelope on exit 2 with --json");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.error.code, "error.invalid_arguments");
    assert!(env.error.message.contains("unrecognized option"));

    // In JSON mode: --json placed after bad option
    let output = Command::new(BIN_PATH)
        .arg("--unknown-flag-xyz")
        .arg("--json")
        .output()
        .expect("run bad option before --json");
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let env: ErrorEnvelope = serde_json::from_str(stdout.trim())
        .expect("must deserialize full ErrorEnvelope when --json is placed after bad option");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.error.code, "error.invalid_arguments");

    // In JSON mode: missing file argument
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .output()
        .expect("run missing file json");
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let env: ErrorEnvelope = serde_json::from_str(stdout.trim())
        .expect("must deserialize full ErrorEnvelope on missing file argument in JSON mode");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.error.code, "error.invalid_arguments");
    assert!(env.error.message.contains("missing file argument"));

    // In JSON mode: multiple file arguments
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures/valid_png.png")
        .arg("fixtures/valid_pdf.pdf")
        .output()
        .expect("run multiple files json");
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let env: ErrorEnvelope = serde_json::from_str(stdout.trim())
        .expect("must deserialize full ErrorEnvelope on multiple files in JSON mode");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.error.code, "error.invalid_arguments");
    assert!(env.error.message.contains("multiple file arguments"));

    // 3. Exit code 3: Format not confirmed / container only / inspection limited
    // fake_docx_zip.zip: container_only -> exit code 3
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures/fake_docx_zip.zip")
        .output()
        .expect("run fake_docx_zip");
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: DetectionReport = serde_json::from_str(stdout.trim())
        .expect("must deserialize full DetectionReport on exit code 3 (container_only)");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.identification, Identification::ContainerOnly);
    assert_eq!(report.container_id.as_deref(), Some("zip"));
    assert_eq!(report.format_id, None);
    assert_eq!(report.recommended_extension, None);

    // budget_xml_deep.docx: limited inspection -> exit code 3
    let output = Command::new(BIN_PATH)
        .arg("--json")
        .arg("fixtures/budget_xml_deep.docx")
        .output()
        .expect("run budget_xml_deep");
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: DetectionReport = serde_json::from_str(stdout.trim())
        .expect("must deserialize full DetectionReport on exit code 3 (limited)");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.inspection, Inspection::Limited);

    // 4. Exit code 4: I/O or non-regular file tested in test_cli_io_errors_and_json_envelope
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
    let report: DetectionReport = serde_json::from_str(stdout.trim())
        .expect("must deserialize full DetectionReport on non-utf8 path");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.format_id.as_deref(), Some("png"));

    let _ = std::fs::remove_file(std::path::Path::new(non_utf8_path));
}
