use kindprobe::model::{DetectionReport, ErrorEnvelope, Identification, Inspection};
use std::io::Write;
use std::process::{Command, Stdio};

const BIN_PATH: &str = env!("CARGO_BIN_EXE_kindprobe");

#[test]
fn test_cli_flags() {
    // --help
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("--help")
        .output()
        .expect("run --help");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));

    // --version
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("--version")
        .output()
        .expect("run --version");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kindprobe"));

    // Missing arguments
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .output()
        .expect("run no args");
    assert_eq!(output.status.code(), Some(2));

    // Unknown option
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("--unknown-flag-xyz")
        .output()
        .expect("run unknown flag");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_cli_io_errors_and_json_envelope() {
    // Non-existent file in text mode -> exit code 4
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
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
        .env("LC_ALL", "C")
        .arg("--unknown-flag-xyz")
        .output()
        .expect("run unknown flag text");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized option"));

    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
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
        .env("LC_ALL", "C")
        .arg("--")
        .arg("fixtures/valid_png.png")
        .output()
        .expect("run double dash");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_cli_non_utf8_arguments_and_paths() {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // 1. Non-UTF-8 option flag should return exit code 2, not panic
        let output = Command::new(BIN_PATH)
            .env("LC_ALL", "C")
            .arg(OsStr::from_bytes(b"--invalid\xff\xfe"))
            .output()
            .expect("run non-utf8 flag");
        assert_eq!(output.status.code(), Some(2));

        // 2. Non-UTF-8 file path should execute without panic
        let non_utf8_path = OsStr::from_bytes(b"/tmp/kindprobe_test_non_utf8_\xff\xfe.png");
        let png_sig = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x5c\x72\x1c\x5d\x00\x00\x00\x00IEND\xae\x42\x60\x82";
        match std::fs::write(std::path::Path::new(non_utf8_path), png_sig) {
            Ok(()) => {
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
            Err(_) => {
                // File systems like macOS APFS reject non-UTF-8 filenames at OS level (EILSEQ).
                // Ensure CLI handles such non-UTF-8 path arguments safely without panicking.
                let output = Command::new(BIN_PATH)
                    .env("LC_ALL", "C")
                    .arg(non_utf8_path)
                    .output()
                    .expect("run non-utf8 non-existent path");
                assert_eq!(output.status.code(), Some(4));
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(stderr.contains("io.open_failed"));
            }
        }
    }

    // 3. Long multibyte path exceeding 1024 bytes with boundary across non-ASCII character
    // Must safely truncate at unicode character boundary without slicing panic
    let mut long_str = "x".repeat(1023);
    long_str.push_str("测试超长路径样本.bin");
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg(&long_str)
        .output()
        .expect("run long multibyte path");
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[truncated]"));
    assert!(stderr.contains("io.open_failed"));
}

#[test]
fn test_dedicated_interactive_console_behavior() {
    // Note on test coverage boundary:
    // On Linux CI/containers, these tests verify the logic flow, single-exit finalization,
    // immediate non-blocking execution under piped/redirected standard streams, EOF resilience,
    // --json bypass, and exact 0/2/3/4 exit code preservation.
    // Physical Windows CSRSS/Conhost console allocation, GetConsoleWindow() presence, and
    // GetConsoleProcessList() == 1 (Explorer) vs >= 2 (shell) behavior are verified on Windows.

    // 1. Normal execution (default / noninteractive / piped streams):
    // Must NOT output pause prompt or wait, and must strictly retain 0, 2, 3, 4 exit codes.
    let test_cases = [
        (&["fixtures/valid_png.png"][..], 0),
        (&["--unknown-flag-xyz"][..], 2),
        (&["fixtures/budget_xml_deep.docx"][..], 3),
        (&["fixtures/does_not_exist_xyz.bin"][..], 4),
    ];
    for (args, expected_code) in test_cases {
        let mut child = Command::new(BIN_PATH)
            .env("LC_ALL", "C")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn failed for args {:?}: {}", args, e));
        // Drop stdin immediately (simulate empty pipe / EOF)
        drop(child.stdin.take());
        let output = child.wait_with_output().expect("wait child");
        assert_eq!(
            output.status.code(),
            Some(expected_code),
            "failed for args {:?}",
            args
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stdout.contains("Press Enter to close"),
            "stdout had pause text for {:?}",
            args
        );
        assert!(
            !stderr.contains("Press Enter to close"),
            "stderr had pause text for {:?}",
            args
        );
    }

    // 2. --json mode: must NOT pause or output pause prompt even if test flag is set
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("--json")
        .arg("fixtures/valid_png.png")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run json with flag");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("Press Enter to close"));
    assert!(!stderr.contains("Press Enter to close"));

    // 3. Dedicated console: double click with no arguments
    // Must display drag-a-file usage hint, prompt "Press Enter to close...", and exit code 2
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn double-click");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait double-click");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Drag and drop a file onto kindprobe.exe"));
    assert!(stderr.contains("Press Enter to close..."));

    // 4. Dedicated console: dropping a single valid file
    // Must display normal identification report, prompt "Press Enter to close...", and exit code 0
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/valid_png.png")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn drag single file");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait drag single file");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("File: fixtures/valid_png.png"));
    assert!(stdout.contains("Identification: Format (png)"));
    assert!(stderr.contains("Press Enter to close..."));

    // 5. Dedicated console: dropping non-existent file
    // Must display error, prompt "Press Enter to close...", and exit code 4
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/does_not_exist_xyz.bin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn drag missing file");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait drag missing file");
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("io.open_failed"));
    assert!(stderr.contains("Press Enter to close..."));

    // 6. Dedicated console: dropping multiple files
    // Must display multiple file arguments error, prompt "Press Enter to close...", and exit code 2
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/valid_png.png")
        .arg("fixtures/valid_pdf.pdf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn drag multiple files");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait drag multiple files");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("multiple file arguments provided"));
    assert!(stderr.contains("Press Enter to close..."));

    // 7. Dedicated console with immediate EOF (stdin closed without Enter):
    // Must NOT hang or infinite-loop, and must exit with the original exit code.
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/valid_png.png")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn drag single file with EOF");
    // Close stdin immediately
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait EOF drag file");
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Press Enter to close..."));
}

