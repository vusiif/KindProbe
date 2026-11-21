# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **源码文件瘦身与模块化重构 (Source Thinning)**：
  - 将原 910 行的单文件 `src/detectors/zip_ooxml.rs` 彻底拆分为内聚的子模块包 `src/detectors/zip_ooxml/`：
    - `mod.rs` (~380 行)：驱动主流程
    - `cd_check.rs` (~168 行)：ZIP 中央目录、EOCD 与条目冲突安全预检
    - `opc.rs` (~180 行)：OPC 路径规范化、包关系与内容类型解析
    - `schemas.rs` (~115 行)：OOXML 各格式定义与契约
    - `xml.rs` (~120 行)：主部件 XML 流式验证与嵌套深度预算拦截
  - 彻底消除了超长臃肿文件，各子文件代码量均控制在 100~380 行以内。
- **扩展支持 Office OOXML 全家族及宏变体 (P1-B 第一批)**：
  - 新增支持 `xlsx` (Excel 工作簿)、`pptx` (PowerPoint 演示文稿)。
  - 新增支持宏变体 `docm`、`xlsm`、`pptm`。
  - 严格保持零误判约束：核验对应主部件 content-type 及合法命名空间根元素（`<workbook>` / `<presentation>` / `<w:document>`）；伪造命名空间或根元素错位者安全降级为 `container_only`。
- **扩展名注册与多语言诊断同步**：
  - `src/extension.rs` 同步登记新格式扩展名映射。
  - `src/main.rs` 补齐对应证据代码的本地化说明。
  - `README.md` 格式支持矩阵与平台状态同步更新为 Windows Verified。
- **全量测试套件扩充**：
  - 新增专门的精炼测试文件 `tests/test_ooxml_extended.rs`，覆盖 XLSX/PPTX/宏变体正向检测与根元素/命名空间错位对抗性测试。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **43 / 43 全部通过 (PASS)**：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_extension`: 2 passed
    - `test_ooxml_extended`: 4 passed (新增)
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功，无任何破坏性变动。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **下一步候选 (P1-B 第二批)**：
  - 扩展高频主流独立图像格式：`jpeg` (SOI/EOI/JFIF/Exif 深度解析) 与 `gif` (87a/89a 逻辑屏幕描述符校验)。
  - 或继续扩展 CFB 旧版 Office 二进制：`xls` / `ppt`。
