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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLanguage {
    English,
    Chinese,
}

pub fn is_chinese_win_langid(langid: u16) -> bool {
    let primary = langid & 0x03ff;
    primary == 0x04
}

pub fn is_chinese_locale_str(s: &str) -> bool {
    let s = s.trim().trim_matches('"').trim_matches('\'').trim();
    if s.is_empty() || s.eq_ignore_ascii_case("c") || s.eq_ignore_ascii_case("posix") {
        return false;
    }
    let lang = s
        .split(|c| c == '_' || c == '-' || c == '.' || c == '@')
        .next()
        .unwrap_or("");
    lang.eq_ignore_ascii_case("zh") || lang.eq_ignore_ascii_case("zho")
}

pub fn detect_posix_locale_from_env() -> Option<String> {
    std::env::var("LC_ALL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("LC_MESSAGES")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .or_else(|| {
            std::env::var("LANG")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
}

#[cfg(windows)]
mod platform_locale {
    use super::{
        detect_posix_locale_from_env, is_chinese_locale_str, is_chinese_win_langid, AppLanguage,
    };

    #[link(name = "kernel32")]
    extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }

    pub fn detect_app_language() -> AppLanguage {
        // Deterministic test hook for platform-specific language ID mapping without changing OS settings
        if let Some(val) = std::env::var_os("KINDPROBE_TEST_WIN_LANGID") {
            if let Ok(id) = val.to_string_lossy().parse::<u16>() {
                return if is_chinese_win_langid(id) {
                    AppLanguage::Chinese
                } else {
                    AppLanguage::English
                };
            }
        }

        // On Windows: POSIX environment variables (LC_ALL -> LC_MESSAGES -> LANG) take precedence if set
        if let Some(locale_str) = detect_posix_locale_from_env() {
            if is_chinese_locale_str(&locale_str) {
                return AppLanguage::Chinese;
            } else {
                return AppLanguage::English;
            }
        }

        // Win32 GetUserDefaultUILanguage() retrieves the current user's native UI language preference
        let langid = unsafe { GetUserDefaultUILanguage() };
        if is_chinese_win_langid(langid) {
            AppLanguage::Chinese
        } else {
            AppLanguage::English
        }
    }
}

#[cfg(not(windows))]
mod platform_locale {
    use super::{detect_posix_locale_from_env, is_chinese_locale_str, is_chinese_win_langid, AppLanguage};

    pub fn detect_app_language() -> AppLanguage {
        // Deterministic test hook for platform-specific language ID mapping on non-Windows platforms
        if let Some(val) = std::env::var_os("KINDPROBE_TEST_WIN_LANGID") {
            if let Ok(id) = val.to_string_lossy().parse::<u16>() {
                return if is_chinese_win_langid(id) {
                    AppLanguage::Chinese
                } else {
                    AppLanguage::English
                };
            }
        }

        // On Linux/POSIX platforms: locale precedence LC_ALL -> LC_MESSAGES -> LANG
        if let Some(locale_str) = detect_posix_locale_from_env() {
            if is_chinese_locale_str(&locale_str) {
                return AppLanguage::Chinese;
            }
        }

        AppLanguage::English
    }
}

fn detect_app_language() -> AppLanguage {
    platform_locale::detect_app_language()
}

#[cfg(windows)]
mod win_console {
    type DWORD = u32;
    type BOOL = i32;
    type HANDLE = *mut std::ffi::c_void;
    type HWND = *mut std::ffi::c_void;

    const STD_OUTPUT_HANDLE: DWORD = 0xfffffff5; // -11i32 as u32
    const STD_INPUT_HANDLE: DWORD = 0xfffffff6;  // -10i32 as u32

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
        std::env::var_os("KINDPROBE_TEST_DEDICATED_CONSOLE").map_or(false, |v| v == "1")
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

参数:
    <FILE>    待检测文件的路径

选项:
    --json        向标准输出输出机器可读的 JSON v1
    -h, --help    打印帮助信息
    -V, --version 打印版本信息

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

ARGS:
    <FILE>    Path to the file to inspect

OPTIONS:
    --json        Output machine-readable JSON v1 to stdout
    -h, --help    Print help information
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
    }
}

