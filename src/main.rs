mod cli;

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::detect_path;
use kindprobe::model::{DetectError, ErrorDetails, ErrorEnvelope};

const VERSION: &str = env!("CARGO_PKG_VERSION");

use cli::batch::run_batch;
use cli::i18n::{
    detect_app_language, extract_arch_display_cn, extract_arch_display_en, format_category_cn,
    format_evidence_explanation_cn, format_extension_status_cn, format_identification_cn,
    format_inspection_cn, format_verification_cn, format_warning_explanation_cn, AppLanguage,
};

#[cfg(windows)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod win_console {
    type DWORD = u32;
    type BOOL = i32;
    type HANDLE = *mut std::ffi::c_void;
    type HWND = *mut std::ffi::c_void;

    const STD_OUTPUT_HANDLE: DWORD = 0xfffffff5; // -11i32 as u32
    const STD_INPUT_HANDLE: DWORD = 0xfffffff6; // -10i32 as u32

    #[link(name = "kernel32")]
    extern "system" {
        fn GetConsoleProcessList(process_list: *mut DWORD, count: DWORD) -> DWORD;
        fn GetConsoleWindow() -> HWND;
        fn GetStdHandle(which: DWORD) -> HANDLE;
        fn GetConsoleMode(handle: HANDLE, mode: *mut DWORD) -> BOOL;
    }

    pub fn is_dedicated_interactive_console() -> bool {
        // Test override for automated verification of console behavior
        if let Some(v) = std::env::var_os("KINDPROBE_TEST_DEDICATED_CONSOLE") {
            return v == "1";
        }

        // SAFETY: All Win32 console calls below interact only with standard Windows subsystem
        // queries without executing arbitrary memory transformations:
        // - GetConsoleWindow() takes no parameters and returns an HWND or NULL.
        // - GetStdHandle() passes standard predefined handle IDs (STD_OUTPUT_HANDLE, STD_INPUT_HANDLE).
        // - GetConsoleMode() writes to valid stack-allocated DWORD references (&mut out_mode / &mut in_mode).
        // - GetConsoleProcessList() writes to a stack-allocated buffer (pids) with exact capacity 2 matching count argument.
        unsafe {
            // 1. Must have a valid console window handle
            let hwnd = GetConsoleWindow();
            if hwnd.is_null() {
                return false;
            }

            // 2. Standard output must be attached to an interactive console buffer (not redirected/piped)
            let h_out = GetStdHandle(STD_OUTPUT_HANDLE);
            if h_out.is_null() || h_out == (-1isize as HANDLE) {
                return false;
            }
            let mut out_mode: DWORD = 0;
            if GetConsoleMode(h_out, &mut out_mode) == 0 {
                return false;
            }

            // 3. Standard input must be attached to an interactive console input buffer (not redirected/piped)
            let h_in = GetStdHandle(STD_INPUT_HANDLE);
            if h_in.is_null() || h_in == (-1isize as HANDLE) {
                return false;
            }
            let mut in_mode: DWORD = 0;
            if GetConsoleMode(h_in, &mut in_mode) == 0 {
                return false;
            }

            // 4. Check console process list:
            // When Explorer launches a console app (drag-and-drop or double-click),
            // a fresh console window is allocated hosting solely this process (count == 1).
            // When launched from an existing shell (cmd.exe, powershell.exe), the shell is also attached (count >= 2).
            // If the call fails (count == 0), fail-open (return false, no pause).
            let mut pids = [0u32; 2];
            let count = GetConsoleProcessList(pids.as_mut_ptr(), 2);
            count == 1
        }
    }
}

#[cfg(not(windows))]
mod win_console {
    pub fn is_dedicated_interactive_console() -> bool {
        // Test override for automated Linux verification of the interactive console branches
        std::env::var_os("KINDPROBE_TEST_DEDICATED_CONSOLE").is_some_and(|v| v == "1")
    }
}

