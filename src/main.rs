use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::model::{DetectError, ErrorDetails, ErrorEnvelope};
use kindprobe::{detect_path, detect_path_core, Identification};

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
    let lang = s.split(['_', '-', '.', '@']).next().unwrap_or("");
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
        .or_else(|| std::env::var("LANG").ok().filter(|s| !s.trim().is_empty()))
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
        // SAFETY: Win32 GetUserDefaultUILanguage takes no arguments, performs no memory
        // mutations or pointer dereferences, reads only user locale from the OS kernel,
        // and returns a scalar u16 integer (LANGID). It cannot cause memory corruption or UB.
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
    use super::{
        detect_posix_locale_from_env, is_chinese_locale_str, is_chinese_win_langid, AppLanguage,
    };

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

fn format_identification_cn(id: kindprobe::model::Identification) -> &'static str {
    match id {
        kindprobe::model::Identification::Format => "格式已确认",
        kindprobe::model::Identification::ContainerOnly => "仅识别容器",
        kindprobe::model::Identification::Unknown => "未知格式",
    }
}

fn format_category_cn(cat: &str) -> &'static str {
    match cat {
        "archive" => "归档压缩",
        "audio" => "音频",
        "automotive" => "车载总线",
        "bioinformatics" => "生物信息",
        "blockchain" => "区块链",
        "bytecode" => "字节码",
        "cad" => "CAD/工业设计",
        "calendar" => "日程",
        "capture" => "网络抓包",
        "code" | "source" => "源码与脚本",
        "configuration" => "配置文件",
        "contact" => "联系人",
        "container" => "容器",
        "crypto" => "加密与证书",
        "data" => "结构化数据",
        "database" => "数据库",
        "disk" | "disk_image" => "磁盘镜像",
        "document" => "文档",
        "ebook" => "电子书",
        "emulation" => "仿真器镜像",
        "engineering" => "工程数据",
        "exchange" => "数据交换",
        "executable" => "可执行文件",
        "finance" => "金融票据",
        "firmware" => "固件",
        "font" => "字体",
        "forensics" => "数字取证",
        "game" | "game_asset" => "游戏资源",
        "geospatial" | "gis" => "地理空间",
        "hardware" => "硬件描述",
        "image" => "图像",
        "location" => "位置数据",
        "material" => "材质数据",
        "media" | "multimedia" => "多媒体",
        "model" => "三维与AI模型",
        "navigation" => "导航数据",
        "network" => "网络数据",
        "package" => "安装包",
        "robotics" => "机器人数据",
        "rom" => "ROM固件",
        "schema" => "数据模式",
        "scientific" => "科学数据",
        "security" => "安全凭据",
        "system" => "系统文件",
        "telecom" => "通信数据",
        "text" => "文本与源码",
        "texture" => "纹理贴图",
        "translation" => "本地化翻译",
        "video" => "视频",
        "virtualization" => "虚拟化镜像",
        "web" => "Web资源",
        "unknown" => "未知",
        _ => "未知",
    }
}

