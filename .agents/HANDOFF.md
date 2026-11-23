# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (后缀引导快速探测路径已就绪，乐观命中与对抗透明回退架构，37 大测试套件 163 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 后缀引导式快速探测 / 乐观命中路径架构 (Fast-Path Probing)
- **内容真确性优先与零妥协**：后缀仅作为预测线索（Prediction Hint），绝对不无脑信任后缀；必须在真实文件头部成功通过该格式的特征签名或结构验证后才能确认；
- **核心实现与规范契约** (`src/detectors/fast_path.rs`, 75 行)：严格符合 50~80 行黄金规范；实现 `probe_fast_path`，针对 ZIP 复合族系（OOXML/APK/JAR/EPUB/ODF）、CFB 二进制复合族系（DOC/XLS/PPT/MSI）、PDF、专有二进制格式（图片/归档/媒体/可执行/字体/数据库/虚拟磁盘）与 400 项声明式规则执行 O(1) 快速校验；
- **对抗样本与伪装透明回退**：若快速校验未通过（如扩展名伪装成 `.png` 的 PE 可执行文件、重命名损坏文件或未知扩展名），自动重置流偏移量并平滑回退至原有的全量探测流程（Full Scan Pipeline），准确诊断出真实类型并给出正确的 `ExtensionStatus::Mismatch`，保证对抗安全性 100%；
- **API 平滑支持** (`src/budget.rs` & `src/lib.rs`)：在 `DetectOptions` 中无缝引入 `pub extension_hint: Option<String>`；`detect_path` 自动提取路径后缀注入线索，调用方亦可手动指定 hint 进行极速探测。

### 1.2 规则引擎解耦与单规则快速比对
- **规则引擎抽取** (`src/detectors/rules/engine.rs` & `src/detectors/rules/mod.rs`)：将单个规则的匹配逻辑抽离为 `evaluate_single_rule`，并暴露 `probe_rule_by_format`，允许按 `target_fid` 瞬间定位对应规则进行单条核验，无需遍历全量分卷。

### 1.3 集成测试与对抗验证 (`tests/test_fast_path_and_mismatch.rs`)
- 新增 `test_fast_path_and_mismatch.rs` 专项测试套件，全面覆盖真实 fixture 乐观命中、声明式规则命中、对抗性样本回退与 Mismatch 诊断、截断损坏文件与未知扩展名回退；
- 累计 37 个测试套件，**163 个测试用例全部通过 (163/163 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 零警告。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (163/163 PASS)。
- **规范契约**：新增核心实现 `fast_path.rs` 精准收敛于 75 行，严格处于 50~80 行黄金区间。
- **Git 仓库状态**：代码与交接文档准备就绪，等待原子提交与推送验证。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) CI 全绿。
