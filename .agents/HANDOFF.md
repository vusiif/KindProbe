# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 536 项大关！全量 25 个规则分卷严密收敛于 79~80 行黄金规范，40 大测试套件 170 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `306baf0` GitHub Actions CI 全绿通过)

---

## 1. [已完成] (Completed)

### 1.1 扩展多垂直分卷规则矩阵（格式覆盖达到 536 项）
全量 25 个规则分卷文件严格收敛在 79~80 行黄金区间（严守 50~80 行铁律，`mod.rs` 保持 79 行）：
- **归档与备份分卷** ([`src/detectors/rules/generated/archives.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/archives.rs), 80 行)：
  - 新增 Disk ARchive (DAR) 专业系统级备份归档格式 (`\x00\x00\x00\x7b`, `.dar`)。
- **掌机音频芯片分卷** ([`src/detectors/rules/generated/audio_chips.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/audio_chips.rs), 79 行)：
  - 新增 Game Boy Sound System 游戏音乐转储格式 (`GBS\x01`, `.gbs`)。
- **模块化字节码分卷** ([`src/detectors/rules/generated/bytecode.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/bytecode.rs), 79 行)：
  - 新增 Java 9+ JMOD 模块归档字节码格式 (`JM\x01\x00`, `.jmod`)。
- **地理信息交换分卷** ([`src/detectors/rules/generated/cad_and_gnss.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/cad_and_gnss.rs), 79 行)：
  - 新增 MapInfo Interchange Format 矢量地理数据格式 (`Version `, `.mif`, `.mid`)。
- **证书吊销安全分卷** ([`src/detectors/rules/generated/crypto.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/crypto.rs), 79 行)：
  - 新增 X.509 Certificate Revocation List (CRL) 证书吊销列表 (`-----BEGIN X509 CRL-----`, `.crl`)。
- **高性能 LSM 树数据库分卷** ([`src/detectors/rules/generated/database.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/database.rs), 80 行)：
  - 新增 LevelDB / RocksDB SSTable 列存不可变数据表格式 (`SST1`, `.sst`)。
- **便携光盘镜像分卷** ([`src/detectors/rules/generated/disk_images.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/disk_images.rs), 79 行)：
  - 新增 Compressed ISO (CSO) 虚拟光盘镜像格式 (`CISO`, `.cso`)。
- **经典帮助文档分卷** ([`src/detectors/rules/generated/documents.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/documents.rs), 80 行)：
  - 新增 Microsoft WinHelp 经典帮助文档格式 (`?_\x00\x00`, `.hlp`)。
- **X11 编译位图字体分卷** ([`src/detectors/rules/generated/fonts.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/fonts.rs), 79 行)：
  - 新增 X11 Portable Compiled Font 点阵字形格式 (`\x01fcp`, `.pcf`)。
- **游戏室内三维地图分卷** ([`src/detectors/rules/generated/game_assets.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/game_assets.rs), 80 行)：
  - 新增 id Software Quake / GoldSrc BSP 空间分割地图格式 (`IBSP`, `.bsp`)。
- **高压缩光栅图像分卷** ([`src/detectors/rules/generated/images.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/images.rs), 79 行)：
  - 新增 Better Portable Graphics (BPG) 图像格式 (`BPG\xfb`, `.bpg`)。
- **无损音频编码分卷** ([`src/detectors/rules/generated/media.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/media.rs), 79 行)：
  - 新增 True Audio (TTA) 无损音频编解码格式 (`TTA1`, `.tta`)。
- **3D 动画与网格分卷** ([`src/detectors/rules/generated/models.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/models.rs), 80 行)：
  - 新增 Autodesk 3D Studio 场景几何模型格式 (`\x4d\x4d`, `.3ds`)。
- **扩展音乐模组分卷** ([`src/detectors/rules/generated/multimedia_stream.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/multimedia_stream.rs), 80 行)：
  - 新增 FastTracker 2 Extended Module 多轨音频模组格式 (`Extended Module:`, `.xm`)。
