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

pub(crate) fn detect_app_language() -> AppLanguage {
    platform_locale::detect_app_language()
}

pub(crate) fn format_identification_cn(id: kindprobe::model::Identification) -> &'static str {
    match id {
        kindprobe::model::Identification::Format => "格式已确认",
        kindprobe::model::Identification::ContainerOnly => "仅识别容器",
        kindprobe::model::Identification::Unknown => "未知格式",
    }
}

pub(crate) fn format_category_cn(cat: &str) -> &'static str {
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

pub(crate) fn format_evidence_explanation_cn(code: &str) -> Option<&'static str> {
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

pub(crate) fn extract_arch_display_cn(
    evidence: &[kindprobe::model::EvidenceItem],
) -> Option<String> {
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

pub(crate) fn extract_arch_display_en(
    evidence: &[kindprobe::model::EvidenceItem],
) -> Option<String> {
    for ev in evidence {
        if ev.code.contains(".arch.") || ev.code == "sig.macho.universal" {
            if let Some(desc) = &ev.description {
                return Some(desc.trim_start_matches("Target Architecture: ").to_string());
            }
        }
    }
    None
}

pub(crate) fn format_verification_cn(v: kindprobe::model::Verification) -> &'static str {
    match v {
        kindprobe::model::Verification::Signature => "特征签名",
        kindprobe::model::Verification::Structure => "内部结构",
        kindprobe::model::Verification::None => "无",
    }
}

pub(crate) fn format_inspection_cn(i: kindprobe::model::Inspection) -> &'static str {
    match i {
        kindprobe::model::Inspection::Complete => "完整",
        kindprobe::model::Inspection::Limited => "受限",
        kindprobe::model::Inspection::Ambiguous => "存在歧义",
    }
}

pub(crate) fn format_extension_status_cn(es: kindprobe::model::ExtensionStatus) -> &'static str {
    match es {
        kindprobe::model::ExtensionStatus::Match => "匹配",
        kindprobe::model::ExtensionStatus::Mismatch => "不匹配",
        kindprobe::model::ExtensionStatus::Missing => "缺失",
        kindprobe::model::ExtensionStatus::Unrecognized => "未识别",
        kindprobe::model::ExtensionStatus::Undetermined => "待定",
    }
}

pub(crate) fn format_warning_explanation_cn<'a>(code: &str, default_msg: &'a str) -> &'a str {
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