fn format_evidence_explanation_cn(code: &str) -> Option<&'static str> {
    match code {
        "sig.pe.arch.x86" => Some("目标架构: x86 (Intel 386 / 32位)"),
        "sig.pe.arch.x64" => Some("目标架构: x86-64 (AMD64 / 64位)"),
        "sig.pe.arch.arm" => Some("目标架构: ARM (32位小端)"),
        "sig.pe.arch.arm_thumb2" => Some("目标架构: ARMv7 Thumb-2 (Windows RT)"),
        "sig.pe.arch.arm64" => Some("目标架构: ARM64 (AArch64 / 64位)"),
        "sig.pe.arch.arm64ec" => Some("目标架构: ARM64EC (仿真兼容)"),
        "sig.pe.arch.ia64" => Some("目标架构: Intel Itanium (IA-64)"),
        "sig.pe.arch.riscv32" => Some("目标架构: RISC-V 32位"),
        "sig.pe.arch.riscv64" => Some("目标架构: RISC-V 64位"),
        "sig.pe.arch.riscv128" => Some("目标架构: RISC-V 128位"),
        "sig.pe.opt.pe32" => Some("PE32 (32位寻址空间)"),
        "sig.pe.opt.pe32plus" => Some("PE32+ (64位寻址空间)"),
        "sig.pe.subsystem.gui" => Some("Windows 图形界面子系统 (GUI)"),
        "sig.pe.subsystem.console" => Some("Windows 控制台子系统 (CUI)"),
        "sig.elf.arch.x86" => Some("目标架构: x86 (Intel 386 / 32位)"),
        "sig.elf.arch.x86_64" => Some("目标架构: x86-64 (AMD64 / 64位)"),
        "sig.elf.arch.arm" => Some("目标架构: ARM (32位)"),
        "sig.elf.arch.arm64" => Some("目标架构: ARM64 (AArch64 / 64位)"),
        "sig.elf.arch.riscv" => Some("目标架构: RISC-V"),
        "sig.elf.class.32bit" => Some("ELF 32位"),
        "sig.elf.class.64bit" => Some("ELF 64位"),
        "sig.elf.type.executable" => Some("ELF 可执行文件 (ET_EXEC)"),
        "sig.elf.type.shared_object" => Some("ELF 动态共享库 (ET_DYN)"),
        "sig.macho.arch.x86" => Some("目标架构: x86 (i386)"),
        "sig.macho.arch.x86_64" => Some("目标架构: x86-64 (AMD64)"),
        "sig.macho.arch.arm" => Some("目标架构: ARM (32位)"),
        "sig.macho.arch.arm64" => Some("目标架构: ARM64 (Apple Silicon)"),
        "sig.macho.universal" => Some("通用二进制 (多架构 Universal Binary)"),
        "sig.tar.embedded" => Some("在压缩数据流中检测到内嵌 POSIX ustar 归档"),
        "sig.tar.ustar" => Some("POSIX tar (ustar) 头部签名有效"),
        "sig.tar.checksum" => Some("POSIX tar 头部校验和验证有效"),
        _ => None,
    }
}

fn extract_arch_display_cn(evidence: &[kindprobe::model::EvidenceItem]) -> Option<String> {
    for ev in evidence {
        if ev.code.contains(".arch.") || ev.code == "sig.macho.universal" {
            if let Some(cn) = format_evidence_explanation_cn(&ev.code) {
                return Some(cn.trim_start_matches("目标架构: ").to_string());
            }
            if let Some(desc) = &ev.description {
                return Some(desc.trim_start_matches("Target Architecture: ").to_string());
            }
        }
    }
    None
}