- **X11 单色位图分卷** ([`src/detectors/rules/generated/netpbm.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/netpbm.rs), 79 行)：
  - 新增 X11 X BitMap C 语言光栅位图格式 (`#define `, `.xbm`)。
- **清单与包元数据分卷** ([`src/detectors/rules/generated/packages.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/packages.rs), 79 行)：
  - 新增 Java / Assembly Manifest 标准清单文件 (`Manifest-Version: 1.0`, `.manifest`, `.mf`)。
- **流媒体播放列表分卷** ([`src/detectors/rules/generated/playlists.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/playlists.rs), 79 行)：
  - 新增 RealAudio Metafile 流媒体播放清单格式 (`rtsp://`, `.ram`)。
- **机器人与机械制造分卷** ([`src/detectors/rules/generated/robotics_firmware.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/robotics_firmware.rs), 80 行)：
  - 新增 Gazebo Simulation Description Format 机器人仿真物理世界描述 (`<sdf `, `.sdf`)。
- **掌机游戏 ROM 分卷** ([`src/detectors/rules/generated/roms.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/roms.rs), 79 行)：
  - 新增 Nintendo Game Boy Advance (GBA) 掌机 ROM 格式 (`\x24\xff\xae\x51`, `.gba`)。
- **地球物理地震数据分卷** ([`src/detectors/rules/generated/scientific.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/scientific.rs), 79 行)：
  - 新增 SEG-Y 勘探地震测线标准数据格式 (`C 1 `, `.sgy`, `.segy`)。
- **欧洲广播级字幕分卷** ([`src/detectors/rules/generated/subtitles.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/subtitles.rs), 79 行)：
  - 新增 EBU STL 电视广播字幕流格式 (`850`, `.stl`)。
- **Web 聚合归档分卷** ([`src/detectors/rules/generated/telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/telecom_exchange.rs), 79 行)：
  - 新增 MIME Encapsulation HTML 复合网页存档格式 (`From: <Saved by`, `.mhtml`, `.mht`)。
- **Crunch 超压缩 GPU 纹理分卷** ([`src/detectors/rules/generated/textures.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/textures.rs), 80 行)：
  - 新增 Crunch 开源高级 GPU 压缩纹理格式 (`Cr`, `.crn`)。
- **航天机械 CAD 交换分卷** ([`src/detectors/rules/generated/vector_graphics.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/vector_graphics.rs), 80 行)：
  - 新增 Initial Graphics Exchange Specification (IGES) 航天机械 CAD 数据交换规范 (`S      1`, `.iges`, `.igs`)。

### 1.2 全局扩展名索引表跨越 536 项大关 ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))
- 扩展名总数由 511 项成功扩充至 **536 项**；
- 严格遵循单调递增 ASCII 字典排序，通过 `test_registry_is_strictly_sorted_without_duplicates` 自动化断言验证。

### 1.3 集成测试与全量保障
- 在 [`tests/test_vertical_expansion_500.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_vertical_expansion_500.rs) 中全面覆盖了本批次 24 个新增格式；
- 全量 40 个测试套件，**170 个测试用例全部通过 (170/170 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Warnings, 0 Errors)。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **静态代码检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告。
- **测试覆盖率**：全量 40 个套件、170 个测试用例全部通过 (170/170 PASS)。
- **规范契约**：全量 25 个规则分卷文件行数全部稳定在 79~80 行黄金区间（严密收敛于 50~80 行）。
- **Git 仓库状态**：前次提交 `306baf0` CI 绿灯通过，本次 536 项扩展改动已就绪。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 经用户指令后执行 Git 原子提交并推送到 GitHub 远程仓库 (`origin/main`)；
  2. 监控 GitHub Actions 确认三端全绿；
  3. 持续向 550+ / 600+ 项格式大关冲刺。
