# KindProbe 阶段收尾全项目审查报告 (Final Whole-Project Review)

**审查日期:** 2026-11-02T21:10:00+08:00  
**审查员:** CAO Node 9890 全项目审查员 (`agy_whole_reviewer`, lolicon)  
**工作目录:** `/srv/projects/rust/KindProbe`  
**Git 基线:** 分支 `main`（尚未创建任何提交，未提交工作区快照）  
**阶段定位:** **P0（路线与复用验证）阶段收尾**，尚未进入 P1  
**前序状态:** 开发者 `kindprobe_agy_developer` Round 1 修复自测完成；独立验证员 `kindprobe_claude_verifier` Round 1 独立复验 PASS（52 组夹具 False ID=0, Miss=0, Crash=0，Release 二进制 SHA256 `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`）  
**本审查性质:** 源码级全局静态审查、接口与架构一致性审视、规范条款对照与缺口分析（不执行构建与测试运行）

---

## 1. 发现的问题与缺陷 (Findings to Fix)

以下列出全项目源码审查中独立发现的问题与潜在缺陷。依据严重度排序，严格标注触发条件、影响范围及结论性质（`inferred` 源码推断 / `reproduced` 已实测复现）。

> **阻断性评估结论**：**无阻断性缺陷 (No Blocking Defects)**。所发现的问题均属于深层边界、防御性实现偏差或特定极端构造下的非预期行为，未破坏当前固定 P0 门槛集（False ID 仍为 0），不构成打回重测的阻断条件，建议由主管建档后在进入 P1-A 迭代时修复。

### 1.1 [中度缺陷 · 推断] MS-DOC FIB `fcMin` 与 `fcMac` 偏移量硬编码错位
- **位置:** `src/detectors/cfb_doc.rs:198-208`
- **触发条件:** 探测 Word 97+ 二进制 DOC 文件中的文本流偏移有效性。
- **规范与预期:** 依据微软官方规范 [MS-DOC] 2.5.1 `FibBase` 结构定义，FibBase 总大小为 32 字节（0x20）。其中 `fcMin`（4 字节）位于偏移 24..28（0x18..0x1C），`fcMac`（4 字节）位于偏移 28..32（0x1C..0x20）。应比对这两处以断言 `fcMin <= fcMac`。
- **实际表现:** 
  ```rust
  let fc_min = u32::from_le_bytes([fib_header[32], fib_header[33], fib_header[34], fib_header[35]]);
  let fc_mac = u32::from_le_bytes([fib_header[36], fib_header[37], fib_header[38], fib_header[39]]);
  ```
  代码从偏移 32..36 读取 `fc_min`，从偏移 36..40 读取 `fc_mac`。实际上偏移 32 处是 FibBase 结束后的 `csw`（2 字节，表示 `fibRgW` 数组长度）及 `fibRgW` 的前两个字段。
- **后果:** 代码实际比较的是 `csw` 和 `fibRgW` 数据，而非真实的文本流起始/结束偏移 `fcMin` 与 `fcMac`。由于标准 Word 97 文档中 `csw` 通常为固定小整数（0x000E），碰巧小于后续数据，故在现有样本中未触发报错，但此处的逻辑属于结构字段解析错误。
- **性质:** `inferred`（源码推断，依据 [MS-DOC] 规范确凿）。

### 1.2 [中度缺陷 · 推断] XML 深度预算超限时无条件将未校验根元素的文件晋升确认为 DOCX
- **位置:** `src/detectors/zip_ooxml.rs:604-620`, `793-802`
- **触发条件:** 传入具有合法 OOXML 包关系（`_rels/.rels`）与内容类型（`[Content_Types].xml`），但主部件 `word/document.xml` 根元素**非** `<w:document>` 且 XML 嵌套深度超过 64 的恶意文件。
- **规范与预期:** 规格 §6.2 明确规定：“`format + limited` 仅用于格式身份所必需的证据已完整取得，但可选或后续规定检查受限的情形；若身份必需步骤受限，应降为 container_only 或 unknown，不得保留伪确认”。由于主部件是否属于 Word 文档核心未被确认，应当降级为 `container_only` + `limited`。
- **实际表现:** 
  `verify_main_document_xml` 在 `current_depth > max_depth` 时立即提前返回 `Err(WarningItem("budget.xml_depth_exceeded"))`，此时内部可能尚未确认根标签是否合法（甚至根标签是其他非 Word 节点）；而外层 `detect_zip_ooxml:605` 捕获到 `budget.xml_depth_exceeded` 后，无条件将其设置成：
  ```rust
  report.identification = Identification::Format;
  report.format_id = Some("docx".to_string());
  report.inspection = Inspection::Limited;
  ```