fn is_dedicated_interactive_console() -> bool {
    win_console::is_dedicated_interactive_console()
}

fn print_help(lang: AppLanguage) {
    match lang {
        AppLanguage::Chinese => {
            println!(
                "KindProbe v{} - 基于内容优先的文件格式识别工具

用法:
    kindprobe [OPTIONS] <FILE>
    kindprobe [OPTIONS] -- <FILE>
    kindprobe --batch [-r] [--jsonl] <PATH>...

参数:
    <FILE>    待检测文件的路径
    <PATH>    批量模式下的文件或目录路径

选项:
    --json           向标准输出输出机器可读的 JSON v1
    --batch          启用批量模式（支持多文件/目录，使用零分配核心引擎）
    -r, --recursive  递归遍历目录（自动启用批量模式）
    --jsonl          以 JSON Lines 格式逐行输出结果（自动启用批量模式）
    -j, --jobs <N>   批量模式并发线程数（默认自动检测 CPU 核心数）
    -h, --help       打印帮助信息
    -V, --version    打印版本信息

退出码:
    0    格式确认且检测完成
    2    命令行参数错误
    3    格式未确认、仅识别容器或检测受限/存在歧义
    4    I/O 错误、文件不存在、权限不足或非普通文件
    1    未预期的内部错误
",
                VERSION
            );
        }
        AppLanguage::English => {
            println!(
                "KindProbe v{} - Content-first file format identification tool

USAGE:
    kindprobe [OPTIONS] <FILE>
    kindprobe [OPTIONS] -- <FILE>
    kindprobe --batch [-r] [--jsonl] <PATH>...

ARGS:
    <FILE>    Path to the file to inspect
    <PATH>    File or directory paths in batch mode

OPTIONS:
    --json           Output machine-readable JSON v1 to stdout
    --batch          Enable batch mode (multiple files/dirs via zero-alloc core)
    -r, --recursive  Recursively traverse directories (implies --batch)
    --jsonl          Output one JSON object per line (implies --batch)
    -j, --jobs <N>   Worker threads in batch mode (default: auto-detected CPU count)
    -h, --help       Print help information
    -V, --version    Print version information

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
    }
}

fn print_version() {
    println!("kindprobe {}", VERSION);
}

enum ArgErrorType {
    UnrecognizedOption(String),
    InvalidOptionValue { option: String, value: String },
    MissingOptionValue(String),
    MultipleFiles,
    MissingFile,
}

fn print_arg_error(json_mode: bool, lang: AppLanguage, err: ArgErrorType) -> i32 {
    if json_mode {
        let msg = match &err {
            ArgErrorType::UnrecognizedOption(s) => format!("unrecognized option '{}'", s),
            ArgErrorType::InvalidOptionValue { option, value } => {
                format!("invalid value '{}' for option '{}'", value, option)
            }
            ArgErrorType::MissingOptionValue(opt) => {
                format!("option '{}' requires a value", opt)
            }
            ArgErrorType::MultipleFiles => "multiple file arguments provided".to_string(),
            ArgErrorType::MissingFile => {
                "missing file argument; use --help for usage information".to_string()
            }
        };
        let env = ErrorEnvelope {
            schema_version: 1,
            error: ErrorDetails {
                code: "error.invalid_arguments".to_string(),
                message: msg,
            },
        };
        let json_err = serde_json::to_string(&env).unwrap_or_else(|_| {
            r#"{"schema_version":1,"error":{"code":"error.invalid_arguments","message":"argument error"}}"#.to_string()
        });
        println!("{}", json_err);
    } else {
        match (lang, &err) {
            (AppLanguage::Chinese, ArgErrorType::UnrecognizedOption(s)) => {
                eprintln!("错误: 未识别的选项 '{}'", s);
            }
            (AppLanguage::Chinese, ArgErrorType::InvalidOptionValue { option, value }) => {
                eprintln!("错误: 选项 '{}' 的参数值 '{}' 无效", option, value);
            }
            (AppLanguage::Chinese, ArgErrorType::MissingOptionValue(opt)) => {
                eprintln!("错误: 选项 '{}' 缺少参数值", opt);
            }
            (AppLanguage::Chinese, ArgErrorType::MultipleFiles) => {
                eprintln!("错误: 提供了多个文件参数");
            }
            (AppLanguage::Chinese, ArgErrorType::MissingFile) => {
                eprintln!("错误: 缺少文件参数；使用 --help 查看用法信息");
            }
            (AppLanguage::English, ArgErrorType::UnrecognizedOption(s)) => {
                eprintln!("Error: unrecognized option '{}'", s);
            }
            (AppLanguage::English, ArgErrorType::InvalidOptionValue { option, value }) => {
                eprintln!("Error: invalid value '{}' for option '{}'", value, option);
            }
            (AppLanguage::English, ArgErrorType::MissingOptionValue(opt)) => {
                eprintln!("Error: option '{}' requires a value", opt);
            }
            (AppLanguage::English, ArgErrorType::MultipleFiles) => {
                eprintln!("Error: multiple file arguments provided");
            }
            (AppLanguage::English, ArgErrorType::MissingFile) => {
                eprintln!("Error: missing file argument; use --help for usage information");
            }
        }
    }
    2
}

fn run(args: &[OsString], json_mode: bool, is_dedicated: bool, lang: AppLanguage) -> i32 {
    let cli_start = Instant::now();

    let mut target_files: Vec<PathBuf> = Vec::new();
    let mut after_double_dash = false;
    let mut batch_mode = false;
    let mut recursive = false;
    let mut jsonl_mode = false;
    let mut jobs: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if after_double_dash {
            if !batch_mode && !target_files.is_empty() {
                return print_arg_error(json_mode, lang, ArgErrorType::MultipleFiles);
            }
            target_files.push(PathBuf::from(arg));
        } else if arg == OsStr::new("--") {
            after_double_dash = true;
        } else if arg == OsStr::new("--json") {
            // Handled in pre-scan; in batch mode, --json emits JSONL
        } else if arg == OsStr::new("--batch") {
            batch_mode = true;
        } else if arg == OsStr::new("-r") || arg == OsStr::new("--recursive") {
            batch_mode = true;
            recursive = true;
        } else if arg == OsStr::new("--jsonl") {
            batch_mode = true;
            jsonl_mode = true;
        } else if arg == OsStr::new("-j") || arg == OsStr::new("--jobs") {
            let opt_str = arg.to_string_lossy().into_owned();
            i += 1;
            if i >= args.len() {
                return print_arg_error(json_mode, lang, ArgErrorType::MissingOptionValue(opt_str));
            }
            let val_str = args[i].to_string_lossy().into_owned();
            match val_str.parse::<usize>() {
                Ok(n) if n >= 1 => jobs = Some(n),
                _ => {
                    return print_arg_error(
                        json_mode,
                        lang,
                        ArgErrorType::InvalidOptionValue {
                            option: opt_str,
                            value: val_str,
                        },
                    );
                }
            }
        } else if arg == OsStr::new("-h") || arg == OsStr::new("--help") {
            print_help(lang);
            return 0;
        } else if arg == OsStr::new("-V") || arg == OsStr::new("--version") {
            print_version();
            return 0;
        } else {
            let s = arg.to_string_lossy();
            if s.starts_with('-') {
                return print_arg_error(
                    json_mode,
                    lang,
                    ArgErrorType::UnrecognizedOption(s.into_owned()),
                );
            } else {
                if !batch_mode && !target_files.is_empty() {
                    return print_arg_error(json_mode, lang, ArgErrorType::MultipleFiles);
                }
                target_files.push(PathBuf::from(arg));
            }
        }
        i += 1;
    }

    if batch_mode {
        if target_files.is_empty() {
            return print_arg_error(json_mode, lang, ArgErrorType::MissingFile);
        }
        return run_batch(
            &target_files,
            recursive,
            jsonl_mode || json_mode,
            jobs,
            lang,
        );
    }

    let file_path = match target_files.into_iter().next() {
        Some(f) => f,
        None => {
            if is_dedicated {
                match lang {
                    AppLanguage::Chinese => {
                        eprintln!(
                            "KindProbe v{} - 基于内容优先的文件格式识别工具\n\n\
                             将文件拖放到 kindprobe.exe 上即可识别其格式。\n\n\
                             用法:\n    \
                                 kindprobe [OPTIONS] <FILE>\n    \
                                 kindprobe [OPTIONS] -- <FILE>\n\n\
                             要查看命令行选项和语法，请运行: kindprobe --help",
                            VERSION
                        );
                    }
                    AppLanguage::English => {
                        eprintln!(
                            "KindProbe v{} - Content-first file format identification tool\n\n\
                             Drag and drop a file onto kindprobe.exe to identify its format.\n\n\
                             Usage:\n    \
                                 kindprobe [OPTIONS] <FILE>\n    \
                                 kindprobe [OPTIONS] -- <FILE>\n\n\
                             For command-line options and syntax, run: kindprobe --help",
                            VERSION
                        );
                    }
                }
                return 2;
            } else {
                return print_arg_error(json_mode, lang, ArgErrorType::MissingFile);
            }
        }
    };

    let path = file_path.as_path();
    let display_str = file_path.to_string_lossy();
    let bounded_display = if display_str.len() > 1024 {
        let mut end = 1024;
        while !display_str.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...[truncated]", &display_str[..end])
    } else {
        display_str.to_string()
    };

    let options = DetectOptions::default();

    match detect_path(path, &options) {
        Ok(report) => {
            let cli_elapsed_micros = cli_start.elapsed().as_micros() as u64;

            if json_mode {
                let serialized = match serde_json::to_string(&report) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Serialization error: {}", e);
                        return 1;
                    }
                };
                println!("{}", serialized);
            } else {
                match lang {
                    AppLanguage::Chinese => {
                        println!("文件: {}", bounded_display);
                        println!(
                            "识别结果: {} ({})",
                            format_identification_cn(report.identification),
                            report.format_id.as_deref().unwrap_or("none")
                        );
                        println!("类别: {}", format_category_cn(&report.category));
                        if let Some(arch) = extract_arch_display_cn(&report.evidence) {
                            println!("目标架构: {}", arch);
                        }
                        if let Some(cid) = &report.container_id {
                            println!("容器: {}", cid);
                        }
                        println!("验证方式: {}", format_verification_cn(report.verification));
                        println!("检测程度: {}", format_inspection_cn(report.inspection));
                        if let Some(rec) = &report.recommended_extension {
                            println!("推荐扩展名: .{}", rec);
                        }
                        println!(
                            "扩展名状态: {}",
                            format_extension_status_cn(report.extension_status)
                        );
                        if !report.evidence.is_empty() {
                            println!("证据:");
                            for ev in &report.evidence {
                                if let Some(cn) = format_evidence_explanation_cn(&ev.code) {
                                    println!("  - {} ({})", ev.code, cn);
                                } else if let Some(desc) = &ev.description {
                                    println!("  - {} ({})", ev.code, desc);
                                } else {
                                    println!("  - {}", ev.code);
                                }
                            }
                        }
                        if !report.warnings.is_empty() {
                            println!("警告:");
                            for w in &report.warnings {
                                println!(
                                    "  - [{}] {}",
                                    w.code,
                                    format_warning_explanation_cn(&w.code, &w.message)
                                );
                            }
                        }
                        println!("统计信息:");
                        println!("  读取源数据: {} 字节", report.stats.source_bytes_read);
                        println!("  寻道次数: {}", report.stats.seek_count);
                        println!("  检查条目数: {}", report.stats.entries_inspected);
                        println!("  解压数据: {} 字节", report.stats.decompressed_bytes);
                        println!("  引擎耗时: {} µs", report.stats.duration_micros);
                        println!("  CLI 总耗时: {} µs", cli_elapsed_micros);
                    }
                    AppLanguage::English => {
                        println!("File: {}", bounded_display);
                        println!(
                            "Identification: {:?} ({})",
                            report.identification,
                            report.format_id.as_deref().unwrap_or("none")
                        );
                        println!("Category: {}", report.category);
                        if let Some(arch) = extract_arch_display_en(&report.evidence) {
                            println!("Target Architecture: {}", arch);
                        }
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
                                if let Some(desc) = &ev.description {
                                    println!("  - {} ({})", ev.code, desc);
                                } else {
                                    println!("  - {}", ev.code);
                                }
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
                }
            }

            report.cli_exit_code()
        }
        Err(err) => {
            let (code_str, json_msg_str, exit_code) = match &err {
                DetectError::Io(e) => (
                    "io.open_failed",
                    format!("Failed to open or read file '{}': {}", bounded_display, e),
                    4,
                ),
                DetectError::NotRegularFile(_) => (
                    "io.not_regular_file",
                    format!(
                        "Path '{}' is not a regular file (e.g. directory or device)",
                        bounded_display
                    ),
                    4,
                ),
                DetectError::InvalidParameter(msg) => ("error.invalid_parameter", msg.clone(), 1),
            };

            if json_mode {
                let env = ErrorEnvelope {
                    schema_version: 1,
                    error: ErrorDetails {
                        code: code_str.to_string(),
                        message: json_msg_str,
                    },
                };
                let json_err = serde_json::to_string(&env).unwrap_or_else(|_| {
                    r#"{"schema_version":1,"error":{"code":"error.serialization_failure","message":"failed to serialize error envelope"}}"#.to_string()
                });
                println!("{}", json_err);
            } else {
                match lang {
                    AppLanguage::Chinese => {
                        let localized_msg = match &err {
                            DetectError::Io(e) => {
                                format!("打开或读取文件 '{}' 失败: {}", bounded_display, e)
                            }
                            DetectError::NotRegularFile(_) => {
                                format!("路径 '{}' 不是普通文件（例如目录或设备）", bounded_display)
                            }
                            DetectError::InvalidParameter(msg) => msg.clone(),
                        };
                        eprintln!("错误 [{}]: {}", code_str, localized_msg);
                    }
                    AppLanguage::English => {
                        eprintln!("Error [{}]: {}", code_str, json_msg_str);
                    }
                }
            }
            exit_code
        }
    }
}

fn main() {
    let args: Vec<OsString> = env::args_os().collect();

    // Pre-scan for --json flag so argument errors occurring before or after --json
    // can reliably emit the JSON ErrorEnvelope per specification §8
    let mut json_mode = false;
    let mut saw_double_dash = false;
    for arg in &args[1..] {
        if saw_double_dash {
            break;
        }
        if arg == OsStr::new("--") {
            saw_double_dash = true;
        } else if arg == OsStr::new("--json") || arg == OsStr::new("--jsonl") {
            json_mode = true;
        }
    }

    // Dedicated interactive console pause is active only when NOT in --json mode
    let is_dedicated = if json_mode {
        false
    } else {
        is_dedicated_interactive_console()
    };

    let lang = detect_app_language();
    let exit_code = run(&args, json_mode, is_dedicated, lang);

    if is_dedicated {
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        match lang {
            AppLanguage::Chinese => eprintln!("\n按 Enter 键关闭..."),
            AppLanguage::English => eprintln!("\nPress Enter to close..."),
        }
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }

    process::exit(exit_code);
}