fn print_version() {
    println!("kindprobe {}", VERSION);
}

enum ArgErrorType {
    UnrecognizedOption(String),
    MultipleFiles,
    MissingFile,
}

fn print_arg_error(json_mode: bool, lang: AppLanguage, err: ArgErrorType) -> i32 {
    if json_mode {
        let msg = match &err {
            ArgErrorType::UnrecognizedOption(s) => format!("unrecognized option '{}'", s),
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
        let json_err = serde_json::to_string(&env).unwrap();
        println!("{}", json_err);
    } else {
        match (lang, &err) {
            (AppLanguage::Chinese, ArgErrorType::UnrecognizedOption(s)) => {
                eprintln!("错误: 未识别的选项 '{}'", s);
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

fn format_identification_cn(id: kindprobe::model::Identification) -> &'static str {
    match id {
        kindprobe::model::Identification::Format => "格式已确认",
        kindprobe::model::Identification::ContainerOnly => "仅识别容器",
        kindprobe::model::Identification::Unknown => "未知格式",
    }
}

fn format_category_cn(cat: &str) -> &'static str {
    match cat {
        "image" => "图像",
        "document" => "文档",
        "container" => "容器",
        "unknown" => "未知",
        _ => "未知",
    }
}

fn format_verification_cn(v: kindprobe::model::Verification) -> &'static str {
    match v {
        kindprobe::model::Verification::Signature => "特征签名",
        kindprobe::model::Verification::Structure => "内部结构",
        kindprobe::model::Verification::None => "无",
    }
}

fn format_inspection_cn(i: kindprobe::model::Inspection) -> &'static str {
    match i {
        kindprobe::model::Inspection::Complete => "完整",
        kindprobe::model::Inspection::Limited => "受限",
        kindprobe::model::Inspection::Ambiguous => "存在歧义",
    }
}

fn format_extension_status_cn(es: kindprobe::model::ExtensionStatus) -> &'static str {
    match es {
        kindprobe::model::ExtensionStatus::Match => "匹配",
        kindprobe::model::ExtensionStatus::Mismatch => "不匹配",
        kindprobe::model::ExtensionStatus::Missing => "缺失",
        kindprobe::model::ExtensionStatus::Unrecognized => "未识别",
        kindprobe::model::ExtensionStatus::Undetermined => "待定",
    }
}

