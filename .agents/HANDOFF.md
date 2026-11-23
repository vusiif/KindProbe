# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式扩展突破 450 项大关！扩充多媒体、固件交换、科学地理与字幕分卷，39 大测试套件 167 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 扩展规则垂直分卷与 450 项格式覆盖
实现文件严格符合 50~80 行黄金规范区间：
- **现代多媒体视频字幕与歌词时标分卷** ([`src/detectors/rules/generated/subtitles.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/subtitles.rs), 69 行)：
  - 支持 WebVTT 现代网页视频字幕与时标轨迹 (`WEBVTT`, `.vtt`)；
  - 支持 Advanced SubStation Alpha 高级特效字幕 (`[Script Info]`, `.ass`, `.ssa`)；
  - 支持 SubRip 经典跨平台视频字幕 (`1\r\n00:`, `1\n00:`, `.srt`)；
  - 支持 SAMI 媒体可访问性字幕 (`<SAMI>`, `.smi`, `.sami`)；
  - 支持 LRC 同步动态歌词文件 (`[ti:`, `[ar:`, `.lrc`)；
  - 支持 YouTube 专用字幕格式 (`0:00:0`, `.sbv`)；
  - 支持 W3C Timed Text 标记语言与 DFXP 交换格式 (`<tt `, `.ttml`, `.dfxp`)。
- **3D 打印制造、导航定位与开源地理信息分卷** ([`src/detectors/rules/generated/cad_and_gnss.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/cad_and_gnss.rs), 69 行)：
  - 支持 STL 3D 打印标准网格面片 (`solid `, `.stl`)；
  - 支持 VRML 3D 虚拟现实空间模型 (`#VRML V`, `.wrl`, `.vrml`)；
  - 支持 WebAssembly 文本汇编模块 (`(module`, `.wat`)；
  - 支持 NMEA 0183 航海与 GNSS 卫星定位语句 (`$GP`, `.nmea`, `.gps`)；
  - 支持 BibTeX 学术文献与引文数据库 (`@article{`, `@book{`, `.bib`)；
  - 支持 OpenStreetMap 开放地理 XML 数据集 (`<osm version=`, `.osm`)；
  - 支持 MapInfo 矢量地理表格数据 (`!table\n!version `, `.tab`)；
  - 支持 TopoJSON 拓扑地理信息编码 (`{"type":"Topology"`, `.topojson`)；
  - 完善 ESRI Shapefile 索引文件支持 (`.shx` 映射至 `shp`)。
- **多媒体播放列表分卷** ([`src/detectors/rules/generated/playlists.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/playlists.rs), 69 行)：
  - 支持 M3U / M3U8, PLS, CUE, WPL, XSPF, ASX, B4S；
  - 新增 JSPF JSON 播放列表格式 (`{"playlist":`, `.jspf`)；
  - 扩充 Windows Media 流重定向扩展名 (`.wax`, `.wvx`)。
- **通信固件与数据交换分卷** ([`src/detectors/rules/generated/telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/telecom_exchange.rs), 69 行)：
  - 支持 Apple Binary Property List (`bplist00`, `.bplist`)；
  - 支持 Motorola S-Record 固件镜像 (`S0`, `S1`, `S3`, `.srec`, `.s19`, `.s28`, `.s37`)；
  - 支持 vCard 虚拟名片 (`BEGIN:VCARD`, `.vcf`, `.vcard`)；
  - 支持 iCalendar 日程交换 (`BEGIN:VCALENDAR`, `.ics`, `.ical`, `.ifb`)；
  - 支持 LDAP 数据交换格式 (`version: 1\n`, `.ldif`)；
  - 新增 Intel HEX 嵌入式固件格式 (`:10`, `.ihex`, `.ihx`, `.hex`)；
  - 新增 Quicken Interchange Format 财务交易交换格式 (`!Type:`, `.qif`)。
- **分卷聚合入口更新** ([`src/detectors/rules/generated/mod.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/mod.rs), 79 行)：聚合 24 个垂直分卷。

### 1.2 全局扩展名索引表突破 450 项大关 ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))
- 扩展名总数正式由 421 项 -> 438 项 -> **450 项**；
- 严格保持单调递增 ASCII 排序，通过 `test_registry_is_strictly_sorted_without_duplicates` 自动化断言验证。

### 1.3 集成测试与全量保障
- 扩充测试套件 [`tests/test_subtitles_and_cad_gnss.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_subtitles_and_cad_gnss.rs) 与 [`tests/test_playlists_and_telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_playlists_and_telecom_exchange.rs)；
- 全量 39 个测试套件，**167 个测试用例全部通过 (167/167 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Warnings, 0 Errors)。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **静态代码检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告。
- **测试覆盖率**：全量 39 个套件、167 个测试用例全部通过 (167/167 PASS)。
- **规范契约**：所有实现文件行数均稳定在 69~79 行黄金区间（严格处于 50~80 行）。
- **Git 仓库状态**：已就绪，等待提交与推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 经用户指令后执行 Git 原子提交并推送到 GitHub 远程仓库 (`origin/main`)；
  2. 监控 GitHub Actions 确认三端全绿。
