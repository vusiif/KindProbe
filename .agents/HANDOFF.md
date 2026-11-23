# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 511 项大关！全量 25 个规则分卷严密收敛于 77~79 行黄金规范，40 大测试套件 169 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `af30006` GitHub Actions CI 全绿)

---

## 1. [已完成] (Completed)

### 1.1 扩展多垂直分卷规则矩阵（格式覆盖达到 511 项）
全量 25 个规则分卷文件严格收敛在 77~79 行黄金区间（严守 50~80 行铁律）：
- **归档与压缩分卷** ([`src/detectors/rules/generated/archives.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/archives.rs), 77 行)：
  - 新增 SquashFS Linux 只读压缩文件系统格式 (`hsqs`, `.sqsh`, `.squashfs`)。
- **经典芯片音频分卷** ([`src/detectors/rules/generated/audio_chips.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/audio_chips.rs), 78 行)：
  - 新增 NES Sound Format 游戏音频音乐格式 (`NESM\x1a`, `.nsf`)。
- **编译器与字节码分卷** ([`src/detectors/rules/generated/bytecode.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/bytecode.rs), 78 行)：
  - 新增 LLVM Bitcode 现代编译器底层中间表示字节码格式 (`BC\xc0\xde`, `.bc`)。
- **地理与空间航迹分卷** ([`src/detectors/rules/generated/cad_and_gnss.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/cad_and_gnss.rs), 78 行)：
  - 新增 Google Earth Keyhole Markup Language 航线标记格式 (`<kml`, `.kml`)。
- **安全密钥与认证分卷** ([`src/detectors/rules/generated/crypto.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/crypto.rs), 78 行)：
  - 新增 OpenSSH Ed25519 现代公钥格式 (`ssh-ed25519 `, `.pub`)。
- **大数据与数据库分卷** ([`src/detectors/rules/generated/database.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/database.rs), 77 行)：
  - 新增 Apache ORC 大数据列式存储数据库文件格式 (`ORC`, `.orc`)。
- **虚拟磁盘与镜像分卷** ([`src/detectors/rules/generated/disk_images.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/disk_images.rs), 78 行)：
  - 新增 Compressed ISO 高压缩虚拟光盘镜像格式 (`IsZ\x01`, `.isz`)。
- **网络存档与文档分卷** ([`src/detectors/rules/generated/documents.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/documents.rs), 77 行)：
  - 新增 ISO 28500 WARC 万维网网页存档标准格式 (`WARC/1.`, `.warc`)。
- **字体与点阵分卷** ([`src/detectors/rules/generated/fonts.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/fonts.rs), 78 行)：
  - 新增 Adobe BDF 字形点阵分发字体格式 (`STARTFONT `, `.bdf`)。
- **商业游戏引擎资产分卷** ([`src/detectors/rules/generated/game_assets.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/game_assets.rs), 77 行)：
  - 新增 Unreal Engine 虚幻引擎包体与地图资产格式 (`\xc1\x83\x2a\x9e`, `.uasset`, `.umap`)。
- **现代高保真图像分卷** ([`src/detectors/rules/generated/images.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/images.rs), 78 行)：
  - 新增 Quite OK Image Format (QOI) 极速无损光栅图像格式 (`qoif`, `.qoi`)。
- **多声道影音分卷** ([`src/detectors/rules/generated/media.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/media.rs), 78 行)：
  - 新增 Dolby Digital AC-3 杜比环绕声音频流格式 (`\x0b\x77`, `.ac3`)。
- **3D 扫描与点云分卷** ([`src/detectors/rules/generated/models.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/models.rs), 77 行)：
  - 新增 Stanford PLY 多边形 3D 几何模型网格格式 (`ply\n`, `.ply`)。
- **专业音乐模组分卷** ([`src/detectors/rules/generated/multimedia_stream.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/multimedia_stream.rs), 77 行)：
  - 新增 Scream Tracker 3 电子多轨音乐模组格式 (`SCRM`, `.s3m`)。
- **任意像素位图分卷** ([`src/detectors/rules/generated/netpbm.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/netpbm.rs), 78 行)：
  - 新增 Netpbm Portable Arbitrary Map 通用光栅格式 (`P7\n`, `.pam`)。
- **Web 性能与网络分卷** ([`src/detectors/rules/generated/packages.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/packages.rs), 78 行)：
  - 新增 W3C HTTP Archive 抓包与网络时序分析格式 (`{"log":`, `.har`)。
- **富媒体同步播放分卷** ([`src/detectors/rules/generated/playlists.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/playlists.rs), 78 行)：
  - 新增 W3C Synchronized Multimedia Integration Language 播放标记格式 (`<smil`, `.smil`)。
- **机器人仿真分卷** ([`src/detectors/rules/generated/robotics_firmware.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/robotics_firmware.rs), 77 行)：
  - 新增 ROS Unified Robot Description Format 机器人构型 XML (`<robot`, `.urdf`)。
- **游戏 ROM 差分补丁分卷** ([`src/detectors/rules/generated/roms.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/roms.rs), 78 行)：
  - 新增 International Patching System (IPS) 补丁格式 (`PATCH`, `.ips`)。
- **电生理与神经科学分卷** ([`src/detectors/rules/generated/scientific.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/scientific.rs), 78 行)：
  - 新增 Axon Binary File 电生理膜电位记录波形格式 (`ABF `, `.abf`)。
- **经典影视字幕分卷** ([`src/detectors/rules/generated/subtitles.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/subtitles.rs), 78 行)：
  - 新增 MicroDVD 帧基准影视字幕格式 (`{1}{1}`, `.sub`)。
- **金融电子报文分卷** ([`src/detectors/rules/generated/telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/telecom_exchange.rs), 78 行)：
  - 新增 Open Financial Exchange 银行金融交易交换格式 (`OFXHEADER`, `.ofx`, `.qfx`)。
- **GPU 通用纹理分卷** ([`src/detectors/rules/generated/textures.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/textures.rs), 77 行)：
  - 新增 Khronos Basis Universal 超压缩 GPU 纹理格式 (`sB\x01\x00`, `.basis`)。
- **CAD 工程幻灯片分卷** ([`src/detectors/rules/generated/vector_graphics.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/vector_graphics.rs), 77 行)：
  - 新增 Autodesk AutoCAD Slide 矢量工程幻灯片格式 (`AutoCAD Slide\r\n\x1a\x00`, `.sld`)。

### 1.2 全局扩展名索引表跨越 511 项大关 ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))
- 扩展名总数由 485 项成功扩充至 **511 项**；
- 严格遵循单调递增 ASCII 字典排序，通过 `test_registry_is_strictly_sorted_without_duplicates` 自动化断言验证。

### 1.3 集成测试与全量保障
- 新增独立集成测试套件 [`tests/test_vertical_expansion_500.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_vertical_expansion_500.rs)，全面覆盖新增垂直领域格式；
- 全量 40 个测试套件，**169 个测试用例全部通过 (169/169 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Warnings, 0 Errors)。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **静态代码检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告。
- **测试覆盖率**：全量 40 个套件、169 个测试用例全部通过 (169/169 PASS)。
- **规范契约**：全量 25 个规则分卷文件行数全部稳定在 77~79 行黄金区间（严密收敛于 50~80 行）。
- **Git 仓库状态**：已就绪，等待用户指令提交。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 经用户指令后执行 Git 原子提交并推送到 GitHub 远程仓库 (`origin/main`)；
  2. 监控 GitHub Actions 确认三端全绿；
  3. 视需求进一步扩充工业协议、专用硬件、小众操作系统等特色格式。
