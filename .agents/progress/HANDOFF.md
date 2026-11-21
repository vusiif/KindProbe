# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **数据库、列式存储与网络数据抓包格式扩展 (P1-B 第八批)**：
  - 新增 `sqlite3` (SQLite 3 数据库文件)：
    - 验证 `SQLite format 3\0` 16 字节头部魔数。
    - 深度核验页大小 (Page Size) 必须为 512..=32768 的 2 的幂次或 `1`（表示 65536 字节）。
    - 核验读写格式版本（1 = 回滚日志，2 = WAL 预写日志），输出 `sig.sqlite.header` 与 `sig.sqlite.page_size` 结构化证据。
  - 新增 `pcap` 与 `pcapng` (网络数据包转储文件)：
    - 严格识别经典 PCAP（微秒与纳秒分辨率、大端与小端序 4 种魔数变体），核验 2.4 主次版本号。
    - 严格识别 PCAPNG：核验分段头部块（Section Header Block `0x0A0D0D0A`）与字节序魔数（BOM `0x1A2B3C4D` / `0x4D3C2B1A`）。
  - 新增 `parquet` 与 `avro` (现代大数据列式与对象容器格式)：
    - 核验 Apache Parquet 4 字节头部魔数 `PAR1`。
    - 核验 Apache Avro 4 字节头部魔数 `Obj\x01`。
  - 核心架构极致瘦身：
    - 新增独立模块 `src/detectors/databases/`：`sqlite.rs` (~70 行), `pcap.rs` (~85 行), `parquet_avro.rs` (~65 行), `mod.rs` (~40 行)，全部严格控制在 90 行以内。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `sqlite3`, `sqlite`, `db`, `db3`, `pcap`, `cap`, `pcapng`, `parquet`, `avro`。
  - `src/main.rs`：补齐各项证据代码与本地化中文描述。
  - `README.md`：能力矩阵更新，标注为 Supported。
- **独立完整测试套件**：
  - 新增独立测试文件 `tests/test_databases_extended.rs`（4 个测试用例，覆盖 SQLite3 校验、PCAP/PCAPNG 差异、Parquet/Avro、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **84 / 84 全部通过 (PASS)**（耗时约 1.2 秒）：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_databases_extended`: 4 passed (新增)
    - `test_executables_extended`: 5 passed
    - `test_extension`: 2 passed
    - `test_fonts_and_ebooks`: 6 passed
    - `test_images_extended`: 6 passed
    - `test_media_extended`: 7 passed
    - `test_ooxml_extended`: 4 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **提交与推送**：
  - 本批次修改待提交。
- **后续规划 (P1-B 第九批)**：
  - 磁盘镜像与虚拟化容器格式：`iso` (ISO 9660 光盘镜像，`CD001` 卷描述符)、`vmdk` (VMware 虚拟磁盘 `KDMV` 魔数)、`qcow2` (QEMU 镜像 `QFI\xFB` 魔数)。