- **后果:** 攻击者可通过构造假根标签 + 超深嵌套的 XML，诱使检测引擎越过根标签合法性验证，将非 DOCX 的 OOXML 包错误确认晋升为 DOCX（违背 §6.2 约束）。
- **性质:** `inferred`（源码推断）。

### 1.3 [中/低度缺陷 · 推断] CFB 遍历未限定流为根级直属，子存储嵌入对象同名流可能误判整文件格式
- **位置:** `src/detectors/cfb_doc.rs:83-90`
- **触发条件:** 输入包含 OLE 嵌入 Word 对象的复合文档（如 Excel `.xls` 文件内嵌入了一个 Word 表格，生成了子存储 `/MBD00012345/WordDocument`）。
- **规范与预期:** 规格 §5 明确要求：“CFB 中的 Excel/PowerPoint 流、嵌入 OLE、加密 OOXML 包不能被当 DOC”。独立 Word 文档的 `WordDocument` 流必须为复合文档根存储下的顶级流。
- **实际表现:** 代码使用 `comp.walk()` 遍历整棵复合文档树，一旦找到任意匹配 `entry.name().eq_ignore_ascii_case("WordDocument")` 的流即记录为其路径并进行 FIB 检验，未校验 `entry.parent()` 是否为 Root Storage。
- **后果:** 若一个包含 OLE 嵌入 Word 对象的 Excel 电子表格传入，一旦其内嵌 Word 流 FIB 合法，整个复合文档将被误判确认为 `format_id: doc`。
- **性质:** `inferred`（源码推断）。

### 1.4 [低度缺陷 · 推断] `detect_bytes` 与 `detect_reader` 未调用后缀比较导致违背规格 §6.3 默认 `missing` 约定
- **位置:** `src/lib.rs:52-61` 及 `src/detectors/mod.rs`
- **触发条件:** 调用方使用 `detect_bytes` 或 `detect_reader` API 进行格式识别。
- **规范与预期:** 规格 §6.3 优先级 1 明确规定：“1. 调用方没有提供可用后缀：`missing`，无论是否已确认格式”。
- **实际表现:** `detect_stream` 各分支将初始 `extension_status` 设为 `Undetermined`。在 `detect_path` 中会随后调用 `compare_extension(&report, path.extension())`；但在 `detect_bytes` 和 `detect_reader` 中，未执行 `compare_extension(&report, None)`，导致返回报告中 `extension_status` 恒为 `Undetermined`。
- **后果:** 库 API 契约不完全一致：通过 `detect_path` 探测无后缀文件返回 `Missing`，而通过 `detect_bytes` 探测同一内容返回 `Undetermined`。
- **性质:** `inferred`（源码审查直接证据）。

### 1.5 [低度缺陷 · 推断] `tests/test_cli_contract.rs` 引入 Unix 专有模块缺少 `#[cfg(unix)]` 守卫
- **位置:** `tests/test_cli_contract.rs:84`
- **触发条件:** 在 Windows 宿主环境下直接运行 `cargo test`。
- **规范与预期:** 跨平台集成测试中，平台专有代码必须由 `#[cfg(...)]` 保护（如 `tests/test_extension.rs:55` 所示）。
- **实际表现:** `test_cli_contract.rs:84` 的 `test_cli_non_utf8_arguments_and_paths` 未加 `#[cfg(unix)]` 条件编译，直接声明 `use std::os::unix::ffi::OsStrExt;`。
- **后果:** 在非 Unix 平台（如 Windows）编译测试时，测试套件将直接编译失败中断。
- **性质:** `inferred`（源码审查直接证据）。

### 1.6 [代码质量 · 推断] PDF 签名探针中的版本号检查分支实为无操作 (No-Op)
- **位置:** `src/detectors/signatures.rs:38-47`
- **触发条件:** 探测任意带有 `%PDF-` 签名的输入。
- **实际表现:**
  ```rust
  if pos + 7 <= header.len() {
      let ver = &header[pos + 5..pos + 7];
      if ver[0].is_ascii_digit() && ver[1] == b'.' {
          return SignatureMatch::Pdf;
      }
  }
  return SignatureMatch::Pdf;
  ```
  无论版本号是否满足数字加点，外部均无条件执行 `return SignatureMatch::Pdf;`，内部条件分支无任何分流意义。
- **性质:** `inferred`（源码审查直接证据）。

### 1.7 [防御性质量 · 推断] `detect_path` 文件元数据检查存在微小 TOCTOU 竞态
- **位置:** `src/lib.rs:28-40`
- **触发条件:** 在多进程并发或外部文件快速变动的环境下调用 `detect_path`。
- **实际表现:** 代码先调用 `std::fs::metadata(path)` 检查 `is_file()`，随后才调用 `File::open(path)`。根据规格 §7 / §9.3“文件打开后检查句柄属性，目录/管道/设备拒绝”的要求，应当在打开后的 `File` 句柄上调用 `file.metadata()?.is_file()`，避免检查与打开之间产生路径替换竞态。
- **性质:** `inferred`（源码审查直接证据）。

