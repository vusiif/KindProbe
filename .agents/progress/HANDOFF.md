# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **压缩、打包与归档容器格式扩展 (P1-B 第十批)**：
  - 新增 `tar` (POSIX Tar 归档格式)：
    - 验证 512 字节头部块与 offset 257 处 `ustar\0` / `ustar ` 魔数。
    - 深度计算校验和：将 148..156 字节视为空格（0x20）累加全部无符号字节，与记录的八进制 Checksum 进行精确核验，输出 `sig.tar.ustar` 与 `sig.tar.checksum` 结构化证据。
  - 新增 `gzip` (RFC 1952 Gzip 压缩容器)：
    - 验证 `\x1F\x8B` 2 字节魔数。
    - 结构化核验压缩方法 CM = 8 (DEFLATE) 以及标志位 FLG 高 3 位保留位为 0，输出 `sig.gzip.magic` 与 `sig.gzip.header`。
  - 新增 `bzip2` (Bzip2 压缩容器)：
    - 验证 `BZh` 3 字节魔数。
    - 结构化核验块大小 (`'1'`..=`'9'`) 与 Pi 常数块特征魔数 `0x31 0x41 0x59 0x26 0x53 0x59`（或 EOS 标记），输出 `sig.bzip2.magic` 与 `sig.bzip2.header`。
  - 新增 `xz` (XZ 压缩容器)：
    - 验证 `\xFD7zXZ\x00` 6 字节魔数。
    - 验证 Stream Flags 并通过自主实现的零依赖 IEEE CRC32 算法进行精确完整性校验，输出 `sig.xz.magic` 与 `sig.xz.crc`。
  - 新增 `zstd` (Zstandard 压缩容器)：
    - 验证 `0x28 0xB5 0x2F 0xFD` 4 字节魔数，输出 `sig.zstd.magic`。
  - 核心架构极致瘦身：
    - 在 `src/detectors/archives/` 下拆分增加：`crc32.rs` (~25 行), `tar.rs` (~75 行), `gzip.rs` (~65 行), `bzip2.rs` (~75 行), `xz_zstd.rs` (~80 行)，全面模块化且单文件均在 80 行以内。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `tar`, `gz`, `gzip`, `tgz`, `bz2`, `bzip2`, `tbz2`, `tbz`, `xz`, `txz`, `zst`, `zstd`, `tzst`。
  - `src/main.rs`：补齐各项证据代码与本地化中文描述。
  - `README.md`：能力矩阵更新，标注为 Supported。
- **独立完整测试套件**：
  - 新增独立测试文件 `tests/test_compressions_extended.rs`（5 个测试用例，覆盖 Tar 校验和、Gzip 头部、Bzip2 块结构、XZ/ZSTD 校验、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **94 / 94 全部通过 (PASS)**：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_compressions_extended`: 5 passed (新增)
    - `test_databases_extended`: 4 passed
    - `test_disks_extended`: 5 passed
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
  - 本批次修改待提交并推送到远端。
- **后续规划 (P1-B 第十一批)**：
  - 结构化文本与标记数据格式：`json`, `xml`, `html`, `yaml`, `toml` 等安全嗅探与文本消歧。
