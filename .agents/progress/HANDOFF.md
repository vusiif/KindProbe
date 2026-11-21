# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **旧版 Office 复合文档 (CFB) 模块化与扩展支持 (P1-B 第四批)**：
  - 新增 `xls` (Microsoft Excel 97-2003 BIFF8 / BIFF5)：
    - 检查根存储 `/` 直属 `Workbook` (BIFF8) 或 `Book` (BIFF5) 流。
    - 验证 BOF 记录签名（`0x0809`）与 BIFF 版本号（`0x0600` 或 `0x0500`），生成 `cfb.workbook_stream` 与 `cfb.biff_record` 证据。
    - 严格识别子存储嵌套 OLE 嵌入流与伪造非 BIFF 流，安全降级为 `ContainerOnly` 并标记告警。
  - 新增 `ppt` (Microsoft PowerPoint 97-2003)：
    - 检查根存储 `/` 直属 `PowerPoint Document` 流与 32 字节结构头。
    - 关联检查 `/Current User` 流并提供联动证据 `cfb.current_user_stream`。
  - 核心架构彻底瘦身：
    - 废除原单文件 `src/detectors/cfb_doc.rs`（275 行）。
    - 建立 `src/detectors/cfb/` 模块：`doc.rs` (~150 行), `xls.rs` (~80 行), `ppt.rs` (~65 行), `mod.rs` (~110 行)，全部保持精简。
- **扩展名映射与中英文本地化**：
  - `src/extension.rs`：完整注册 `xls`, `xlt`, `ppt`, `pot`, `pps` 互斥映射。
  - `src/main.rs`：补齐 CFB XLS 与 PPT 的证据代码与告警中文说明。
  - `README.md`：更新能力矩阵，将 `xls` 与 `ppt` 标记为 Supported。
- **全量回归与测试套件**：
  - 新建独立集成测试 `tests/test_cfb_extended.rs`（7 个测试用例，覆盖 BIFF8/BIFF5、PPT 与 Current User 联动、短流校验、非法 BOF、版本不受支持及扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **62 / 62 全部通过 (PASS)**（耗时约 1.0 秒）：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed (新增)
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_extension`: 2 passed
    - `test_images_extended`: 6 passed
    - `test_ooxml_extended`: 4 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **提交与推送**：
  - 本批次修改已完成并验证完毕，进行 Git 提交。
- **后续规划 (P1-B 第五批)**：
  - 扩展主流多媒体音视频格式：
    - `mp4` / `mov`（ISO Base Media File Format，`ftyp` box 魔数及 major/compatible brand 核验）。
    - `webm` / `mkv`（Matroska / EBML 头部签名 `1A 45 DF A3` 及 DocType 读取）。
    - `mp3`（ID3v2 标签 `49 44 33` 与 MPEG audio sync word `0xFF 0xFB/0xF3`）。
    - `wav`（RIFF 容器 `52 49 46 46` + `WAVE` 格式标志与 `fmt ` 块核验）。
