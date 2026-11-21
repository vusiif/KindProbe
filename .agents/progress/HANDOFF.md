# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (后缀架构治理完成)  
> **远端分支**：`origin/main` (当前提交待推送)

---

## 1. [已完成] (Completed)

### 1.1 后缀独立诊断系统重构 (`src/extension/`)
彻底治理了“后缀数量爆炸”与“两处重复维护”的架构隐患：
- **拆分与瘦身**：将原 177 行扁平单文件 `src/extension.rs` 重构为模块化子系统：
  - `src/extension/mod.rs` (~50 行)：实现轻量核心诊断逻辑与 Fast-Path 机制。
  - `src/extension/registry.rs` (~160 行)：维护严格按 ASCII 升序排列的扩展名二分索引表 `SORTED_EXTENSION_REGISTRY`，提供 $O(\log N)$ 的 `find_registered_format` 检索函数。
- **Fast-Path 性能优化**：优先与当前报告中的 `report.accepted_extensions`（通常仅 1~3 个元素）进行本地匹配，命中即返回 `Match`，彻底绕过全局大表检索。
- **二分查找防退化**：发生后缀不一致时，采用原生 `binary_search_by` 执行 $O(\log N)$ 检索，彻底杜绝线性遍历开销。
- **防乱序自校验测试**：新增内置单元测试 `test_registry_is_strictly_sorted_without_duplicates`，在编译构建期自动化保障表内元素的单调严格递增，杜绝未来手误破坏二分特性。

### 1.2 全品类格式能力矩阵 (已突破 60+ 种格式)
| 类别 | 已支持的具体格式与深度验证逻辑 |
|---|---|
| **办公文档** | **OOXML 家族** (`docx`, `xlsx`, `pptx`, `docm`, `xlsm`, `pptm`)；**CFB 家族** (`doc`, `xls`, `ppt`)；**ODF 家族** (`odt`, `ods`, `odp`)；`pdf`；`chm` |
| **图像设计** | `png`, `jpeg`, `gif`, `bmp`, `webp`, `psd`, `tiff` (II*/MM*), `ico`, `cur` |
| **音视频媒体** | `mp4`, `mov`, `webm`, `mkv`, `wav`, `mp3`, `flac`, `ogg`, `aac`, `aiff`, `mid` |
| **压缩归档** | `zip`, `7z`, `rar`, `tar`, `gzip`, `bzip2`, `xz`, `zstd` |
| **二进制执行** | `elf`, `pe`, `wasm`, `macho`, `class` |
| **字体电子书** | `epub`, `mobi`, `ttf`, `otf`, `woff`, `woff2` |
| **数据库转储** | `sqlite3`, `pcap`, `pcapng`, `parquet`, `avro` |
| **虚拟化镜像** | `qcow2`, `vmdk`, `vhd`, `vhdx`, `iso` |
| **结构化文本** | `json`, `xml`, `html`, `yaml`, `toml`, `csv`, `tsv`, `txt` |
| **软件包分发** | `deb`, `rpm`, `crx`, `apk`, `ipa`, `jar` |

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets` **0 Error**。
- **集成测试基线**：全量 `cargo test` **110 / 110 全部通过 (PASS)**（覆盖 20 个集成测试套件 + 扩展名排序自检单元测试，耗时 ~1.1 秒）。
- **Git 仓库状态**：准备提交并推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无 (None)。
- **后续优化建议**：
  1. 引入 Criterion 基准测试评估各格式微秒级开销与二分查找时延；
  2. 既有旧模块逐步迁移入声明式表。