#[test]
fn test_cli_bilingual_help_and_version() {
    // English Help
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("--help")
        .output()
        .expect("run en help");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("ARGS:"));
    assert!(stdout.contains("OPTIONS:"));
    assert!(stdout.contains("EXIT CODES:"));
    assert!(stdout.contains("Content-first file format identification tool"));

    // Chinese Help
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("--help")
        .output()
        .expect("run zh help");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("用法:"));
    assert!(stdout.contains("参数:"));
    assert!(stdout.contains("选项:"));
    assert!(stdout.contains("退出码:"));
    assert!(stdout.contains("基于内容优先的文件格式识别工具"));

    // Version identical program literal
    let output_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("--version")
        .output()
        .expect("run en version");
    let output_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("--version")
        .output()
        .expect("run zh version");
    assert_eq!(output_en.status.code(), Some(0));
    assert_eq!(output_zh.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output_en.stdout).starts_with("kindprobe "));
    assert_eq!(output_en.stdout, output_zh.stdout);
}

#[test]
fn test_cli_bilingual_success_and_container_reports() {
    // 1. Valid PNG in English
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("fixtures/valid_png.png")
        .output()
        .expect("run en png");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("File: fixtures/valid_png.png"));
    assert!(stdout.contains("Identification: Format (png)"));
    assert!(stdout.contains("Category: image"));
    assert!(stdout.contains("Verification: Structure"));
    assert!(stdout.contains("Inspection: Complete"));
    assert!(stdout.contains("Recommended Extension: .png"));
    assert!(stdout.contains("Extension Status: Match"));
    assert!(stdout.contains("Stats:"));
    assert!(stdout.contains("Source Read:"));

    // 2. Valid PNG in Chinese
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("fixtures/valid_png.png")
        .output()
        .expect("run zh png");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("文件: fixtures/valid_png.png"));
    assert!(stdout.contains("识别结果: 格式已确认 (png)"));
    assert!(stdout.contains("类别: 图像"));
    assert!(stdout.contains("验证方式: 内部结构"));
    assert!(stdout.contains("检测程度: 完整"));
    assert!(stdout.contains("推荐扩展名: .png"));
    assert!(stdout.contains("扩展名状态: 匹配"));
    assert!(stdout.contains("统计信息:"));
    assert!(stdout.contains("读取源数据:"));

    // 3. Container-only in English
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("fixtures/fake_docx_zip.zip")
        .output()
        .expect("run en container");
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Identification: ContainerOnly (none)"));
    assert!(stdout.contains("Container: zip"));
    assert!(stdout.contains("Category: container"));

    // 4. Container-only in Chinese
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("fixtures/fake_docx_zip.zip")
        .output()
        .expect("run zh container");
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("识别结果: 仅识别容器 (none)"));
    assert!(stdout.contains("容器: zip"));
    assert!(stdout.contains("类别: 容器"));
}

