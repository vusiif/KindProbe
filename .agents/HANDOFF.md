# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (16 大垂直领域规则分卷定型，全局扩展名索引表扩至 304 项，30 大测试套件 139 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 规则分卷架构扩展至 16 大垂直领域 (`src/detectors/rules/generated/`)
彻底保持模块化，所有 16 个规则分卷严格控制在 35~80 行黄金规范区间：
- **`textures.rs` (新分卷 15，41 行)**：游戏材质与图形纹理（`dds` DirectDraw Surface, `ktx` / `ktx2` Khronos 纹理, `astc` ARM 可自适应纹理, `pvr` PowerVR 3.0）；
- **`multimedia_stream.rs` (新分卷 16，49 行)**：专业流媒体与广电音视频（`asf` / `wmv` / `wma` 微软高级流格式 GUID, `rm` / `rmvb` / `ra` RealMedia, `swf` Flash 影片 FWS/CWS/ZWS, `dts` / `cda` DTS 环绕声）；
- **`database.rs` (扩充，55 行)**：扩充微软 Access 数据库族谱（`mdb` Jet DB 与 `accdb` ACE DB 专用签名）；
- **`archives.rs` (扩充，80 行)**：扩充经典游戏归档资源（`wad` DOOM IWAD/PWAD, `pak` Quake 资源包）；
- **`documents.rs` (扩充，77 行)**：扩充微软经典文档体系（`hlp` WinHelp 帮助文件, `lit` Microsoft Reader 电子书）；
- **`mod.rs` (56 行)**：全动态聚合 16 大分卷为 `GENERATED_RULES` 零拷贝切片。

### 1.2 字体探测器歧义消除与健壮性加固 (`src/detectors/fonts/ttf_otf.rs`, 69 行)
- **精准消除 MDB/ACCDB 与 TTF 冲突**：
  针对 Access 数据库头部 `\x00\x01\x00\x00` 与 TrueType 字体 sfnt 魔数前 4 字节相同的边界情况，引入快速消歧分支，并对 `num_tables` 施加严格结构约束，杜绝误判。

### 1.3 扩展名索引表跃升至 304 项 (`src/extension/registry.rs`)
- 严格按 ASCII 升序排列并维护单元测试单调性断言；
- 扩展名总数由 284 项跃升至 **304 项**，二分查找耗时稳定在 36 ns / op。

### 1.4 专项集成测试与微基准验证 (`tests/test_textures_and_streams.rs`)
- 新增 `test_textures_and_streams.rs` 专项套件，覆盖纹理贴图、多媒体流、数据库与经典游戏归档；
- 累计 30 个测试套件，**139 个测试用例全部通过**。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (139/139 PASS)。
- **Git 仓库状态**：准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) 全绿。
