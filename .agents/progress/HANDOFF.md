# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **电子书、OpenDocument 与字体格式扩展 (P1-B 第七批)**：
  - 新增 `epub` (EPUB 电子书) 与 `odt`/`ods`/`odp` (OpenDocument)：
    - 在 ZIP 容器检测流中无缝集成 `src/detectors/zip_ooxml/epub.rs` (~70 行)。
    - 严格根据 W3C / IDPF 规范，核验未压缩的首条目 `mimetype` (`application/epub+zip`) 与 `META-INF/container.xml` 结构。
    - 联动支持 ODF 标准（`application/vnd.oasis.opendocument.text` / `spreadsheet` / `presentation`）。
  - 新增 `mobi` (Mobipocket / Kindle / PalmDOC 电子书)：
    - 验证 Palm Database 头部结构及 offset 60..68 处的 `BOOKMOBI` / `BOOKTEXt` 类型创建者标识。
  - 新增 `ttf` 与 `otf` (TrueType / OpenType 字体)：
    - 验证 `\0\x01\0\0`、`true` 与 `OTTO` 魔数，并核验 sfnt 目录 `numTables` 结构有效性。
  - 新增 `woff` 与 `woff2` (Web 开放字体格式)：
    - 验证 `wOFF` 与 `wOF2` 魔数，核验头部结构中长度与表目录计数。
  - 核心架构极致瘦身：
    - 新增独立模块 `src/detectors/fonts/` (`ttf_otf.rs` ~65 行, `woff.rs` ~65 行, `mod.rs` ~35 行)。
    - 新增独立模块 `src/detectors/ebooks/` (`mobi.rs` ~55 行, `mod.rs` ~20 行)。
    - 所有新建源码文件均在 70 行以内，结构精巧，完全杜绝臃肿。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `epub`, `mobi`, `azw`, `prc`, `ttf`, `otf`, `woff`, `woff2`, `odt`, `ods`, `odp`。
  - `src/main.rs`：补齐各项证据代码与本地化中文描述。
  - `README.md`：更新能力矩阵，标注为 Supported。
- **独立完整测试套件**：
  - 新增独立测试文件 `tests/test_fonts_and_ebooks.rs`（6 个测试用例，覆盖 TTF、OTF、WOFF 1.0、WOFF 2.0、EPUB 容器、ODT 容器、MOBI 结构、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **80 / 80 全部通过 (PASS)**（耗时约 1.2 秒）：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_executables_extended`: 5 passed
    - `test_extension`: 2 passed
    - `test_fonts_and_ebooks`: 6 passed (新增)
    - `test_images_extended`: 6 passed
    - `test_media_extended`: 7 passed
    - `test_ooxml_extended`: 4 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **提交与推送**：
  - 本批次修改待提交。
- **后续规划 (P1-B 第八批)**：
  - 数据库与序列化数据格式：`sqlite3` (SQLite 3 数据库文件，`SQLite format 3\0` 魔数 + 页大小等头部字段解析)、`pcap` / `pcapng` (网络抓包文件魔数 `D4 C3 B2 A1` / `0A 0D 0D 0A`)。