### 1.8 [防御性质量 · 推断] OPC 部件名称查找为大小写敏感，不完全贴合 ECMA-376 规范
- **位置:** `src/detectors/zip_ooxml.rs:767-774`
- **触发条件:** DOCX 包内 `[Content_Types].xml` 中的 PartName 与 `_rels/.rels` 中的 Target 出现大小写混用（如 `/Word/Document.xml` vs `/word/document.xml`）。
- **实际表现:** 使用标准 `HashMap::get` 进行大小写敏感查找。根据 ECMA-376 Part 2 Section 9.1.1，OPC 部件名称比较规范为 ASCII 大小写不敏感。如果大小写不一致且 defaults 未覆盖该扩展名，可能导致无法查找到对应 ContentType。
- **性质:** `inferred`（源码推断）。

---

## 2. 整体设计与架构质量 (Overall Design & Quality)

### 2.1 架构设计与模块分层
1. **核心统一性与适配器模式**:
   - `kindprobe` 严格遵循了统一探测核心的设计思路。`detect_path`、`detect_bytes` 与 `detect_reader` 共同基于 `BudgetReader<R>` 包装流，消除了不同调用入口的判定分歧。
   - 内容识别与扩展名诊断严格解耦，`compare_extension` 作为独立纯函数运作，保证了文件名提示不污染内容证据链。
2. **合法状态契约与自校验**:
   - `DetectionReport::validate_invariants()` 在每次探测返回前强制校验规格 §6.2 状态矩阵（如禁止 `format + ambiguous`，禁止无格式时提供推荐扩展名，禁止 `verification=none` 与已确认格式并存），构筑了稳固的运行时防御边界。
3. **分层降级与错误信封**:
   - 区分了底层真实 I/O 故障（冒泡为 `Err(DetectError::Io)`，CLI 退出码 4）、格式截断（`input.truncated`）、预算耗尽（`inspection: limited`）以及内容歧义（`zip.conflicting_entries`），错误处理模型设计层次清晰。

### 2.2 资源预算与边界保护
1. **多重预算护栏覆盖**:
   - 首次头部窗口（4 KiB）、最大总读取字节（8 MiB）、ZIP 目录读取窗口（1 MiB）、容器条目数（10,000）、单 XML 解压上限（256 KiB）、总解压上限（1 MiB）、XML 嵌套深度（64）均已在 `DetectOptions` 中建模并在对应解析器中有效生效。
2. **预算缺口**:
   - `BudgetReader` 记录了 `seek_count`，但未配置 `max_seek_count` 上限。对于极端设计的碎片化流，理论上存在大量零读取 `seek` 导致 I/O 放大的隐患。
3. **度量指标说明**:
   - `scripts/run_release_measurements.py` 中记录的峰值常驻内存（~27.7 MB）经审查系外部测试脚本通过 `/usr/bin/time -v docker run ...` 测得的宿主 Docker 客户端进程内存，而非 `kindprobe` 独立二进制本身的 RSS（Release 二进制无动态库开销，内建内存预计远小于此）。

---

## 3. 下一阶段功能建议 (Next-Step Proposals)

> [!IMPORTANT]
> **重要声明**：以下建议仅为基于当前项目代码审视后提出的演进构想，**绝不构成开发授权**，本轮审核不会执行，也不得擅自开始下一阶段实施。

| 序号 | 建议方向 | 用户价值 | 前置条件 | 预估规模 |
|---|---|---|---|:---:|
| **P1-1** | **P1-A 阶段 API 与契约加固** | 修复审查发现的 8 项缺陷与防御性缺口（特别是 FIB fcMin/fcMac 偏移错位、XML 深度与根标签复合判定、`detect_bytes` missing 状态对齐）。 | P0 人工验收通过，主管下达 P1-A 任务。 | **S** (1-2 天) |
| **P1-2** | **Office 宏变体与 XLSX/PPTX 扩展 (P1-B)** | 扩展支持 DOCM、XLSM、PPTM 宏格式识别，以及电子表格与演示文稿容器分流，提升真实办公场景可用性。 | 完成 PRONOM 对应规则与 ECMA-376 关系命名空间审查，合成无侵权夹具。 | **M** (3-5 天) |
| **P1-3** | **跨平台 (Windows / macOS) 编译与 CI 守卫** | 解决 Windows 平台下 `OsStrExt` 兼容性问题，打通跨平台 CI 矩阵与文件对象防护测试。 | 准备 Windows / macOS 构建测试流水线。 | **S** (1-2 天) |
| **P1-4** | **Seek 计数预算上限与流控防放大** | 在 `BudgetReader` 中增加 `max_seek_count` 限制，防御恶意构造的高频 seek 拒绝服务攻击。 | 测量各类大文件格式平均所需 seek 次数基准。 | **S** (1 天) |

