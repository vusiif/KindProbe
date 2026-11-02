use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::detect_path;
use kindprobe::model::{DetectError, ErrorDetails, ErrorEnvelope};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        "KindProbe v{} - Content-first file format identification tool

USAGE:
    kindprobe [OPTIONS] <FILE>
    kindprobe [OPTIONS] -- <FILE>

ARGS:
    <FILE>    Path to the file to inspect

OPTIONS:
    --json       Output machine-readable JSON v1 to stdout
    -h, --help   Print help information
    -V, --version Print version information

EXIT CODES:
    0    Format confirmed and inspection complete
    2    CLI argument error
    3    Format not confirmed, container only, or inspection limited/ambiguous
    4    I/O error, file not found, permission denied, or not a regular file
    1    Unexpected internal failure
",
        VERSION
    );
}

fn print_version() {
    println!("kindprobe {}", VERSION);
}

fn main() {
    let cli_start = Instant::now();
    let args: Vec<OsString> = env::args_os().collect();

    let mut json_mode = false;
    let mut target_file: Option<PathBuf> = None;
    let mut after_double_dash = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if after_double_dash {
            if target_file.is_some() {
                eprintln!("Error: multiple file arguments provided");
                process::exit(2);
            }
            target_file = Some(PathBuf::from(arg));
        } else if arg == OsStr::new("--") {
            after_double_dash = true;
        } else if arg == OsStr::new("--json") {
            json_mode = true;
        } else if arg == OsStr::new("-h") || arg == OsStr::new("--help") {
            print_help();
            process::exit(0);
        } else if arg == OsStr::new("-V") || arg == OsStr::new("--version") {
            print_version();
            process::exit(0);
        } else {
            let s = arg.to_string_lossy();
            if s.starts_with('-') {
                eprintln!("Error: unrecognized option '{}'", s);
                process::exit(2);
            } else {
                if target_file.is_some() {
                    eprintln!("Error: multiple file arguments provided");
                    process::exit(2);
                }
                target_file = Some(PathBuf::from(arg));
            }
        }
        i += 1;
    }

    let file_path = match target_file {
        Some(f) => f,
        None => {
            eprintln!("Error: missing file argument\nUse --help for usage information");
            process::exit(2);
        }
    };

    let path = file_path.as_path();
    let display_str = file_path.to_string_lossy();
    let bounded_display = if display_str.len() > 1024 {
        format!("{}...[truncated]", &display_str[..1024])
    } else {
        display_str.to_string()
    };

    let options = DetectOptions::default();

    match detect_path(path, &options) {
        Ok(report) => {
            let cli_elapsed_micros = cli_start.elapsed().as_micros() as u64;

            if json_mode {
                let serialized = serde_json::to_string(&report).unwrap_or_else(|e| {
                    eprintln!("Serialization error: {}", e);
                    process::exit(1);
                });
                println!("{}", serialized);
            } else {
                println!("File: {}", bounded_display);
                println!(
                    "Identification: {:?} ({})",
                    report.identification,
                    report.format_id.as_deref().unwrap_or("none")
                );
                println!("Category: {}", report.category);
                if let Some(cid) = &report.container_id {
                    println!("Container: {}", cid);
                }
                println!("Verification: {:?}", report.verification);
                println!("Inspection: {:?}", report.inspection);
                if let Some(rec) = &report.recommended_extension {
                    println!("Recommended Extension: .{}", rec);
                }
                println!("Extension Status: {:?}", report.extension_status);
                if !report.evidence.is_empty() {
                    println!("Evidence:");
                    for ev in &report.evidence {
                        println!("  - {}", ev.code);
                    }
                }
                if !report.warnings.is_empty() {
                    println!("Warnings:");
                    for w in &report.warnings {
                        println!("  - [{}] {}", w.code, w.message);
                    }
                }
                println!("Stats:");
                println!("  Source Read: {} bytes", report.stats.source_bytes_read);
                println!("  Seeks: {}", report.stats.seek_count);
                println!("  Entries Inspected: {}", report.stats.entries_inspected);
                println!("  Decompressed: {} bytes", report.stats.decompressed_bytes);
                println!("  Engine Time: {} µs", report.stats.duration_micros);
                println!("  CLI Total Time: {} µs", cli_elapsed_micros);
            }

            process::exit(report.cli_exit_code());
        }
        Err(err) => {
            let (code_str, msg_str, exit_code) = match &err {
                DetectError::Io(e) => (
                    "io.open_failed",
                    format!("Failed to open or read file '{}': {}", bounded_display, e),
                    4,
                ),
                DetectError::NotRegularFile(_) => (
                    "io.not_regular_file",
                    format!("Path '{}' is not a regular file (e.g. directory or device)", bounded_display),
                    4,
                ),
                DetectError::InvalidParameter(msg) => (
                    "error.invalid_parameter",
                    msg.clone(),
                    1,
                ),
            };

            if json_mode {
                let env = ErrorEnvelope {
                    schema_version: 1,
                    error: ErrorDetails {
                        code: code_str.to_string(),
                        message: msg_str,
                    },
                };
                let json_err = serde_json::to_string(&env).unwrap();
                println!("{}", json_err);
            } else {
                eprintln!("Error [{}]: {}", code_str, msg_str);
            }
            process::exit(exit_code);
        }
    }
}
