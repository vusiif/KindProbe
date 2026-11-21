# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **Windows 原生集成测试跨平台修复**：
  - `tests/test_cli_contract.rs`：使用 `#[cfg(unix)]` 守卫 Unix 专有的 `std::os::unix::ffi::OsStrExt` 测试；空环境变量断言适配跨平台（POSIX 回退 English，Windows 回退原生系统语言）。
- **Windows 目录只读打开拒绝兼容**：
  - `src/lib.rs`：在 Windows 下以只读打开目录会返回 `PermissionDenied`，增加 `path.metadata()` 检测非普通文件并精准映射为 `DetectError::NotRegularFile`（错误码 `io.not_regular_file`）。
- **Windows 语言优先级对齐**：
  - `src/main.rs`：在 Windows 平台增加显式 POSIX 环境变量（`LC_ALL` -> `LC_MESSAGES` -> `LANG`）优先判定逻辑；未设置环境变量时回退至 Win32 原生 `GetUserDefaultUILanguage()`。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - Windows 原生环境下 `cargo test` **39/39 全部通过 (PASS)**：
    - `src/main.rs` (unittests): 2/2 PASS
    - `test_api_consistency`: 1/1 PASS
    - `test_budget_adversarial`: 8/8 PASS
    - `test_cfb_doc`: 2/2 PASS
    - `test_cli_contract`: 12/12 PASS
    - `test_codex_acceptance_regression`: 9/9 PASS
    - `test_extension`: 2/2 PASS
    - `test_zip_ooxml`: 3/3 PASS
  - `cargo clippy --all-targets` 检查 0 错误通过。
  - `cargo build` 二进制正常生成，`kindprobe.exe` 命令行与 JSON v1 输出均验证正常。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **人工交互验收 (Human Acceptance)**：
  - 剩余唯一的交付门槛：用户在 Windows 文件资源管理器中将单个测试文件拖拽到 `kindprobe.exe` 上，确认控制台输出可读且显示“按 Enter 键关闭...”提示并不闪退。
- **后续阶段 (P1-B 规划)**：
  - Office 宏变体 (DOCM / XLSM / PPTM)、表格与幻灯片 (XLSX / PPTX / XLS / PPT)、7z/RAR 及音视频格式探测。
