# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 485 项大关！全量 25 个规则分卷严密收敛于 78~79 行黄金规范，39 大测试套件 167 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `8e01ab9` GitHub Actions CI 三端构建全绿)

---

## 1. [已完成] (Completed)

### 1.1 扩展多垂直分卷规则矩阵（格式覆盖达到 485 项）
全量 25 个规则分卷文件严格收敛在 78~79 行黄金区间（严守 50~80 行铁律）：
- **矢量图形与动画分卷** ([`src/detectors/rules/generated/vector_graphics.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/vector_graphics.rs), 78 行)：
  - 新增 Computer Graphics Metafile 矢量图元元文件格式 (`BEGMF`, `.cgm`)。
- **加密与安全认证分卷** ([`src/detectors/rules/generated/crypto.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/crypto.rs), 79 行)：
  - 新增 Minisign 现代轻量级签名文件格式 (`untrusted comment: `, `.minisig`)。
- **字体与排版分卷** ([`src/detectors/rules/generated/fonts.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/fonts.rs), 79 行)：
  - 新增 X11 Server Normal Format 点阵位图字体格式 (`\x04\x10\x00\x00`, `.snf`)。
- **胶片与高动态图像分卷** ([`src/detectors/rules/generated/images.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/images.rs), 79 行)：
  - 新增 Kodak Cineon 电影级高精度光栅胶片图像格式 (`\x80\x2a\x5f\xd7`, `.cin`)。
- **便携光栅与复古位图分卷** ([`src/detectors/rules/generated/netpbm.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/netpbm.rs), 79 行)：
  - 新增 Amiga Electronic Arts IFF Interleaved Bitmap 图像格式 (`FORM...ILBM`, `.lbm`, `.ilbm`)。
- **游戏 ROM 与差分补丁分卷** ([`src/detectors/rules/generated/roms.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/roms.rs), 79 行)：
  - 新增 Universal Patch Sync (UPS) 游戏差分补丁格式 (`UPS1`, `.ups`)。
- **科学与结晶学分卷** ([`src/detectors/rules/generated/scientific.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/scientific.rs), 79 行)：
  - 新增 Crystallographic Information File 晶体结构学数据交换格式 (`data_`, `.cif`)。
- **电信与企业交换分卷** ([`src/detectors/rules/generated/telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/telecom_exchange.rs), 79 行)：
  - 新增 UN/EDIFACT 联合国电子数据交换标准格式 (`UNA:`, `.edi`, `.edifact`)。
- **空间航迹与地理分卷** ([`src/detectors/rules/generated/cad_and_gnss.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/cad_and_gnss.rs), 79 行)：
  - 新增 EasyGPS / TopoGrafix Waypoint 航点坐标定位格式 (`<loc version="1.0"`, `.loc`)。
- **媒体播放列表分卷** ([`src/detectors/rules/generated/playlists.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/playlists.rs), 79 行)：
  - 新增 AIMP Player 专业播放列表格式 (`AIMP`, `.aimppl`)。
- **媒体字幕与歌词分卷** ([`src/detectors/rules/generated/subtitles.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/subtitles.rs), 79 行)：
  - 新增 Universal Subtitle Format 通用多流字幕格式 (`<USFSubtitles`, `.usf`)。

### 1.2 全局扩展名索引表突破 485 项大关 ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))
- 扩展名总数由 472 项扩充至 **485 项**；
- 严格遵循单调递增 ASCII 字典排序，通过 `test_registry_is_strictly_sorted_without_duplicates` 自动化断言验证。

### 1.3 集成测试与全量保障
- 在 [`tests/test_codegen_generated_rules.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_codegen_generated_rules.rs)、[`tests/test_playlists_and_telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_playlists_and_telecom_exchange.rs)、[`tests/test_subtitles_and_cad_gnss.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_subtitles_and_cad_gnss.rs) 覆盖全部新增格式；
- 全量 39 个测试套件，**167 个测试用例全部通过 (167/167 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Warnings, 0 Errors)。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **静态代码检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告。
- **测试覆盖率**：全量 39 个套件、167 个测试用例全部通过 (167/167 PASS)。
- **规范契约**：全量 25 个规则分卷文件行数全部稳定在 78~79 行黄金区间（严密收敛于 50~80 行）。
- **Git 仓库状态**：前次提交 `8e01ab9` CI 绿灯通过，本次 485 项扩展改动已就绪。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 经用户指示后执行 Git 原子提交并推送到 GitHub 远程仓库 (`origin/main`)；
  2. 监控 GitHub Actions 确认三端全绿；
  3. 继续向 500 项大关冲刺（如扩充特殊医学波形、工业自动化协议、专用压缩包等）。