#[test]
fn test_cli_bilingual_arguments_and_io_errors() {
    // 1. Missing file argument
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .output()
        .expect("run en missing arg");
    assert_eq!(out_en.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out_en.stderr)
        .contains("Error: missing file argument; use --help for usage information"));

    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .output()
        .expect("run zh missing arg");
    assert_eq!(out_zh.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out_zh.stderr)
        .contains("错误: 缺少文件参数；使用 --help 查看用法信息"));

    // 2. Unrecognized option
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("--unrecognized-opt")
        .output()
        .expect("run en bad opt");
    assert_eq!(out_en.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out_en.stderr)
        .contains("Error: unrecognized option '--unrecognized-opt'"));

    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("--unrecognized-opt")
        .output()
        .expect("run zh bad opt");
    assert_eq!(out_zh.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out_zh.stderr).contains("错误: 未识别的选项 '--unrecognized-opt'")
    );

    // 3. Multiple files
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("fixtures/valid_png.png")
        .arg("fixtures/valid_pdf.pdf")
        .output()
        .expect("run en multi files");
    assert_eq!(out_en.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out_en.stderr).contains("Error: multiple file arguments provided")
    );

    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("fixtures/valid_png.png")
        .arg("fixtures/valid_pdf.pdf")
        .output()
        .expect("run zh multi files");
    assert_eq!(out_zh.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out_zh.stderr).contains("错误: 提供了多个文件参数"));

    // 4. Missing file (I/O error, exit code 4)
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("fixtures/not_found_sample.bin")
        .output()
        .expect("run en missing file");
    assert_eq!(out_en.status.code(), Some(4));
    let stderr_en = String::from_utf8_lossy(&out_en.stderr);
    assert!(stderr_en.contains("Error [io.open_failed]: Failed to open or read file"));

    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("fixtures/not_found_sample.bin")
        .output()
        .expect("run zh missing file");
    assert_eq!(out_zh.status.code(), Some(4));
    let stderr_zh = String::from_utf8_lossy(&out_zh.stderr);
    assert!(stderr_zh
        .contains("错误 [io.open_failed]: 打开或读取文件 'fixtures/not_found_sample.bin' 失败"));

    // 5. Directory path (not regular file, exit code 4)
    let out_en = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .arg("fixtures")
        .output()
        .expect("run en dir");
    assert_eq!(out_en.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out_en.stderr)
        .contains("Error [io.not_regular_file]: Path 'fixtures' is not a regular file"));

    let out_zh = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .arg("fixtures")
        .output()
        .expect("run zh dir");
    assert_eq!(out_zh.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out_zh.stderr)
        .contains("错误 [io.not_regular_file]: 路径 'fixtures' 不是普通文件"));
}

