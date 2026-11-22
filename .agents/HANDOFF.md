# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (全平台可执行文件 CPU 架构与子系统深度解析，28 大测试套件 133 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 全平台可执行文件 CPU 目标架构与子系统深度解析
在保持零额外 I/O 与纳秒级就地读取（In-Place Parsing）的前提下，完整实现了 PE、ELF、Mach-O 三大主流可执行文件二进制 CPU 架构识别与子系统探测：
- **Windows PE (`src/detectors/executables/pe_meta.rs`, `pe.rs`)**：
  - COFF `Machine` 字段解析：`x86`, `x64` (AMD64), `arm64` (AArch64), `arm64ec` (仿真兼容), `arm` (32位小端), `arm_thumb2` (Windows RT), `ia64` (Itanium), `riscv32`, `riscv64`, `riscv128`, `powerpc`, `mips`；
  - Optional Header 与地址寻址空间校验：`sig.pe.opt.pe32` (32位寻址), `sig.pe.opt.pe32plus` (64位寻址)；
  - Subsystem 子系统判断：`sig.pe.subsystem.gui` (图形界面), `sig.pe.subsystem.console` (控制台 CUI)；
  - 架构拆分重构：将 PE 解析拆分为 `pe_meta.rs` (59 行) 与 `pe.rs` (63 行)，严格遵循 50~80 行模块化规范。
- **Linux / BSD ELF (`src/detectors/executables/elf.rs`, 90 行)**：
  - 字节序自适应解析 `e_machine` 架构：`x86`, `x86_64`, `arm`, `arm64`, `riscv`, `mips`, `powerpc`, `powerpc64`, `sparc`；
  - `e_type` 文件类型判定：可执行文件 (`ET_EXEC`) vs 动态共享库 (`ET_DYN`)，动态自适应推荐扩展名 `.elf` / `.so`；
  - 地址长度：`sig.elf.class.32bit` 与 `sig.elf.class.64bit`。
- **macOS / iOS Mach-O (`src/detectors/executables/macho.rs`, 82 行)**：
  - `cputype` 架构解析：`x86`, `x86_64`, `arm`, `arm64` (Apple Silicon M系列)；
  - 通用多架构二进制：FAT / Universal Binary (`sig.macho.universal`)。

### 1.2 人类友好终端展示与机器 JSON 双模呈现 (`src/main.rs`)
- 中文模式与英文模式下，当检测到 CPU 架构时，在元数据显著位置输出：
  - 中文：`目标架构: ARM64 (AArch64 / 64位)` / `目标架构: x86-64 (AMD64 / 64位)`
  - 英文：`Target Architecture: ARM64 (AArch64 / 64-bit)`
- 完善证据链多语言释义与描述输出，证据列表直接附带中文说明（例如 `sig.pe.arch.arm64 (目标架构: ARM64 (AArch64 / 64位))`）；
- `--json` 机器可读模式在不破坏既有 Schema Invariant 的前提下，通过 `EvidenceItem.description` 完整输出目标 CPU 架构描述。

### 1.3 专项端到端测试覆盖 (`tests/test_executables_arch.rs`)
- 覆盖 x86、x64、ARM64、ARM64EC、ARM Thumb-2 等 PE 架构；
- 覆盖 ELF ARM64、RISC-V 64-bit、Mach-O Apple Silicon ARM64、x86-64；
- 端到端验证 CLI 中英文输出与 JSON 反序列化断言；
- 累计 28 个测试套件，**133 个测试用例全部通过**。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (133/133 PASS)。
- **Git 仓库状态**：准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认全平台 (Ubuntu, macOS, Windows) 全绿。
