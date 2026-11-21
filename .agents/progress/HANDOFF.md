# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **多媒体音视频格式扩展与模块化 (P1-B 第五批)**：
  - 新增 `mp4` 与 `mov` (ISO Base Media File Format & QuickTime)：
    - 解析 box 结构，核验 `ftyp` box 长度及主要品牌（`isom`, `mp41`, `mp42`, `m4a`, `m4v`, `qt  `）。
    - 支持经典 QuickTime MOV 起始 box（`moov`, `wide`, `free`）识别。
  - 新增 `webm` 与 `mkv` (EBML / Matroska)：
    - 验证 4 字节 EBML Header 签名（`1A 45 DF A3`）。
    - 动态解析 EBML 头内部的 `DocType`（`0x42 0x82`），严格区分 `webm` 与 `matroska`。
  - 新增 `wav` 与 `webp` (RIFF Container)：
    - 验证 RIFF 头部与 FourCC 类型（`WAVE`, `WEBP`, `AVI `）。
    - 遍历子块结构，验证 WAV 的 `fmt ` 音频格式块与 WebP 的 `VP8 `/`VP8L`/`VP8X` 编码块。
    - 未确认子类型的 RIFF 容器安全降级为 `ContainerOnly`。
  - 新增 `mp3` (MPEG-1/2 Audio Layer III)：
    - 支持 ID3v2 标签头（`ID3` + synchsafe 大小计算）+ MPEG 帧同步字双重结构核验。
    - 支持直接以 MPEG Audio Layer III 帧同步字（`0xFF 0xFB/0xF3` 等）开头的原生 MP3。
  - 核心架构极致瘦身：
    - 创建精简模块目录 `src/detectors/media/`：`mp4.rs` (~115 行), `matroska.rs` (~90 行), `riff.rs` (~115 行), `mp3.rs` (~100 行), `mod.rs` (~45 行)，所有源文件严格保持在 120 行以内。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `mp4`, `m4v`, `m4a`, `mov`, `qt`, `webm`, `mkv`, `mka`, `wav`, `wave`, `webp`, `avi`, `mp3`。
  - `src/main.rs`：补齐多媒体各证据项的中文本地化映射。
  - `README.md`：能力矩阵更新，将上述音视频格式全部标注为 Supported。
- **独立完整测试套件**：
  - 新增独立测试文件 `tests/test_media_extended.rs`（7 个测试用例，覆盖 MP4、MOV、WebM、MKV、WAV、WebP、ID3 MP3、Raw MP3、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **69 / 69 全部通过 (PASS)**（耗时约 1.0 秒）：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_extension`: 2 passed
    - `test_images_extended`: 6 passed
    - `test_media_extended`: 7 passed (新增)
    - `test_ooxml_extended`: 4 passed
    - `test_zip_ooxml`: 3 passed
  - `cargo clippy --all-targets` 0 Error。
  - `cargo build` 成功。

## 3. 待办与阻塞 (Blockers & Next Steps)
- **提交与推送**：
  - 本批次修改待提交。
- **后续规划 (P1-B 第六批)**：
  - 可执行文件与可移植二进制格式（ELF, Windows PE/EXE/DLL, Mach-O, WebAssembly WASM）。
