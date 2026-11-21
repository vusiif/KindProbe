# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (P1-B 阶段完成)  
> **远端分支**：`origin/main` (Commit: `b95e298`)

---

## 1. [已完成] (Completed)

### 1.1 架构重大演进：声明式规则引擎 (Declarative Rules Engine)
为根治“每增一种格式新建一个源文件”导致的文件爆炸与样板冗余，建立了 **20% 深层容器 + 80% 声明式表驱动** 的双轨架构：
- **核心契约 (`src/detectors/rules/`)**：
  - `types.rs`：定义支持切片数组的多段魔数结构 `MagicPattern`（原生支持多段复合偏移，如 Debian `!<arch>\n` + `debian-binary`）、三态校验枚举 `ValidationStatus`（`Valid`, `Invalid`, `Truncated`）、规则定义 `SignatureRule`（含优先级 `priority: u8` 与最小头长 `min_header_len`）。
  - `engine.rs`：通用评估器 `evaluate_rules`，自动按优先级排序匹配、自动触发三态校验、自动推断截断状态（`Truncated`）并构造标准 `DetectionReport`。
  - `registry.rs`：单一规则注册表 `DECLARATIVE_RULES`，单个源文件直接声明驱动 10 种工业级格式。
- **结构化文本网关系统 (`src/detectors/text/`)**：
  - `gateway.rs`：统一前置网关，执行 UTF-8 BOM 自动剥离、空字节 `\0` 强制拦截、UTF-8 编码校验及控制字符比率过滤（>1% 判定为二进制拦截），彻底杜绝二进制文件误入文本探测。
  - `json.rs`, `xml_html.rs`, `config.rs`, `csv.rs`, `plain.rs`：针对 JSON、XML、HTML、YAML、TOML、CSV、TSV 实施特征密度与列一致性校验，防范纯文本误报。

### 1.2 全品类格式能力矩阵 (已达 55+ 种格式)
| 类别 | 已支持的具体格式与深度验证逻辑 |
|---|---|
| **办公文档** | **OOXML 家族** (`docx`, `xlsx`, `pptx`, `docm`, `xlsm`, `pptm`) OPC XML 关系正规化解析；**CFB 家族** (`doc`, `xls`, `ppt`) OLE2 复合扇区链与 FIB/BOF 校验；**ODF 家族** (`odt`, `ods`, `odp`) ZIP 容器 mimetype 验证；`pdf` (%PDF- 头部验证) |
| **图像设计** | `png` (IHDR 结构校验), `jpeg` (SOI+APP/DQT/SOF 段解析), `gif` (87a/89a+逻辑屏幕尺寸), `bmp`, `webp` (RIFF+VP8), **`psd`** (8BPS+版本 1), **`tiff`** (II*/MM* 双字节序+IFD 偏移), **`ico`** / **`cur`** (图标与光标目录项) |
| **音视频媒体** | `mp4` / `mov` (ISO-BMFF ftyp/moov 解析), `webm` / `mkv` (EBML 头部+DocType 解析), `wav` (RIFF+WAVE+fmt), `mp3` (ID3v2 synchsafe+MPEG frame sync), **`flac`** (fLaC+STREAMINFO 块校验), **`ogg`** (OggS 容器+逻辑流头校验) |
| **压缩归档** | `zip` (EOCD 目录搜索), `7z` (魔数+StartHeader CRC32 校验), `rar` (RAR4 主头+RAR5 扩展签名), `tar` (POSIX ustar 魔数+八进制 Checksum 累加校验), `gzip` (DEFLATE 方法+标志位), `bzip2` (BZh+Pi 常数块), `xz` (Stream Flags CRC32 校验), `zstd` (魔数校验) |
| **二进制执行** | `elf` (32/64 位体系+字节序+e_type), `pe` (DOS MZ+e_lfanew+PE 签名+EXE/DLL 消歧), `wasm` (\\0asm+版本 1), `macho` (单架构与通用 FAT 二进制), `class` (Java 字节码 0xCAFEBABE) |
| **字体电子书** | `epub` (ZIP mimetype+container.xml), `mobi` (PalmDOC+BOOKMOBI), `ttf` / `otf` (表目录结构校验), `woff` / `woff2` (魔数+表目录结构校验) |
| **数据库转储** | `sqlite3` (SQLite format 3\\0+Page Size 2 的幂次+WAL 读写版本号), `pcap` (4 种微秒/纳秒魔数+2.4 版本), `pcapng` (SHB+BOM 校验), `parquet` (PAR1), `avro` (Obj\\x01) |
| **虚拟化镜像** | `qcow2` (QFI\\xFB+版本 2/3+cluster_bits 9..21), `vmdk` (Sparse KDMV+Flat 描述符), `vhd` / `vhdx` (conectix / vhdxfile), `iso` (第 16 扇区 0x8000 PVD 卷描述符流探针) |
| **结构化文本** | `json`, `xml`, `html`, `yaml`, `toml`, `csv`, `tsv`, `txt` (UTF-8 纯文本兜底) |
| **软件包分发** | **`deb`** (多段魔数联合校验 `!<arch>\\n` + `debian-binary`), **`rpm`** (魔数+主版本号), **`crx`** (Cr24+版本校验) |