fn extract_arch_display_en(evidence: &[kindprobe::model::EvidenceItem]) -> Option<String> {
    for ev in evidence {
        if ev.code.contains(".arch.") || ev.code == "sig.macho.universal" {
            if let Some(desc) = &ev.description {
                return Some(desc.trim_start_matches("Target Architecture: ").to_string());
            }
        }
    }
    None
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
        "ooxml.conflicting_relationships" => {
            "OOXML _rels/.rels 中存在多个冲突的 officeDocument 核心关系"
        }
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
        "sig.mp4.ftyp" => "ISO-BMFF / MP4 ftyp 结构有效",
        "sig.mp4.brand" => "MP4 major/compatible brand 标识有效",
        "sig.mov.brand" => "QuickTime MOV 品牌标识有效",
        "sig.mov.header" => "QuickTime 经典 moov/wide/free 头部有效",
        "sig.ebml.header" => "EBML 头部签名有效",
        "sig.ebml.doctype_webm" => "WebM DocType 标识有效",
        "sig.ebml.doctype_mkv" => "Matroska DocType 标识有效",
        "sig.riff.header" => "RIFF 容器头部有效",
        "sig.wav.wave_header" => "WAVE 格式标识有效",
        "sig.wav.fmt_chunk" => "WAV fmt 音频格式块结构有效",
        "sig.webp.header" => "WebP 格式标识有效",
        "sig.webp.vp8" => "WebP VP8 编码数据块有效",
        "sig.avi.header" => "AVI 格式标识有效",
        "sig.id3.tag" => "ID3v2 元数据标签有效",
        "sig.mp3.frame_sync" => "MPEG Audio Layer III 帧同步有效",
        "sig.elf.magic" => "ELF 魔数签名有效",
        "sig.elf.header" => "ELF 32/64 位体系结构与版本有效",
        "sig.pe.mz_header" => "DOS MZ 文件头签名有效",
        "sig.pe.signature" => "Windows PE 头部签名有效",
        "sig.wasm.magic" => "WebAssembly 魔数签名有效",
        "sig.wasm.version" => "WebAssembly 版本号结构有效",
        "sig.macho.magic" => "Mach-O 单架构魔数签名有效",
        "sig.macho.fat_magic" => "Mach-O 通用二进制 (FAT) 魔数有效",
        "sig.java.class" => "Java Class 字节码结构签名有效",
        "sig.font.ttf_magic" => "TrueType 字体魔数有效",
        "sig.font.otf_magic" => "OpenType 字体魔数有效",
        "sig.font.num_tables" => "字体表目录表数量结构有效",
        "sig.font.woff_magic" => "WOFF 1.0 魔数有效",
        "sig.font.woff_header" => "WOFF 1.0 头部表结构有效",
        "sig.font.woff2_magic" => "WOFF 2.0 魔数有效",
        "sig.font.woff2_header" => "WOFF 2.0 头部表结构有效",
        "sig.mobi.header" => "Mobipocket / Kindle BOOKMOBI 头部有效",
        "sig.palmdoc.header" => "PalmDOC BOOKTEXt 头部有效",
        "epub.mimetype" => "EPUB mimetype 未压缩首条目校验有效",
        "epub.container" => "EPUB META-INF/container.xml 结构有效",
        "odf.mimetype" => "OpenDocument mimetype 校验有效",
        "sig.sqlite.header" => "SQLite 3 数据库文件头魔数有效",
        "sig.sqlite.page_size" => "SQLite 3 页大小与读写格式版本有效",
        "sig.pcap.magic" => "PCAP 数据包转储魔数有效",
        "sig.pcap.version" => "PCAP 2.4 主次版本号有效",
        "sig.pcapng.shb" => "PCAPNG 分段头部块 (SHB) 结构有效",
        "sig.pcapng.bom" => "PCAPNG 字节序魔数 (BOM) 有效",
        "sig.parquet.magic" => "Apache Parquet 列式数据魔数有效",
        "sig.avro.magic" => "Apache Avro 容器魔数有效",
        "sig.qcow.magic" => "QEMU Copy-On-Write (QCOW) 魔数有效",
        "sig.qcow.header" => "QCOW2/QCOW3 版本与聚簇位有效",
        "sig.vmdk.sparse_magic" => "VMware 虚拟磁盘 (Sparse VMDK) 魔数有效",
        "sig.vmdk.version" => "VMware VMDK 版本有效",
        "sig.vmdk.desc_magic" => "VMware 文本描述符 (Descriptor VMDK) 头部有效",
        "sig.vhdx.magic" => "微软虚拟硬盘 (VHDX) 签名有效",
        "sig.vhd.magic" => "微软虚拟硬盘 (VHD) 签名有效",
        "sig.iso.volume_descriptor" => "ISO 9660 主卷描述符 (PVD) 签名有效",
        "sig.tar.ustar" => "POSIX tar (ustar) 头部签名有效",
        "sig.tar.checksum" => "POSIX tar 头部校验和 (Checksum) 验证有效",
        "sig.tar.embedded" => "在压缩数据流中检测到内嵌 POSIX ustar 归档",
        "sig.gzip.magic" => "Gzip 压缩魔数签名有效",
        "sig.gzip.header" => "Gzip 头部（DEFLATE 压缩方法与保留标志）有效",
        "sig.bzip2.magic" => "Bzip2 压缩魔数签名有效",
        "sig.bzip2.header" => "Bzip2 块大小与 Pi 常数块特征有效",
        "sig.xz.magic" => "XZ 压缩魔数签名有效",
        "sig.xz.crc" => "XZ Stream Flags CRC32 校验有效",
        "sig.zstd.magic" => "Zstandard 压缩魔数签名有效",
        "sig.text.json" => "JSON 文档语法结构有效",
        "sig.text.xml" => "XML 处理指令或文档根标记有效",
        "sig.text.html" => "HTML DOCTYPE 或核心标签结构有效",
        "sig.text.yaml" => "YAML 文档分隔符或键值结构有效",
        "sig.text.toml" => "TOML 表头段落或键值赋值结构有效",
        "sig.text.csv" => "CSV 逗号分隔表格结构与字段一致性有效",
        "sig.text.tsv" => "TSV 制表符分隔表格结构与字段一致性有效",
        "sig.text.plain" => "UTF-8 纯文本编码及控制字符过滤有效",
        "sig.deb.magic" => "Debian 软件包 (deb) ar 头部与 debian-binary 签名有效",
        "sig.rpm.magic" => "Red Hat 软件包 (rpm) 头部魔数与主版本有效",
        "sig.crx.magic" => "Chrome 扩展包 (crx) 头部魔数与版本有效",
        "sig.flac.magic" => "FLAC 无损音频魔数与 STREAMINFO 头部有效",
        "sig.ogg.magic" => "Ogg 多媒体容器头部签名有效",
        "sig.psd.magic" => "Adobe Photoshop (PSD) 头部魔数与版本有效",
        "sig.tiff.magic" => "TIFF 图像字节序魔数与 IFD 偏移有效",
        "sig.ico.magic" => "Windows 图标 (ICO) 头部签名有效",
        "sig.cur.magic" => "Windows 光标 (CUR) 头部签名有效",
        "zip.apk.manifest" => "Android 安装包 (APK) 清单文件 (AndroidManifest.xml) 有效",
        "zip.apk.dex" => "Android 安装包 (APK) 字节码文件 (classes.dex) 有效",
        "zip.ipa.payload" => "iOS 安装包 (IPA) Payload 应用程序清单有效",
        "zip.jar.manifest" => "Java 归档包 (JAR) 清单文件 (META-INF/MANIFEST.MF) 有效",
        "sig.aac.magic" => "AAC 音频 ADTS 帧同步魔数有效",
        "sig.aiff.magic" => "AIFF 音频 FORM 容器与 AIFF/AIFC 标识有效",
        "sig.mid.magic" => "MIDI 音频 MThd 头部魔数与长度有效",
        "sig.chm.magic" => "微软已编译 HTML 帮助 (CHM) 头部签名有效",
        _ => default_msg,
    }
}

