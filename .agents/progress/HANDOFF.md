# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **扩展支持主流归档格式 (P1-B 第三批)**：
  - 新增 `7z` (7-Zip 归档格式)：
    - 验证 6 字节魔数签名（`37 7A BC AF 27 1C`）。
    - 深度核验 32 字节起始头 (StartHeader) 中的 CRC32 校验和，提供 `sig.7z.header` 与 `sig.7z.start_header` 证据（`Verification::Structure`）。
    - 异常或短文件安全降级为 `Verification::Signature`，截断输入推断候选 `7z`。
  - 新增 `rar` (RAR 归档格式)：
    - 严格区分并核验 RAR4（7 字节签名 + `0x73` 主档案头验证）与 RAR5（8 字节扩展签名）。
    - 区分结构级与签名级证据；截断输入产生 `input.truncated` 并推断候选 `rar`。
  - 架构瘦身与模块化：
    - 新增独立子模块包 `src/detectors/archives/`（`sevenz.rs` ~70 行, `rar.rs` ~85 行, `mod.rs` ~35 行），完全杜绝单文件臃肿。
- **扩展名诊断与多语言支持同步**：
  - `src/extension.rs`：完整注册 `7z` 与 `rar` 互斥映射。
  - `src/main.rs`：补齐 7z 与 RAR 各证据代码的中文本地化说明。
  - `README.md`：能力矩阵更新，将 7-Zip 与 RAR 标注为 Supported。
- **独立精炼测试套件**：
  - 新增独立测试文件 `tests/test_archives_extended.rs`（6 个测试用例，覆盖 StartHeader CRC32 验证、RAR4 主头、RAR5 签名、签名级降级与截断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **55 / 55 全部通过 (PASS)**（耗时约 1.0 秒）：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed (新增)
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
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
  - 本批次修改待提交。
- **后续规划 (P1-B 第四批)**：
  - 扩展旧版 Office 复合文档（CFB）：`xls` (Excel 97-2003 BIFF8/5) 与 `ppt` (PowerPoint 97-2003)。
  - 音视频媒体（MP4 / MOV / WebM / MP3 / WAV）。
