# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (1000+ 格式里程碑达成；完成全维度安全审计与加固，库层全面推行 forbid(unsafe_code)，全工程 0 panic/0 unwrap/0 expect 绝对无恐慌保证，全套测试绿标)  
> **远端分支**：`origin/main` (安全加固提交已推送到主线)

---

## 1. [已完成] (Completed)

### 1.1 全维度安全审计与防御加固 (Security Audit & Hardening)
依据 `security-and-hardening` 规范实施 STRIDE 威胁建模与代码审计：
1. **内存安全加固 (Memory Safety)**：
   - 在 [`src/lib.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/lib.rs) 顶部强行注入 `#![forbid(unsafe_code)]`，从编译器级别彻底阻断探测核心库引入任何 Unsafe 行为；
   - 审计二进制入口 [`src/main.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/main.rs)，仅存在的两处 Win32 平台探测系统 API 调用补充完备的 `// SAFETY:` 论证注解，杜绝未定义行为 (UB)。
2. **绝对防恐慌拒绝服务阻断 (Zero-Panic Guarantee)**：
   - 清理并消除全工程仅存的 2 处 JSON 错误信封序列化 `.unwrap()` 调用，改用 `unwrap_or_else` 降级容错；
   - 实现整个 `src/` 代码库 **0 `unwrap()`、0 `expect()`、0 `panic!()`**。面对任何畸形、截断或恶意构造的文件输入均优雅返回结果或错误。
3. **资源耗尽与 DoS 边界防线验证 (Resource Caps & DoS Defense)**：
   - 验证 `BudgetReader` 限制读取字节上限（默认 8MB）及 ZIP 目录窗口（默认 1MB）；
   - 验证 `DecompressLimitReader` 限制解压膨胀上限（单个文件 256KB，总解压 1MB），阻断解压炸弹（Zip Bomb）；
   - 验证 XML 深度硬预算（`max_xml_depth = 64`）防御“十亿笑脸”（Billion Laughs）递归攻击；
   - 验证 CFB 目录遍历上限（`max_entries`）防御 FAT 循环链攻击；
   - 验证 ZIP 目录检测逻辑对抗同名伪造条目（`zip.conflicting_entries`）攻击。
4. **纯粹纯 Rust 依赖链与零网络暴露 (Supply Chain & Trust Boundaries)**：
   - 无任何 C/C++ 动态链接依赖库（无 `zlib-sys`、`libarchive`、`openssl`），解压由纯 Rust `miniz_oxide` 驱动；
   - 零网络栈（无 `reqwest`、`tokio`、`curl`），无外部端口监听或 SSRF 攻击面。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**187+ 个测试用例全部通过 (187+/187+ PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **Git 状态**：
  - 工作树整洁，安全加固改动已提交并推送到远端 `origin/main`。

---

## 3. [待办与下一步] (Next Steps)

1. 保持远端 CI 构建监控；
2. 持续支持后续功能扩展与新格式接入。