#[derive(serde::Serialize)]
struct BatchResultJson<'a> {
    path: &'a str,
    status: &'static str,
    format_id: Option<&'static str>,
    category: &'static str,
    container_id: Option<&'static str>,
    identification: Identification,
    recommended_extension: Option<&'static str>,
}

#[derive(serde::Serialize)]
struct BatchErrorJson<'a> {
    path: &'a str,
    status: &'static str,
    error_code: &'static str,
    message: String,
}

fn error_code_for_detect_error(err: &DetectError) -> &'static str {
    match err {
        DetectError::NotRegularFile(_) => "io.not_regular_file",
        DetectError::InvalidParameter(_) => "arg.invalid_parameter",
        DetectError::Io(io_err) => match io_err.kind() {
            std::io::ErrorKind::NotFound => "io.not_found",
            std::io::ErrorKind::PermissionDenied => "io.permission_denied",
            _ => "io.read_error",
        },
    }
}

fn process_batch_file(
    path: &std::path::Path,
    options: &DetectOptions,
    jsonl_mode: bool,
    lang: AppLanguage,
    out: &mut impl Write,
    err_out: &mut impl Write,
) -> (usize, usize, usize) {
    let path_str = path.to_string_lossy();
    match detect_path_core(path, options) {
        Ok(core) => {
            if jsonl_mode {
                let _ = serde_json::to_writer(
                    &mut *out,
                    &BatchResultJson {
                        path: &path_str,
                        status: "ok",
                        format_id: core.format_id,
                        category: core.category,
                        container_id: core.container_id,
                        identification: core.identification,
                        recommended_extension: core.recommended_extension,
                    },
                );
                let _ = out.write_all(b"\n");
            } else {
                let fmt = core.format_id.unwrap_or("unknown");
                match lang {
                    AppLanguage::Chinese => {
                        let _ = writeln!(
                            out,
                            "{} -> {} ({})",
                            path_str,
                            fmt,
                            format_category_cn(core.category)
                        );
                    }
                    AppLanguage::English => {
                        let _ = writeln!(out, "{} -> {} ({})", path_str, fmt, core.category);
                    }
                }
            }
            if core.identification == Identification::Format {
                (1, 0, 0)
            } else {
                (0, 1, 0)
            }
        }
        Err(err) => {
            let code = error_code_for_detect_error(&err);
            if jsonl_mode {
                let _ = serde_json::to_writer(
                    &mut *out,
                    &BatchErrorJson {
                        path: &path_str,
                        status: "error",
                        error_code: code,
                        message: err.to_string(),
                    },
                );
                let _ = out.write_all(b"\n");
            } else {
                match lang {
                    AppLanguage::Chinese => {
                        let _ = writeln!(err_out, "错误 [{}]: {} ({})", code, path_str, err);
                    }
                    AppLanguage::English => {
                        let _ = writeln!(err_out, "Error [{}]: {} ({})", code, path_str, err);
                    }
                }
            }
            (0, 0, 1)
        }
    }
}