#[test]
fn test_cli_bilingual_dedicated_console_drag_and_pause() {
    // 1. Chinese dedicated console: double click (no file)
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn zh double-click");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait zh double-click");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("将文件拖放到 kindprobe.exe 上即可识别其格式。"));
    assert!(stderr.contains("按 Enter 键关闭..."));

    // 2. Chinese dedicated console: drag single file with Enter
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/valid_png.png")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn zh drag single file");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait zh drag single file");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("文件: fixtures/valid_png.png"));
    assert!(stderr.contains("按 Enter 键关闭..."));

    // 3. Chinese dedicated console: immediate EOF without Enter
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/valid_png.png")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn zh drag EOF");
    drop(child.stdin.take()); // Close stdin immediately
    let output = child.wait_with_output().expect("wait zh drag EOF");
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("按 Enter 键关闭..."));

    // 4. Chinese dedicated console: dropping missing file
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/does_not_exist_xyz.bin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn zh drag missing file");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().expect("wait zh drag missing file");
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("错误 [io.open_failed]:"));
    assert!(stderr.contains("按 Enter 键关闭..."));

    // 5. Chinese dedicated console: dropping multiple files
    let mut child = Command::new(BIN_PATH)
        .env("LC_ALL", "zh_CN.UTF-8")
        .env("KINDPROBE_TEST_DEDICATED_CONSOLE", "1")
        .arg("fixtures/valid_png.png")
        .arg("fixtures/valid_pdf.pdf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn zh drag multiple files");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n");
    }
    let output = child
        .wait_with_output()
        .expect("wait zh drag multiple files");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("错误: 提供了多个文件参数"));
    assert!(stderr.contains("按 Enter 键关闭..."));
}

#[test]
fn test_cli_json_contract_identical_across_locales() {
    let locales = [
        "C",
        "POSIX",
        "en_US.UTF-8",
        "zh_CN.UTF-8",
        "zh_TW.UTF-8",
        "zh_HK.UTF-8",
        "ja_JP.UTF-8",
    ];

    // 1. Success detection JSON must be byte-for-byte identical across all locales
    let mut baseline_json: Option<String> = None;
    for loc in locales {
        let output = Command::new(BIN_PATH)
            .env("LC_ALL", loc)
            .arg("--json")
            .arg("fixtures/valid_png.png")
            .output()
            .unwrap_or_else(|e| panic!("run json under {}: {}", loc, e));
        assert_eq!(output.status.code(), Some(0));
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut report: DetectionReport = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("parse json under {}: {}", loc, e));
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.format_id.as_deref(), Some("png"));
        assert_eq!(report.category, "image");
        assert_eq!(report.identification, Identification::Format);
        assert_eq!(report.inspection, Inspection::Complete);
        assert_eq!(
            report.verification,
            kindprobe::model::Verification::Structure
        );
        assert_eq!(
            report.extension_status,
            kindprobe::model::ExtensionStatus::Match
        );

        // Normalize non-deterministic runtime duration for exact schema/content comparison
        report.stats.duration_micros = 0;
        let normalized = serde_json::to_string(&report).unwrap();

        if let Some(ref base) = baseline_json {
            assert_eq!(
                &normalized, base,
                "JSON stdout deviated under locale {}",
                loc
            );
        } else {
            baseline_json = Some(normalized);
        }
    }

    // 2. ErrorEnvelope for missing file must be identical across all locales
    let mut baseline_err: Option<String> = None;
    for loc in locales {
        let output = Command::new(BIN_PATH)
            .env("LC_ALL", loc)
            .arg("--json")
            .arg("fixtures/not_found_sample.bin")
            .output()
            .unwrap_or_else(|e| panic!("run missing file json under {}: {}", loc, e));
        assert_eq!(output.status.code(), Some(4));
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let env: ErrorEnvelope = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("parse error envelope under {}: {}", loc, e));
        assert_eq!(env.schema_version, 1);
        assert_eq!(env.error.code, "io.open_failed");
        assert!(env.error.message.starts_with("Failed to open or read file"));

        if let Some(ref base) = baseline_err {
            assert_eq!(&stdout, base, "ErrorEnvelope deviated under locale {}", loc);
        } else {
            baseline_err = Some(stdout);
        }
    }

    // 3. ErrorEnvelope for invalid argument must be identical across all locales
    let mut baseline_arg_err: Option<String> = None;
    for loc in locales {
        let output = Command::new(BIN_PATH)
            .env("LC_ALL", loc)
            .arg("--json")
            .arg("--invalid-flag-abc")
            .output()
            .unwrap_or_else(|e| panic!("run arg error json under {}: {}", loc, e));
        assert_eq!(output.status.code(), Some(2));
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let env: ErrorEnvelope = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("parse arg error envelope under {}: {}", loc, e));
        assert_eq!(env.schema_version, 1);
        assert_eq!(env.error.code, "error.invalid_arguments");
        assert_eq!(
            env.error.message,
            "unrecognized option '--invalid-flag-abc'"
        );

        if let Some(ref base) = baseline_arg_err {
            assert_eq!(
                &stdout, base,
                "Arg ErrorEnvelope deviated under locale {}",
                loc
            );
        } else {
            baseline_arg_err = Some(stdout);
        }
    }
}

