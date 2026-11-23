# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 591 项大关！新增复古系统虚拟化与生物创意设计两大分卷，全量 29 个规则分卷严密收敛于 60~79 行黄金规范，42 大测试套件 174 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `03cdd22` GitHub Actions CI 全绿通过)

---

## 1. [已完成] (Completed)

### 1.1 扩展架构至 28 个垂直分卷与 591 项格式支持
不仅维持原有 26 个分卷的高信噪比规则，还全新开辟了两个结构严谨的高价值垂直领域规则分卷，并将 `src/detectors/rules/generated/mod.rs` 精简维持在 60 行黄金规范区间：
- **复古系统与虚拟化分卷** ([`src/detectors/rules/generated/retro_and_virtualization.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/retro_and_virtualization.rs), 69 行)：
  1. `adf`: Amiga Disk File 软盘原始镜像 (`DOS\x00`, `.adf`)
  2. `dsk`: Amstrad CPC / Spectrum +3 软盘镜像 (`MV - CPC`, `.dsk`)
  3. `tzx`: ZX Spectrum 磁带精确转储 (`ZXTape!\x1a`, `.tzx`)
  4. `tap`: Commodore 64 原始磁带转储 (`C64-TAPE-RAW`, `.tap`)
  5. `cas`: MSX 计算机磁带转储镜像 (`\x1f\xa6\xde\xba\xcc\x13\x7d\x74`, `.cas`)
  6. `xex`: Atari 8-bit 可执行程序 (`\xff\xff\x00\x00`, `.xex`)
  7. `lnx`: Atari Lynx 掌机卡带 ROM (`LYNX`, `.lnx`)
  8. `hfv`: Apple Macintosh HFS 虚拟软盘 (`\x42\x44`, `.hfv`)
  9. `med`: OctaMED Amiga 音频模组 (`MED\x03`, `.med`)
  10. `rad`: Reality Adlib Tracker FM 合成器音乐 (`RAD by REALITY!!`, `.rad`)
  11. `mus`: Doom 经典游戏音乐格式 (`MUS\x1a`, `.mus`)
  12. `xbe`: 微软第一代 Xbox 游戏可执行文件 (`XBEH`, `.xbe`)
  13. `nsp`: 任天堂 Switch 官方数字安装包 (`PFS0`, `.nsp`)
- **生物化学与创意设计分卷** ([`src/detectors/rules/generated/bio_and_creative_design.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/bio_and_creative_design.rs), 69 行)：
  14. `nii`: NIfTI-1 神经放射三维核磁脑扫描 (offset 344 为 `n+1\x00`, `.nii`)
  15. `ab1`: Applied Biosystems 基因测序毛细管电泳图 (`ABIF`, `.ab1`)
  16. `fcs`: 流式细胞术荧光测量标准数据 (`FCS3.0`, `.fcs`)
  17. `cdx`: ChemDraw 二进制化学分子结构交换文件 (`VjCD0100`, `.cdx`)
  18. `mol2`: Tripos Sybyl 分子化学结构 (`@<TRIPOS>MOLECULE`, `.mol2`)
  19. `ase`: Adobe Swatch Exchange 专业色板交换格式 (`ASEF`, `.ase`)
  20. `pat`: Adobe Photoshop Pattern 图样格式 (`8BPT`, `.pat`)
  21. `cpt`: Corel PHOTO-PAINT 光栅图像格式 (`CPTFILE`, `.cpt`)
  22. `afdesign`: Serif Affinity Designer 矢量创意工程 (`\x00\xff...\x00T3D`, `.afdesign`)
  23. `fh`: Macromedia FreeHand 经典矢量绘图 (`AGGD`, `.fh`)
  24. `art`: AOL ART 压缩图像格式 (`JG\x04\x0e`, `.art`)
  25. `wpg`: WordPerfect Graphics 矢量图格式 (`\xffWPC`, `.wpg`)
  26. `cals`: CALS Raster 军事标准图像 (`srcdocid:`, `.cals`, `.cal`)
- **全局扩展名映射表** ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))：
  - 严格扩充至 **591 项**，单调递增 ASCII 字典排序，0 重复项。
- **全套自动化回归测试套件** ([`tests/test_retro_and_bio_expansion.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_retro_and_bio_expansion.rs))：
  - 新增专用端到端测试，全面覆盖全部 26 个全新格式，并验证扩展名匹配状态 (`ExtensionStatus::Match`)。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 42 个测试套件，174 个测试用例，**100% 通过 (174/174 PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/detectors/rules/generated/` 下全量 29 个分卷文件行数严格分布在 **60~79 行**区间（严守 50~80 行铁律）；
- **远端 CI 状态**：
  - 前序提交 `03cdd22` 在 GitHub Actions（macOS、Ubuntu、Windows 三端矩阵）成功全部通过。

---

## 3. [待办与阻塞] (Blockers & Next Steps)

- **阻塞项**：无阻塞。
- **下一步动作**：
  1. 执行 Git 原子提交并推送至远程仓库（`feat(detectors): expand format coverage to 591 formats across 28 partitions`）；
  2. 观察并确认 GitHub Actions CI 矩阵通过；
  3. 冲刺 600+ 格式里程碑！