fn run_batch(
    paths: &[PathBuf],
    recursive: bool,
    jsonl_mode: bool,
    jobs: Option<usize>,
    lang: AppLanguage,
) -> i32 {
    let start = Instant::now();
    let options = DetectOptions::default();
    let stdout = std::io::stdout();
    let mut out = BufWriter::with_capacity(64 * 1024, stdout.lock());

    let mut total_files: usize = 0;
    let mut identified: usize = 0;
    let mut unknown: usize = 0;
    let mut errors: usize = 0;
    let mut file_paths: Vec<PathBuf> = Vec::new();

    // Explicit stack of (path, is_root_arg) so `--batch <dir>` scans 1 level even when !recursive
    let mut work_stack: Vec<(PathBuf, bool)> =
        paths.iter().rev().map(|p| (p.clone(), true)).collect();

    while let Some((path, is_root)) = work_stack.pop() {
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                total_files += 1;
                errors += 1;
                let path_str = path.to_string_lossy();
                let code = error_code_for_detect_error(&DetectError::Io(std::io::Error::new(
                    e.kind(),
                    e.to_string(),
                )));
                if jsonl_mode {
                    let _ = serde_json::to_writer(
                        &mut out,
                        &BatchErrorJson {
                            path: &path_str,
                            status: "error",
                            error_code: code,
                            message: e.to_string(),
                        },
                    );
                    let _ = out.write_all(b"\n");
                } else {
                    match lang {
                        AppLanguage::Chinese => {
                            eprintln!("错误 [{}]: {} ({})", code, path_str, e);
                        }
                        AppLanguage::English => {
                            eprintln!("Error [{}]: {} ({})", code, path_str, e);
                        }
                    }
                }
                continue;
            }
        };

        let is_symlink_dir = meta.file_type().is_symlink() && path.is_dir();
        if is_symlink_dir {
            continue;
        }

        if meta.is_dir() && (recursive || is_root) {
            match std::fs::read_dir(&path) {
                Ok(rd) => {
                    let mut children: Vec<PathBuf> = rd
                        .filter_map(|res| res.ok())
                        .filter_map(|entry| {
                            let ft = entry.file_type().ok()?;
                            let child_path = entry.path();
                            // Skip symlinked directories to avoid traversal cycles
                            if ft.is_symlink() && child_path.is_dir() {
                                return None;
                            }
                            if !recursive && ft.is_dir() {
                                return None;
                            }
                            Some(child_path)
                        })
                        .collect();
                    children.sort();
                    for child in children.into_iter().rev() {
                        work_stack.push((child, false));
                    }
                }
                Err(e) => {
                    total_files += 1;
                    errors += 1;
                    let path_str = path.to_string_lossy();
                    if jsonl_mode {
                        let _ = serde_json::to_writer(
                            &mut out,
                            &BatchErrorJson {
                                path: &path_str,
                                status: "error",
                                error_code: "io.read_error",
                                message: e.to_string(),
                            },
                        );
                        let _ = out.write_all(b"\n");
                    } else {
                        match lang {
                            AppLanguage::Chinese => {
                                eprintln!("错误 [io.read_error]: {} ({})", path_str, e);
                            }
                            AppLanguage::English => {
                                eprintln!("Error [io.read_error]: {} ({})", path_str, e);
                            }
                        }
                    }
                }
            }
            continue;
        }

        file_paths.push(path);
    }

    total_files += file_paths.len();
    let default_jobs = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, 8);
    let num_threads = jobs
        .unwrap_or(default_jobs)
        .clamp(1, file_paths.len().max(1))
        .min(16);

    if file_paths.len() < 32 || num_threads <= 1 {
        let mut stderr = std::io::stderr();
        for path in &file_paths {
            let (id_inc, unk_inc, err_inc) =
                process_batch_file(path, &options, jsonl_mode, lang, &mut out, &mut stderr);
            identified += id_inc;
            unknown += unk_inc;
            errors += err_inc;
        }
    } else {
        #[derive(Default)]
        struct BatchChunkResult {
            id_cnt: usize,
            unk_cnt: usize,
            err_cnt: usize,
            local_out: Vec<u8>,
            local_err: Vec<u8>,
        }

        let chunk_size = file_paths.len().div_ceil(num_threads);
        let chunk_results: Vec<Result<BatchChunkResult, usize>> = std::thread::scope(|s| {
            let handles: Vec<_> = file_paths
                .chunks(chunk_size)
                .map(|chunk| {
                    let opts_ref = &options;
                    let chunk_len = chunk.len();
                    s.spawn(move || {
                        let mut res = BatchChunkResult {
                            local_out: Vec::with_capacity(chunk.len() * 128),
                            ..BatchChunkResult::default()
                        };
                        for path in chunk {
                            let (i, u, e) = process_batch_file(
                                path,
                                opts_ref,
                                jsonl_mode,
                                lang,
                                &mut res.local_out,
                                &mut res.local_err,
                            );
                            res.id_cnt += i;
                            res.unk_cnt += u;
                            res.err_cnt += e;
                        }
                        res
                    })
                    .join()
                    .map_err(|_| chunk_len)
                })
                .collect();
            handles
        });

        let mut stderr = std::io::stderr();
        for res in chunk_results {
            match res {
                Ok(chunk_res) => {
                    identified += chunk_res.id_cnt;
                    unknown += chunk_res.unk_cnt;
                    errors += chunk_res.err_cnt;
                    let _ = out.write_all(&chunk_res.local_out);
                    if !chunk_res.local_err.is_empty() {
                        let _ = stderr.write_all(&chunk_res.local_err);
                    }
                }
                Err(lost_count) => {
                    errors += lost_count;
                    match lang {
                        AppLanguage::Chinese => {
                            let _ = writeln!(stderr, "错误 [batch.worker_panic]: 批处理工作线程异常终止（{} 个文件未完成）", lost_count);
                        }
                        AppLanguage::English => {
                            let _ = writeln!(stderr, "Error [batch.worker_panic]: batch worker thread panicked ({} files failed)", lost_count);
                        }
                    }
                }
            }
        }
    }

    let _ = out.flush();
    if !jsonl_mode {
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let rate = if elapsed_ms > 0.0 {
            (total_files as f64) / (elapsed_ms / 1000.0)
        } else {
            0.0
        };
        match lang {
            AppLanguage::Chinese => eprintln!(
                "已扫描 {} 个文件，耗时 {:.2} ms ({:.0} files/s)：已识别 {}，未知 {}，错误 {}",
                total_files, elapsed_ms, rate, identified, unknown, errors
            ),
            AppLanguage::English => eprintln!(
                "Scanned {} files in {:.2} ms ({:.0} files/s): {} identified, {} unknown, {} errors",
                total_files, elapsed_ms, rate, identified, unknown, errors
            ),
        }
    }

    if errors > 0 && identified == 0 && unknown == 0 {
        4
    } else {
        0
    }
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