#[test]
fn test_cli_locale_precedence_and_fallback() {
    // 1. LC_ALL overrides LC_MESSAGES and LANG:
    // LC_ALL="C", LC_MESSAGES="zh_CN.UTF-8", LANG="zh_CN.UTF-8" => English
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "C")
        .env("LC_MESSAGES", "zh_CN.UTF-8")
        .env("LANG", "zh_CN.UTF-8")
        .arg("--help")
        .output()
        .expect("run precedence LC_ALL=C");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));
    assert!(!stdout.contains("用法:"));

    // 2. LC_ALL empty falls through to LC_MESSAGES:
    // LC_ALL="", LC_MESSAGES="zh_CN.UTF-8", LANG="en_US.UTF-8" => Chinese
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "")
        .env("LC_MESSAGES", "zh_CN.UTF-8")
        .env("LANG", "en_US.UTF-8")
        .arg("--help")
        .output()
        .expect("run precedence LC_MESSAGES=zh_CN");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("用法:"));

    // 3. LC_ALL and LC_MESSAGES empty fall through to LANG:
    // LC_ALL="", LC_MESSAGES="", LANG="zh_CN.UTF-8" => Chinese
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "")
        .env("LC_MESSAGES", "")
        .env("LANG", "zh_CN.UTF-8")
        .arg("--help")
        .output()
        .expect("run precedence LANG=zh_CN");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("用法:"));

    // 4. LC_MESSAGES="C" takes precedence over LANG="zh_CN.UTF-8":
    // LC_ALL="", LC_MESSAGES="C", LANG="zh_CN.UTF-8" => English
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "")
        .env("LC_MESSAGES", "C")
        .env("LANG", "zh_CN.UTF-8")
        .arg("--help")
        .output()
        .expect("run precedence LC_MESSAGES=C");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));

    // 5. All empty / unset => English fallback on POSIX, or native system locale on Windows
    let output = Command::new(BIN_PATH)
        .env("LC_ALL", "")
        .env("LC_MESSAGES", "")
        .env("LANG", "")
        .arg("--help")
        .output()
        .expect("run precedence all empty");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    #[cfg(not(windows))]
    assert!(stdout.contains("USAGE:"));
    #[cfg(windows)]
    assert!(stdout.contains("USAGE:") || stdout.contains("用法:"));

    // 6. Test Windows Language ID mapping hook across variants without altering OS settings
    // 0x0804 (zh-CN) -> Chinese
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "2052")
        .arg("--help")
        .output()
        .expect("run win langid zh-CN");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("用法:"));

    // 0x0404 (zh-TW) -> Chinese
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "1028")
        .arg("--help")
        .output()
        .expect("run win langid zh-TW");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("用法:"));

    // 0x0c04 (zh-HK) -> Chinese
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "3076")
        .arg("--help")
        .output()
        .expect("run win langid zh-HK");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("用法:"));

    // 0x1004 (zh-SG) -> Chinese
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "4100")
        .arg("--help")
        .output()
        .expect("run win langid zh-SG");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("用法:"));

    // 0x1404 (zh-MO) -> Chinese
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "5124")
        .arg("--help")
        .output()
        .expect("run win langid zh-MO");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("用法:"));

    // 0x0409 (en-US) -> English
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "1033")
        .arg("--help")
        .output()
        .expect("run win langid en-US");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("USAGE:"));

    // 0x0000 (failure / unassigned) -> English
    let output = Command::new(BIN_PATH)
        .env("KINDPROBE_TEST_WIN_LANGID", "0")
        .arg("--help")
        .output()
        .expect("run win langid 0");
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("USAGE:"));
}
