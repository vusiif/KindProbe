# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (离线 Codegen 管线与 Tika 权威规则生成完成)  
> **远端分支**：`origin/main` (当前提交待推送)

---

## 1. [已完成] (Completed)

### 1.1 离线代码生成管线 (`tools/codegen.py`)
- **权威信源直驱**：建立零依赖离线生成工具 `tools/codegen.py`，直接解析 Apache Tika 官方 `tika-mimetypes.xml`（275 KB，1598 个 MIME 类型定义与 1274 个 glob 扩展名规则）。
- **固定偏移与预算约束控制**：严格提取纯固定偏移（fixed offset）且 `max(offset + len) <= 4096` 的二进制魔数规则，天然与 `BudgetReader` 的 4 KiB 初始读取窗口契约保持完全一致。
- **位掩码 (Bitmask) 与截断适配**：在 `src/detectors/rules/types.rs` 与 `src/detectors/rules/engine.rs` 中完整实现并优化带有 `mask` 掩码的切片比对与截断探测。
- **极简分卷模块化**：针对规则库膨胀问题，Codegen 将声明式规则按大类输出至 `src/detectors/rules/generated/`：
  - `media.rs` (56 行)：包含 `flv`, `amr`, `ac3`, `au`, `caf`, `drc`, `mng` 等。
  - `images.rs` (70 行)：包含 `xcf`, `exr`, `fits`, `icns`, `jp2`, `bpg`, `pbm`, `pgm`, `ppm` 等。
  - `documents.rs` (35 行)：包含 `dicom`, `postscript`, `rtf`, `dwg` 等。
  - `archives.rs` (42 行)：包含 `cpio`, `lz`, `arj`, `zoo`, `cab` 等。
  - `mod.rs` (22 行)：导出汇总 `GENERATED_RULES: &[&[SignatureRule]]` 切片。
  所有文件严格控制在 35~70 行区间，实现零运行时堆分配（`&'static [SignatureRule]`）。

### 1.2 扩展名单一真实信源 (SSOT) 索引库
- 自动提取全部新老扩展名映射生成 `src/extension/registry.rs`（收录 161 个标准扩展名），严格按 ASCII 升序排列并去重，提供零哈希分配的 $O(\log N)$ 二分查找。
- 通过单元测试 `test_registry_is_strictly_sorted_without_duplicates` 自动化守护排序单调递增性。

### 1.3 全品类格式能力矩阵 (已拓展至 85+ 种格式)
| 类别 | 已支持的具体格式与深度验证逻辑 |
|---|---|
| **办公文档** | **OOXML 家族** (`docx`, `xlsx`, `pptx`, `docm`, `xlsm`, `pptm`)；**CFB 家族** (`doc`, `xls`, `ppt`)；**ODF 家族** (`odt`, `ods`, `odp`)；`pdf`；`chm`；`dicom`；`postscript`；`rtf`；`dwg` |
| **图像设计** | `png`, `jpeg`, `gif`, `bmp`, `webp`, `psd`, `tiff` (II*/MM*), `ico`, `cur`, `xcf`, `exr`, `fits`, `icns`, `jp2`, `bpg`, `pbm`, `pgm`, `ppm` |
| **音视频媒体** | `mp4`, `mov`, `webm`, `mkv`, `wav`, `mp3`, `flac`, `ogg`, `aac`, `aiff`, `mid`, `flv`, `amr`, `ac3`, `au`, `caf`, `drc`, `mng` |
| **压缩归档** | `zip`, `7z`, `rar`, `tar`, `gzip`, `bzip2`, `xz`, `zstd`, `cpio`, `lz`, `arj`, `zoo`, `cab` |
| **二进制执行** | `elf`, `pe`, `wasm`, `macho`, `class` |
| **字体电子书** | `epub`, `mobi`, `ttf`, `otf`, `woff`, `woff2` |
| **数据库转储** | `sqlite3`, `pcap`, `pcapng`, `parquet`, `avro` |
| **虚拟化镜像** | `qcow2`, `vmdk`, `vhd`, `vhdx`, `iso` |
| **结构化文本** | `json`, `xml`, `html`, `yaml`, `toml`, `csv`, `tsv`, `txt` |
| **软件包分发** | `deb`, `rpm`, `crx`, `apk`, `ipa`, `jar` |

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets` 0 Error（且无多余警告）。
- **集成测试基线**：全量 `cargo test` **114 / 114 全部通过 (PASS)**（新增 `test_codegen_generated_rules` 套件，全绿，耗时 ~1.1 秒）。
- **Git 仓库状态**：准备提交并推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无 (None)。
- **后续优化建议**：
  1. 可进一步通过 Codegen 支持多段复合偏移模式（如带有兄弟分支的多级 match）；
  2. 既有旧专有探测器在必要时逐步向 `SignatureRule` 归一化。