---

## 4. 覆盖范围、证据划分与基线评估 (Coverage & Limits)

### 4.1 证据划分明确声明
依照审计规程，本项目所有结论严格区分为以下三类：
1. **定向测试已覆盖的 (Covered by Targeted Tests)**:
   - 52 组正反向夹具在 Docker 沙箱隔离环境中的实测行为（PNG/PDF/ZIP/DOCX/DOC 内容识别、真假 OOXML 分流、真假 Word 流分流、预算耗尽降级、I/O 错误传播为 exit code 4、非 UTF-8 参数保护等）。
   - Release 二进制 SHA256 `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68` 与大小 `895,928` 字节的构建重现性。
2. **全项目源码审查发现的 (Discovered by Whole-Project Source Review)**:
   - 节 1 中详述的 8 项推断性缺陷与设计隐患（FIB 偏移读取超出 FibBase、XML 深度超限无条件确认为 DOCX、子存储 OLE 嵌入混淆、`detect_bytes` 扩展名状态偏差、Windows 模块守卫缺失等）。
3. **仍待人工验收的 (Pending Human Acceptance)**:
   - 项目最终开源许可证指定；
   - main 分支代码的提交 (git commit) 与远程推送 (git push) 最终授权；
   - 是否对本报告中的中低度缺陷进行修复再提交，或是直接进入人工验收流程。

### 4.2 审查覆盖与边界说明
- **审查基线:** 本地 WSL 工作区 `/srv/projects/rust/KindProbe`，分支 `main`（无提交，脏状态稳定 checkpoint）。
- **已审查自有源码:** `src/main.rs`, `src/lib.rs`, `src/model.rs`, `src/budget.rs`, `src/extension.rs`, `src/detectors/mod.rs`, `src/detectors/zip_ooxml.rs`, `src/detectors/cfb_doc.rs`, `src/detectors/signatures.rs`（100% 覆盖）。
- **已审查测试与度量:** `tests/*.rs`（6 套测试文件全部审查），`scripts/*.py`（全部复验与度量脚本），`Cargo.toml`, `Cargo.lock`。
- **已审查规范与研发产物:** `docs/FILE_FORMAT_RUST_SPEC_V1_2.md`, `research/*.md`, `research/*.json`, `HANDOFF.md`, `INDEPENDENT_VERIFICATION_R1.md`。
- **略过与未检查区域:** 第三方依赖 crate 内部源码（`zip`, `cfb`, `quick-xml` 仅审核接口和依赖链，未深入其私有实现）；规格 §11.4 提及的 Toss 本机语料严格隔离未读；其他操作系统平台（Windows/macOS）未作实机测试。

### 4.3 P0 硬门槛整体达成结论
| 门槛编号 | 门槛要求 | 审计评定 | 评定依据与说明 |
|---|---|:---:|---|
| **P0-G0** | 现成方案对照与来源/许可审查 | **PASS** | 完整具备 `research/candidates.md`、`baseline-results.json` 等 6 份研发记录，决策明确。 |
| **P0-G1** | 真假 DOCX/ZIP、真假 DOC/CFB 分流 | **PASS** | 52 组测试中分流准确，伪造容器 100% 降级为 `container_only`。 |
| **P0-G2** | 固定测试集误识别为 0 (False ID = 0) | **PASS** | 实测 52 组用例误判数为 0。 |
| **P0-G3** | 预算与异常降级并输出可解释诊断 | **PASS** | 深度、窗口、总量、条目预算及截断降级均有自动化用例与稳定诊断码。 |
| **P0-G4** | 核心共用、后缀独立、真实 I/O 冒泡、无副作用 | **PASS** | 3-API 核心共用，真实 I/O 错误退出码 4，库无写文件/打印/联网副作用。 |
| **P0-G5** | 真实 release 构建与指标度量 | **PASS** | 沙箱全新编译 SHA256 吻合，二进制 875 KiB，执行耗时 < 1 ms。 |
| **P0-G6** | 依赖复用决策、能力与未支持清单 | **PASS** | `README.md` 与交接文档清晰列出当前支持及 P1-B 递延范围。 |

**总体评估结论**：项目当前在 Linux x86_64 目标平台下已完整满足规格书定义的 **P0 全部 7 项硬门槛 (P0-G0 至 P0-G6)**。本审查所揭示的缺陷均为代码防御性深层边界问题，**无阻断性缺陷**。工作区已具备向主管汇报、待人工验收及后续阶段规划的基础。
