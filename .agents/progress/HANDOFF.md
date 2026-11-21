# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **声明式规则驱动扩展 (P1-B 第十二批 路线 B 专业媒体/设计 + 路线 C 软件包)**：
  - 在 `src/detectors/rules/registry.rs` 中采用单一静态表 `DECLARATIVE_RULES`（~180 行定义 10 种规则），避免文件膨胀：
    - **路线 B：专业音频与设计资产**：
      - `flac`：`fLaC` 魔数 + STREAMINFO 块类型结构校验。
      - `ogg`：`OggS` 容器魔数 + 头部版本结构校验（支持 Vorbis / Opus / Theora）。
      - `psd`：`8BPS` 魔数 + 版本 1 及 6 字节保留全零校验。
      - `tiff`：支持 `II*\0`（小端）与 `MM\0*`（大端）双变体魔数 + IFD 偏移合法性核验。
      - `ico` & `cur`：Windows 图标与光标魔数 + 目录项数量合法范围校验。
    - **路线 C：软件包分发格式**：
      - `deb`：使用多段魔数切片数组（`MagicPattern`）联合核验 `!<arch>\n` 与 `debian-binary`，零样板代码支持复合头。
      - `rpm`：`0xED 0xAB 0xEE 0xDB` 魔数 + 主版本号校验。
      - `crx`：Chrome 扩展包 `Cr24` 魔数 + 格式版本号校验。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `deb`, `udeb`, `rpm`, `crx`, `flac`, `ogg`, `oga`, `ogv`, `ogx`, `opus`, `psd`, `tiff`, `tif`, `ico`, `cur`。
  - `src/main.rs`：补齐各项证据代码的中英文描述映射。
  - `README.md`：能力矩阵更新，全面支持已覆盖格式。
- **独立完整测试套件**：
  - 新增 `tests/test_declarative_rules_extended.rs`（4 个测试用例，覆盖软件包、专业音频、设计资产、大小端变体、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **104 / 104 全部通过 (PASS)**：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_compressions_extended`: 5 passed
    - `test_databases_extended`: 4 passed
    - `test_declarative_rules_extended`: 4 passed (新增)
    - `test_disks_extended`: 5 passed
    - `test_executables_extended`: 5 passed
    - `test_extension`: 2 passed
    - `test_fonts_and_ebooks`: 6 passed
    - `test_images_extended`: 6 passed
    - `test_media_extended`: 7 passed
    - `test_ooxml_extended`: 4 passed
    - `test_text_extended`: 6 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **提交与推送**：
  - 本批次修改待提交并推送到远端。
- **后续规划**：
  - 可继续将既有简单二进制探测逐步收敛至声明式表；
  - 启动基准测试 (Criterion Benchmark) 或 Fuzz 模糊测试。
