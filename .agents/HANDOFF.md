# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (18 大垂直领域规则分卷定型，全局扩展名索引表扩至 314 项，31 大测试套件 142 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 规则分卷架构扩展至 18 大垂直领域 (`src/detectors/rules/generated/`)
彻底保持模块化，所有 18 个规则分卷严格控制在 35~80 行黄金规范区间：
- **`bytecode.rs` (新分卷 17，49 行)**：现代运行时与底层字节码（`dex` Android Dalvik 字节码全版本, `luac` Lua 预编译字节码, `ar` Unix 静态链接库与归档 `.a`/`.lib`）；
- **`vector_graphics.rs` (新分卷 18，49 行)**：图元与现代图像格式（`wmf` Windows 元文件, `emf` 增强型元文件, `qoi` Quite OK Image, `jxl` JPEG XL 裸流与容器, `flif` 自由无损图像）；
- **`models.rs` (扩充，62 行)**：新增骨骼动作捕捉数据（`bvh` Biovision Hierarchy）；
- **`database.rs` (扩充，62 行)**：新增电信抓包归档（`snoop` RFC 1761 Snoop 数据包）；
- **`mod.rs` (62 行)**：全动态聚合 18 大分卷为 `GENERATED_RULES` 零拷贝静态切片。

### 1.2 扩展名索引表突破 314 项 (`src/extension/registry.rs`)
- 严格按 ASCII 升序排列并维护单元测试单调性断言；
- 扩展名总数由 304 项跃升至 **314 项**，二分查找耗时稳定在 36 ns / op（吞吐量突破 2700 万 QPS）。

### 1.3 专项集成测试与微基准验证 (`tests/test_bytecode_and_vector.rs`)
- 新增 `test_bytecode_and_vector.rs` 专项套件，覆盖 Dalvik 字节码、Lua 预编译、Unix 静态库、WMF/EMF 图元与 QOI/JXL 图像；
- 累计 31 个测试套件，**142 个测试用例全部通过**。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (142/142 PASS)。
- **Git 仓库状态**：准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) 全绿。
