# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (扩展支持音色库/DuckDB/LZ4/PeaZip/QCOW1/ODG/KMZ/PCX/IPS/BPS，全局扩展名达 370 项，34 大测试套件 153 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 格式矩阵与特征探测纵深扩展
全部分卷与实现文件严格保持在 50~80 行黄金规范区间：
- **专用音色库与合成器** (`audio_chips.rs`, 76 行)：支持 SoundFont 2.0 (`.sf2`/`.sfbk`, RIFF + `sfbk`) 与 DLS (`.dls`, RIFF + `DLS `)；
- **现代分析型数据库** (`database.rs`, 68 行)：支持 DuckDB (`.duckdb`, 偏移 8 `DUCK`)；
- **科学与生物大分子数据** (`scientific.rs`, 69 行)：支持 Protein Data Bank (`.pdb`, `HEADER    `)；
- **无损专业音频** (`media.rs`, 76 行)：支持 True Audio (`.tta`, `TTA1`)；
- **轻量与专用压缩归档** (`packages.rs`, 69 行)：支持 LZ4 (`.lz4`, `\x04\x22\x4d\x18`) 与 PeaZip (`.pea`, `\xea\x06\x00`)；
- **虚拟化与磁盘镜像** (`disk_images.rs`, 65 行)：支持 QEMU Copy-On-Write v1 (`.qcow`, `QFI\xeb`) 与 Apple 稀疏磁盘镜像 (`.sparseimage`, `sprs`)；
- **经典栅格与图元** (`netpbm.rs`, 62 行)：支持 PCX (`.pcx`, `\x0a\x05\x01`) 与 XPM (`.xpm`, `/* XPM */`)；
- **游戏 ROM 补丁** (`roms.rs`, 69 行)：支持 IPS (`.ips`, `PATCH`) 与 BPS (`.bps`, `BPS1`)；
- **ZIP 衍生容器深度探测**：
  - `epub.rs` (69 行)：表驱动优化 ODF 探测，扩展支持 OpenDocument Graphics (`.odg`)；
  - `packages.rs` (65 行)：抽取 `apply_pkg` 并扩展支持 Keyhole Markup Language Zipped (`.kmz`)。

### 1.2 RIFF 分流与容器兜底解耦优化
- `riff.rs`：对非 WAVE/WEBP/AVI 的未知 `form_type` 返回 `RiffMatch::None`，放行给下游专业规则（如 SF2、DLS）；
- `signatures.rs`：在所有专用规则评估完毕后设立 RIFF 容器通用兜底（`sig.riff.header` -> `Identification::ContainerOnly`），兼顾特异性与通用识别。

### 1.3 扩展名索引表扩至 370 项 (`src/extension/registry.rs`)
- 严格按 ASCII 字典单调升序排序，新增 `bps`, `dls`, `duckdb`, `ips`, `kmz`, `lz4`, `odg`, `pcx`, `pdb`, `pea`, `qcow`, `sf2`, `sparseimage`, `tta`；
- 通过 `test_registry_is_strictly_sorted_without_duplicates` 守卫测试。

### 1.4 集成测试与全量验证 (`tests/test_odf_synth_and_lz4.rs`)
- 新增 `test_odf_synth_and_lz4.rs` 专项测试套件，全面覆盖本次扩增的格式；
- 累计 34 个测试套件，**153 个测试用例全部通过 (153/153 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 零警告。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (153/153 PASS)。
- **文件行数规范**：所有修改规则及 ZIP 模块文件严格处于 50~80 行区间。
- **Git 仓库状态**：代码与交接简报更新完毕，等待原子提交与推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) CI 全绿。