fn format_warning_explanation_cn<'a>(code: &str, default_msg: &'a str) -> &'a str {
    match code {
        "budget.source_read_exceeded" => "文件大小超过源读取预算限制",
        "budget.entries_exceeded" => "检查条目数超过最大预算限制",
        "budget.total_decompressed_exceeded" => "解压缩总数据量超过预算限制",
        "budget.xml_depth_exceeded" => "XML 嵌套深度超过预算限制",
        "budget.xml_output_exceeded" => "XML 解压或解析输出量超过预算限制",
        "budget.zip_directory_window_exceeded" => "Zip 核心目录搜索窗口超过预算限制",
        "format.unknown" => "未匹配到任何受支持的格式签名或结构",
        "input.empty" => "输入文件大小为 0 字节",
        "input.truncated" => "文件在达到最小所需文件头字节数前被截断",
        "cfb.corrupted_or_invalid_chain" => "CFB 头部或扇区链损坏或无效",
        "cfb.document_encrypted" => "CFB 复合文档包含加密流或指示加密/混淆",
        "cfb.fib_word97" => "CFB 识别到 Word97+ FibBase 结构",
        "cfb.header" => "CFB 文件头签名有效",
        "cfb.invalid_word_fib" => "WordDocument 流过小或 Fib 魔数不匹配",
        "cfb.stream_open_error" => "打开 CFB 内部流失败",
        "cfb.stream_read_error" => "读取 CFB 内部流失败",
        "cfb.table_stream_missing" => "CFB 容器中缺少必需的表流 (Table Stream)",
        "cfb.table_stream_open_error" => "打开 CFB 表流失败",
        "cfb.table_stream_read_error" => "读取 CFB 表流失败",
        "cfb.unsupported_word_version" => "不受支持或过期的 Word Fib 版本",
        "cfb.word_document_stream" => "WordDocument 流结构检查",
        "cfb.workbook_stream" => "CFB 识别到 Excel Workbook/Book 流",
        "cfb.biff_record" => "CFB 验证 Excel BIFF5/BIFF8 BOF 记录结构",
        "cfb.invalid_workbook_stream" => "CFB Workbook 流过小或无法包含 BOF 记录",
        "cfb.invalid_biff_bof" => "CFB BIFF BOF 记录类型无效",
        "cfb.unsupported_biff_version" => "不受支持的 Excel BIFF 版本",
        "cfb.powerpoint_stream" => "CFB 识别到 PowerPoint Document 流",
        "cfb.invalid_powerpoint_stream" => "CFB PowerPoint Document 流过小",
        "cfb.current_user_stream" => "CFB 识别到 PowerPoint Current User 流",
        "cfb.directory" => "CFB 目录扇区分配或解析错误",
        "zip.central_directory" => "Zip 核心目录解析错误",
        "zip.conflicting_entries" => "Zip 中发现冲突或重复的条目名称",
        "zip.truncated_or_corrupted" => "解析 Zip 压缩包或查找核心目录失败",
        "ooxml.content_type_not_found" => "未找到对应部件的内容类型",
        "ooxml.content_types" => "未找到有效的 openxmlformats 内容类型覆盖项",
        "ooxml.content_types_missing" => "Zip 容器中缺少 [Content_Types].xml",
        "ooxml.content_types_read_error" => "读取 [Content_Types].xml 失败",
        "ooxml.content_types_xml_error" => "解析 [Content_Types].xml XML 结构错误",
        "ooxml.invalid_root_element" => "OOXML 主部件根元素或命名空间无效",
        "ooxml.invalid_target_path" => "关系目标路径解析到 Zip 根目录外或无效",
        "ooxml.main_part_missing" => "Zip 压缩包中缺少主文档部件",
        "ooxml.main_part_read_error" => "读取主文档部件失败",
        "ooxml.main_part_xml_error" => "解析主文档部件 XML 结构错误",
        "ooxml.non_word_package" => "OOXML 包根元素或命名空间与 Word 文档不匹配",
        "ooxml.package_relationships" => "缺少 _rels/.rels 包关系部件",
        "ooxml.rels_read_error" => "读取关系部件失败",
        "ooxml.rels_xml_error" => "解析关系部件 XML 结构错误",
        "ooxml.required_relationship_missing" => "缺少 officeDocument 核心包关系",
        "ooxml.unsupported_content_type" => "OOXML 主部件内容类型不受支持或无法确认为目标格式",
        "ooxml.word_main_part" => "Word 主文档部件及命名空间验证有效",
        "ooxml.word_macro_main_part" => "Word 宏文档主部件及命名空间验证有效",
        "ooxml.spreadsheet_main_part" => "Excel 工作簿主部件及命名空间验证有效",
        "ooxml.spreadsheet_macro_main_part" => "Excel 宏工作簿主部件及命名空间验证有效",
        "ooxml.presentation_main_part" => "PowerPoint 演示文稿主部件及命名空间验证有效",
        "ooxml.presentation_macro_main_part" => "PowerPoint 宏演示文稿主部件及命名空间验证有效",
        "sig.jpeg.soi" => "JPEG SOI 起始签名有效",
        "sig.jpeg.segment" => "JPEG 标记段（APP/DQT/SOF）结构有效",
        "sig.gif.header" => "GIF 87a/89a 文件头签名有效",
        "sig.gif.logical_screen" => "GIF 逻辑屏幕描述符有效",
        "sig.7z.header" => "7-Zip 魔数签名有效",
        "sig.7z.start_header" => "7-Zip 起始头 CRC32 校验有效",
        "sig.rar.header" => "RAR 归档签名有效",
        "sig.rar.main_header" => "RAR4 主档案头有效",
        "sig.rar5.signature" => "RAR5 扩展归档签名有效",
        _ => default_msg,
    }
}

