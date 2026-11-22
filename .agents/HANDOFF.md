# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (复合嵌套压缩包 tar.gz 深度识别落地，29 大测试套件 136 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 复合嵌套归档包流式探测 (`src/detectors/archives/nested.rs`, `tar.rs`, `mod.rs`)
在零额外磁盘 I/O、内存就地读取与严格预算防御（最大解压 512 字节）的前提下，实现流式复合压缩归档探测：
- **`tar.gz` (`.tar.gz`, `.tgz`) 深度解析**：
  - 流式解压探测：通过 `flate2::read::GzDecoder` 尝试就地解压前 512 字节（受限预算，单次解压仅纳秒级）；
  - 核心结构验证：调用 `check_tar_header` 严格校验 POSIX `ustar` 魔数及八进制校验和（Checksum）；
  - 嵌套证据链生成：输出 `sig.tar.embedded`、`sig.tar.ustar`、`sig.tar.checksum`，并将容器精确标记为 `container_id: Some("gzip")`；
  - 优雅降级：非 tar 内容的普通 Gzip 文件（如 `hello.txt.gz`）解压校验不命中时，平滑降级为常规 `gzip` 探测结果，推荐 `.gz`。

### 1.2 复合扩展名诊断机制 (`src/lib.rs`, `src/extension/registry.rs`)
- **多段后缀解析**：在 `detect_path` 中新增 `resolve_path_extension`，支持识别 `.tar.gz`、`.tar.bz2`、`.tar.xz`、`.tar.zst` 等双扩展名复合结构；
- **扩展名索引表扩展**：在 `registry.rs` 中按严格 ASCII 顺序登记 `("tar.gz", "tar.gz")` 与 `("tgz", "tar.gz")`，全局扩展名表扩至 **284 项**；
- 诊断覆盖：对 `pkg.tar.gz`、`pkg.tgz`、`pkg.gz` 均能精准确认为 `ExtensionStatus::Match`，而误标扩展名（如 `pkg.zip`）准确诊断为 `ExtensionStatus::Mismatch`。

### 1.3 专项集成测试与 CLI 端到端验证 (`tests/test_nested_archives.rs`)
- 覆盖普通纯文本 Gzip 与嵌套 Tar Gzip 的差异化识别；
- 覆盖 `tar.gz`、`tgz`、`gz`、`zip` 扩展名诊断矩阵；
- 端到端验证 CLI 中英文输出与 JSON 模式反序列化断言；
- 累计 29 个测试套件，**136 个测试用例全部通过**。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (136/136 PASS)。
- **Git 仓库状态**：准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 提交代码并推送至 `origin/main`；
  2. 监控 GitHub Actions 确认全平台 (Ubuntu, macOS, Windows) 全绿；
  3. 响应用户并进行交接。
