# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (20 大垂直领域规则分卷定型，全局扩展名索引表跃升至 356 项，机器人/固件/游戏资产矩阵就绪，33 大测试套件 149 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 规则分卷架构扩展至 20 大垂直领域 (`src/detectors/rules/generated/`)
全部分卷文件严格保持在 50~80 行黄金规范区间（51~75 行）：
- **`game_assets.rs` (新分卷 19，61 行)**：专业游戏引擎与数据归档（`wad` Doom 游戏包全变种, `pak` Quake 归档包, `pck` Godot 引擎资源包, `unity3d` Unity AssetBundle, `vpk` Valve 资源包, `big` EA 归档, `grp` Build 引擎包）；
- **`robotics_firmware.rs` (新分卷 20，54 行)**：机器人、时序遥测与嵌入式固件（`mcap` Foxglove 机器人流容器, `bag` ROS 2.0 数据包, `pcd` PCL 点云, `dtb`/`dtbo` 扁平化设备树, `uimage` Das U-Boot 系统镜像, `cram` 生物信息比对归档, `bcf` 二进制变异位点文件）；
- **`archives.rs` (解耦重构，61 行)**：将游戏资产解耦剥离至独立分卷，行数从 82 行精准回落至 61 行；
- **`fonts.rs` (扩充，55 行)**：新增紧凑字体格式（`cff` CFF）与经典 Windows 位图字体资源（`fnt`/`fon`）；
- **`vector_graphics.rs` (扩充，54 行)**：新增 Xfig 矢量绘图格式（`fig`）；
- **`mod.rs` (67 行)**：聚合 20 大分卷为 `GENERATED_RULES` 零拷贝静态切片。

### 1.2 扩展名索引表突破 356 项 (`src/extension/registry.rs`)
- 严格按 ASCII 字典升序维护，二分查找耗时稳定在 ~36 ns / op；
- 扩展名总数由 338 项跃升至 **356 项**，通过 `test_registry_is_strictly_sorted_without_duplicates` 守卫测试。

### 1.3 专项集成测试与微基准验证 (`tests/test_games_and_firmware.rs`)
- 新增 `test_games_and_firmware.rs` 专项套件，覆盖游戏引擎资产、机器人遥测、设备树固件、紧凑字体与 Xfig 矢量；
- 累计 33 个测试套件，**149 个测试用例全部通过 (149/149 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 保持绝对 0 告警。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (149/149 PASS)。
- **Git 仓库状态**：准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) 全绿。
