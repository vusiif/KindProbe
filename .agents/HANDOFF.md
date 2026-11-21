# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (P1-B 第十三批完成)  
> **远端分支**：`origin/main` (当前提交待推送)

---

## 1. [已完成] (Completed)

### 1.1 ZIP 容器深度包格式衍生 (`src/detectors/zip_ooxml/packages.rs`)
在 ZIP 核心目录分析基础之上，增加了高价值分发包识别（单文件仅 ~75 行）：
- **`apk` (Android 应用程序包)**：深度校验 ZIP 内部 `AndroidManifest.xml` 与 `classes.dex` 特征。
- **`ipa` (iOS 应用程序包)**：校验 ZIP 内部 `Payload/*.app/Info.plist` 应用清单特征。
- **`jar` (Java 归档包)**：校验 ZIP 内部 `META-INF/MANIFEST.MF` 清单特征。

### 1.2 声明式音频与帮助文档扩展 (`src/detectors/rules/registry.rs`)
通过 `DECLARATIVE_RULES` 声明表直接追加支持：
- **`aac` (高级音频编码)**：ADTS 帧同步魔数（`0xFF 0xF1` / `0xFF 0xF9`）。
- **`aiff` (音频交换格式)**：多段魔数联合校验（`FORM` 容器头 + `AIFF` / `AIFC` 格式块）。
- **`mid` (MIDI 音频)**：`MThd` 4 字节魔数 + 头块长度 6 字节结构校验。
- **`chm` (已编译 HTML 帮助)**：`ITSF` 4 字节魔数 + 格式版本号结构校验。

### 1.3 跨平台自动化 CI/CD 流水线 (`.github/workflows/ci.yml`)
- 新增 GitHub Actions CI 配置文件，矩阵覆盖 `ubuntu-latest`, `windows-latest`, `macos-latest`。
- 自动化运行 `cargo check`, `cargo test`, `cargo clippy`。

### 1.4 全品类格式能力矩阵 (已突破 60+ 种格式)
| 类别 | 已支持的具体格式与深度验证逻辑 |
|---|---|
| **办公文档** | **OOXML 家族** (`docx`, `xlsx`, `pptx`, `docm`, `xlsm`, `pptm`)；**CFB 家族** (`doc`, `xls`, `ppt`)；**ODF 家族** (`odt`, `ods`, `odp`)；`pdf`；**`chm`** (ITSF+版本) |
| **图像设计** | `png`, `jpeg`, `gif`, `bmp`, `webp`, `psd`, `tiff` (II*/MM*), `ico`, `cur` |
| **音视频媒体** | `mp4`, `mov`, `webm`, `mkv`, `wav`, `mp3`, `flac`, `ogg`, **`aac`** (ADTS), **`aiff`** (FORM+AIFF/AIFC), **`mid`** (MThd+长度 6) |
| **压缩归档** | `zip`, `7z`, `rar`, `tar`, `gzip`, `bzip2`, `xz`, `zstd` |
| **二进制执行** | `elf`, `pe`, `wasm`, `macho`, `class` |
| **字体电子书** | `epub`, `mobi`, `ttf`, `otf`, `woff`, `woff2` |
| **数据库转储** | `sqlite3`, `pcap`, `pcapng`, `parquet`, `avro` |
| **虚拟化镜像** | `qcow2`, `vmdk`, `vhd`, `vhdx`, `iso` |
| **结构化文本** | `json`, `xml`, `html`, `yaml`, `toml`, `csv`, `tsv`, `txt` |
| **软件包分发** | `deb`, `rpm`, `crx`, **`apk`** (ZIP+Manifest/Dex), **`ipa`** (ZIP+Payload), **`jar`** (ZIP+Manifest) |

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets` **0 Error**。
- **集成测试基线**：全量 `cargo test` **109 / 109 全部通过 (PASS)**（20 个测试套件，耗时 ~1.1 秒）：
  - 新增 `test_packages_and_audio_extended`: 5 passed
- **Git 仓库状态**：准备提交并推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无 (None)。
- **后续优化建议**：
  1. 引入 Criterion 基准测试评估各格式微秒级开销；
  2. 既有旧模块逐步迁移入声明式表。