fn run(args: &[OsString], json_mode: bool, is_dedicated: bool, lang: AppLanguage) -> i32 {
    let cli_start = Instant::now();

    let mut target_file: Option<PathBuf> = None;
    let mut after_double_dash = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if after_double_dash {
            if target_file.is_some() {
                return print_arg_error(json_mode, lang, ArgErrorType::MultipleFiles);
            }
            target_file = Some(PathBuf::from(arg));
        } else if arg == OsStr::new("--") {
            after_double_dash = true;
        } else if arg == OsStr::new("--json") {
            // Handled in pre-scan
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
                if target_file.is_some() {
                    return print_arg_error(json_mode, lang, ArgErrorType::MultipleFiles);
                }
                target_file = Some(PathBuf::from(arg));
            }
        }
        i += 1;
    }

    let file_path = match target_file {
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
                                println!("  - {}", ev.code);
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
                let json_err = serde_json::to_string(&env).unwrap();
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
        } else if arg == OsStr::new("--json") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chinese_win_langid() {
        // Chinese variants:
        assert!(is_chinese_win_langid(0x0804)); // zh-CN
        assert!(is_chinese_win_langid(0x0404)); // zh-TW
        assert!(is_chinese_win_langid(0x0c04)); // zh-HK
        assert!(is_chinese_win_langid(0x1004)); // zh-SG
        assert!(is_chinese_win_langid(0x1404)); // zh-MO
        assert!(is_chinese_win_langid(0x7804)); // zh-Hans
        assert!(is_chinese_win_langid(0x7c04)); // zh-Hant

        // English & Non-Chinese:
        assert!(!is_chinese_win_langid(0x0409)); // en-US
        assert!(!is_chinese_win_langid(0x0809)); // en-GB
        assert!(!is_chinese_win_langid(0x0411)); // ja-JP
        assert!(!is_chinese_win_langid(0x0407)); // de-DE
        assert!(!is_chinese_win_langid(0x040c)); // fr-FR

        // Unsupported / Failure:
        assert!(!is_chinese_win_langid(0x0000));
        assert!(!is_chinese_win_langid(0xffff));
    }

    #[test]
    fn test_is_chinese_locale_str() {
        // Positive Chinese variants
        assert!(is_chinese_locale_str("zh_CN.UTF-8"));
        assert!(is_chinese_locale_str("zh_TW.UTF-8"));
        assert!(is_chinese_locale_str("zh_HK.UTF-8"));
        assert!(is_chinese_locale_str("zh_SG.GBK"));
        assert!(is_chinese_locale_str("zh_MO"));
        assert!(is_chinese_locale_str("zh-CN"));
        assert!(is_chinese_locale_str("zh-TW"));
        assert!(is_chinese_locale_str("zh"));
        assert!(is_chinese_locale_str("ZH_CN.UTF-8"));
        assert!(is_chinese_locale_str("zh_CN.gb18030"));
        assert!(is_chinese_locale_str("\"zh_CN.UTF-8\""));

        // Negative non-Chinese / C / POSIX / Empty / Unknown
        assert!(!is_chinese_locale_str("C"));
        assert!(!is_chinese_locale_str("POSIX"));
        assert!(!is_chinese_locale_str("C.UTF-8"));
        assert!(!is_chinese_locale_str("en_US.UTF-8"));
        assert!(!is_chinese_locale_str("en_GB"));
        assert!(!is_chinese_locale_str("ja_JP.UTF-8"));
        assert!(!is_chinese_locale_str("de_DE"));
        assert!(!is_chinese_locale_str("fr_FR"));
        assert!(!is_chinese_locale_str(""));
        assert!(!is_chinese_locale_str("   "));
        assert!(!is_chinese_locale_str("unknown"));
    }
}
