# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (跨平台 CI 兼容性加固 & 零 Clippy 警告达成)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 跨平台 CI 兼容性修复 (Windows & macOS)
- **macOS APFS 非 UTF-8 文件名防御**：
  在 `tests/test_cli_contract.rs` 中，适配 macOS APFS 严格 UTF-8 文件系统的内核约束（创建非 UTF-8 文件名时返回 `EILSEQ` 92）。如果系统允许则执行全量探测测试；若系统拒绝则验证 CLI 对非 UTF-8 路径参数的防御性降级行为（退出码 4，输出 `io.open_failed`，严禁 panic）。
- **Windows CRLF 换行符隔离与保护**：
  在根目录新增 `.gitattributes`，显式将 `fixtures/**` 标记为 `binary`，杜绝 Git 在 Windows 检出时自动将 PBM/PGM 等文本型测试固件换行符污染为 `\r\n`，修复 `test_real_fixtures_file_handle_matrix` 跨平台断言失败。
- **全平台 Clippy 零警告达成**：
  重构 `src/budget.rs`（抽取 `WindowState` 简化复杂类型）、`src/detectors/zip_ooxml/cd_check.rs` 与 `src/model.rs`（消除嵌套 if）、`src/main.rs`（优化 pattern 匹配与 Windows FFI 命名规避），确保 `cargo clippy --all-targets -- -D warnings` 零告警通过。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零告警通过 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (100% PASS)。
- **Git 仓库状态**：已暂存，准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 提交并推送至 `origin/main`，监控 GitHub Actions 确保 `ubuntu-latest`, `windows-latest`, `macos-latest` 三大平台全绿。
  2. 依据用户指示，继续扩充更多格式类别（路线 B）。
