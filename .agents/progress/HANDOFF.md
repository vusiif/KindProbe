# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **源码文件瘦身与模块化重构 (Source Thinning)**：
  - `src/detectors/zip_ooxml/`：将原 910 行超长文件拆分为 5 个独立子模块（`cd_check.rs`, `opc.rs`, `schemas.rs`, `xml.rs`, `mod.rs`），单文件保持在 100~380 行。
  - `src/detectors/images/`：新建图像探测子模块包（`png.rs`, `jpeg.rs`, `gif.rs`, `mod.rs`），每个文件仅 50~100 行，内聚度极高。
  - `src/detectors/signatures.rs`：精简至约 50 行，优化大 enum 变体为 `Box<DetectionReport>`，消除大值拷贝开销与 Clippy 警告。
- **扩展支持高频独立图像格式 (P1-B 第二批)**：
  - 新增 `jpeg`：验证 SOI `\xFF\xD8\xFF`，并深度核验 APP0 (JFIF)、APP1 (Exif) 等标记段长度与结构有效性（提供 `sig.jpeg.soi` 与 `sig.jpeg.segment` 证据）；区分签名级与结构级证据。
  - 新增 `gif`：验证 `GIF87a` / `GIF89a` 魔数头，并深度解析逻辑屏幕描述符 (Logical Screen Descriptor) 宽高的非零有效性（提供 `sig.gif.header` 与 `sig.gif.logical_screen` 证据）；异常尺寸诚实降级为 `signature` 验证。
  - 完善截断检测：针对未完整写入的 JPEG / GIF 产生 `input.truncated` 警告并精准推断候选。
- **扩展名诊断与多语言支持**：
  - `src/extension.rs`：完整注册 `jpg`, `jpeg`, `gif` 等格式互斥映射与独立诊断。
  - `src/main.rs`：补齐 JPEG 与 GIF 证据代码的中文本地化说明。
  - `README.md`：能力矩阵同步更新。
- **独立精炼测试套件**：
  - 新增 `tests/test_images_extended.rs`（6 个测试用例，覆盖完整段、签名级降级、异常尺寸与截断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **49 / 49 全部通过 (PASS)**：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_extension`: 2 passed
    - `test_images_extended`: 6 passed (新增)
    - `test_ooxml_extended`: 4 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **后续规划 (P1-B 第三批)**：
  - 扩展主流归档格式：`7z` (7-Zip) 与 `rar` (严格区分 RAR4 / RAR5，防内嵌外层混淆)。
  - 扩展旧版 Office 复合文档：`xls` / `ppt`。
