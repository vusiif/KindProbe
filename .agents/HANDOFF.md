# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式扩展至 421 项！新增多媒体播放列表 playlists 与嵌入式/通信/高维数据交换 telecom_exchange 两大垂直分卷，38 大测试套件 165 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 扩展新增两大垂直分卷（共 22 个分卷矩阵）
新增实现文件严格符合 50~80 行黄金规范区间：
- **流式多媒体播放列表与音轨工作单** ([`src/detectors/rules/generated/playlists.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/playlists.rs), 55 行)：
  - 支持 M3U / M3U8 播放列表 (`#EXTM3U`, `.m3u`, `.m3u8`)；
  - 支持 PLS Shoutcast 流媒体播放列表 (`[playlist]`, `.pls`)；
  - 支持 CUE Sheet 高保真音轨索引工作单 (`FILE "`, `.cue`)；
  - 支持 WPL Windows Media 播放列表 (`<?wpl`, `.wpl`)；
  - 支持 XSPF XML 可共享播放列表 (`<playlist version="1"`, `.xspf`)；
  - 支持 ASX 流式重定向播放列表 (`<asx version=`, `.asx`)；
  - 支持 B4S Winamp 播放列表 (`Winamp XML`, `.b4s`)。
- **嵌入式固件、通信与系统交换数据** ([`src/detectors/rules/generated/telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/telecom_exchange.rs), 55 行)：
  - 支持 Apple 二进制属性列表 (`bplist00`, `.bplist`)；
  - 支持 Motorola S-Record 固件全系（S0 头部、S1 16位数据、S3 32位数据，`.srec`, `.s19`, `.s28`, `.s37`）；
  - 支持 vCard 虚拟名片个人信息交换 (`BEGIN:VCARD`, `.vcf`, `.vcard`)；
  - 支持 iCalendar 日历与日程交换 (`BEGIN:VCALENDAR`, `.ics`, `.ical`, `.ifb`)；
  - 支持 LDIF LDAP 目录数据交换 (`version: 1\n`, `.ldif`)；
  - 完善并强化地球观测科学数据 HDF4 格式接受扩展名 (`.hdf`, `.hdf4`, `.h4`)。
- **分卷聚合入口更新** ([`src/detectors/rules/generated/mod.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/mod.rs), 73 行)：注册两大新分卷，全局分卷增至 22 个。

### 1.2 全局扩展名索引表突破 421 项 ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))
- 扩展名总数由 400 项提升至 **421 项**；
- 严格按 ASCII 升序字典序单调排列，通过 `test_registry_is_strictly_sorted_without_duplicates` 硬性断言测试。

### 1.3 集成测试与全量保障 ([`tests/test_playlists_and_telecom_exchange.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_playlists_and_telecom_exchange.rs))
- 新增专项测试套件 `test_playlists_and_telecom_exchange.rs`，覆盖播放列表族系与固件/系统交换数据的内容探测与扩展名双向诊断；
- 累计 38 个测试套件，**165 个测试用例全部通过 (165/165 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 保持 **0 Errors, 0 Warnings**。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **静态代码检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告。
- **测试覆盖率**：全量 38 个套件、165 个测试用例全部通过 (165/165 PASS)。
- **规范契约**：新增与修改的分卷文件行数严密收敛于 55~76 行黄金区间。
- **Git 仓库状态**：改动就绪，等待提交与推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 经用户确认后执行 Git 原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) CI 全绿。
