# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **可执行文件与系统二进制格式扩展与模块化 (P1-B 第六批)**：
  - 新增 `elf` (Linux/Unix ELF 二进制)：
    - 验证 `\x7FELF` 魔数，深度校验 `EI_CLASS` (32/64 位)、`EI_DATA` (大小端序) 与版本有效性。
    - 验证 `e_type`（执行文件 `ET_EXEC`、动态共享库 `ET_DYN`、目标文件 `ET_REL`），生成结构化证据。
  - 新增 `pe` (Windows Portable Executable: EXE, DLL, SYS, EFI)：
    - 验证 DOS `MZ` 头部签名与 `0x3C` 处的 `e_lfanew` 指针。
    - 验证 `PE\0\0` 签名并解析 COFF 头 Characteristics 标志位，精准区分 `.exe` 与 `.dll`。
    - 兼顾未包含 PE 头的原生 DOS MZ 可执行文件（平滑降级为 Signature 级别）。
  - 新增 `wasm` (WebAssembly 二进制格式)：
    - 验证 `\0asm` (`00 61 73 6D`) 4 字节魔数及版本号 1 (`01 00 00 00`)。
  - 新增 `macho` 与 `class` (macOS Mach-O 二进制与 Java Class 字节码)：
    - 支持 32/64 位单架构 Mach-O（`0xFEEDFACE`, `0xFEEDFACF` 及其 LE 变体）。
    - 针对 `0xCAFEBABE` 魔数进行智能消歧：通过次版本号与主版本号（JDK 1.1~21+）精准区分 Java `.class` 字节码与 Mach-O Universal FAT 二进制。
  - 核心架构极致瘦身：
    - 创建精简模块目录 `src/detectors/executables/`：`elf.rs` (~75 行), `pe.rs` (~100 行), `wasm.rs` (~55 行), `macho.rs` (~115 行), `mod.rs` (~45 行)，严格遵守单文件瘦身要求。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `elf`, `so`, `o`, `exe`, `dll`, `sys`, `efi`, `wasm`, `dylib`, `class`。
  - `src/main.rs`：补齐系统二进制各证据代码的中文本地化说明。
  - `README.md`：能力矩阵更新，将上述格式全部标注为 Supported。
- **独立完整测试套件**：
  - 新增独立测试文件 `tests/test_executables_extended.rs`（5 个测试用例，覆盖 ELF 64/32 位、Windows PE EXE/DLL/DOS MZ、WASM、Mach-O、Java Class 消歧、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **74 / 74 全部通过 (PASS)**（耗时约 1.2 秒）：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_executables_extended`: 5 passed (新增)
    - `test_extension`: 2 passed
    - `test_images_extended`: 6 passed
    - `test_media_extended`: 7 passed
    - `test_ooxml_extended`: 4 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **提交与推送**：
  - 本批次修改待提交。
- **后续规划 (P1-B 第七批)**：
  - 电子书与流式文档格式：`epub` (EPUB 电子书，ZIP 容器 + mimetype 零压缩验证 + META-INF/container.xml 结构解析)、`mobi` (PalmDOC/Mobipocket 魔数)、`fb2`。
  - 字体格式：`ttf` / `otf` / `woff` / `woff2`。