### 1.3 质量与规范指标
- **代码瘦身约束**：彻底打破原有单文件 900+ 行局面，子模块（`zip_ooxml/`, `cfb/`, `images/`, `archives/`, `media/`, `executables/`, `fonts/`, `ebooks/`, `databases/`, `disks/`, `rules/`, `text/`）的每个源文件均严格维持在 **50~80 行**。
- **中英文界限隔离**：代码、注释、Commit Message 纯英文；CLI 本地化及交付沟通纯中文。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功，无任何 Error。
- **代码质量与静态检查**：`cargo clippy --all-targets` **0 Error**。
- **集成测试基线**：全量 `cargo test` **104 / 104 全部通过 (PASS)**（19 个测试套件，耗时 ~1.2 秒）：
  - `src/main.rs` 单元测试: 2 passed
  - `test_api_consistency`: 1 passed
  - `test_archives_extended`: 6 passed
  - `test_budget_adversarial`: 8 passed
  - `test_cfb_doc`: 2 passed
  - `test_cfb_extended`: 7 passed
  - `test_cli_contract`: 12 passed
  - `test_codex_acceptance_regression`: 9 passed
  - `test_compressions_extended`: 5 passed
  - `test_databases_extended`: 4 passed
  - `test_declarative_rules_extended`: 4 passed
  - `test_disks_extended`: 5 passed
  - `test_executables_extended`: 5 passed
  - `test_extension`: 2 passed
  - `test_fonts_and_ebooks`: 6 passed
  - `test_images_extended`: 6 passed
  - `test_media_extended`: 7 passed
  - `test_ooxml_extended`: 4 passed
  - `test_text_extended`: 6 passed
  - `test_zip_ooxml`: 3 passed
- **Git 仓库状态**：
  - 分支：`main`
  - 远端追踪：与 `origin/main` 保持一致，工作区无未跟踪/未提交的脏改动。
  - 最新提交：`b95e298` (*feat(rules): add declarative detectors for packages (deb, rpm, crx) and media (flac, ogg, psd, tiff, ico, cur)*)。

---

## 3. [待办与建议] (Blockers & Next Steps)

### 3.1 阻碍 (Blockers)
- **无 (None)**。当前项目所有功能完整可用，契约测试完备。

### 3.2 建议后续工作 (Recommended Next Actions)
1. **老旧散落魔数归拢 (Refactoring via Rules Engine)**：
   将既有散落在各独立小文件中的极简魔数检测（如 `parquet_avro.rs`, `wasm.rs`, 独立 `bmp.rs`）逐步迁移到 `DECLARATIVE_RULES` 中，进一步精简目录树。
2. **基准性能测试 (Criterion Benchmarks)**：
   在 `benches/bench_detection.rs` 中引入微秒级基准测试，监控高并发文件探测性能与冷启动耗时。
3. **CI/CD 自动化流水线**：
   在 `.github/workflows/ci.yml` 中添加跨平台构建测试（Windows, Ubuntu, macOS），锁定版本质量门禁。
