# KindProbe Current Handoff

## 1. 已完成 (Completed)
- **磁盘镜像与虚拟化容器格式扩展 (P1-B 第九批)**：
  - 新增 `qcow2` (QEMU Copy-On-Write 镜像)：
    - 验证 `QFI\xFB` 4 字节头部魔数。
    - 结构化核验 QCOW2/QCOW3 版本（2 或 3）及聚簇位（cluster_bits 在 9..=21 范围，对应 512B 到 2MB 扇区聚簇）。
    - 给出 `sig.qcow.magic` 与 `sig.qcow.header` 结构化证据。
  - 新增 `vmdk` (VMware 虚拟磁盘)：
    - 支持 Sparse VMDK 二进制格式：`KDMV` 4 字节魔数 + 版本（1..=3）结构核验，输出 `sig.vmdk.sparse_magic` 与 `sig.vmdk.version`。
    - 支持 Monolithic Flat descriptor 文本描述符格式：`# Disk DescriptorFile` 头部特征核验，输出 `sig.vmdk.desc_magic`。
  - 新增 `vhdx` 与 `vhd` (微软虚拟硬盘格式)：
    - 验证 VHDX 8 字节头部签名 `vhdxfile`，输出 `sig.vhdx.magic`。
    - 验证经典 VHD 8 字节头部签名 `conectix`，输出 `sig.vhd.magic`。
  - 新增 `iso` (ISO 9660 光盘镜像)：
    - 针对第 16 扇区（offset 0x8000 / 32768 字节）主卷描述符 (PVD) 执行流探针检测（`\x01CD001` / `\x02CD001`），输出 `sig.iso.volume_descriptor`。
  - 核心架构极致瘦身：
    - 新增独立模块 `src/detectors/disks/`：`iso.rs` (~50 行), `qcow.rs` (~60 行), `vhd.rs` (~65 行), `vmdk.rs` (~75 行), `mod.rs` (~45 行)，严格控制在 80 行以内。
- **扩展名诊断与多语言同步**：
  - `src/extension.rs`：完整注册 `qcow2`, `qcow`, `vmdk`, `vhdx`, `vhd`, `iso`, `img`。
  - `src/main.rs`：补齐各项证据代码与本地化中文描述。
  - `README.md`：能力矩阵更新，标注为 Supported。
- **独立完整测试套件**：
  - 新增独立测试文件 `tests/test_disks_extended.rs`（5 个测试用例，覆盖 QCOW2 结构、VMDK 稀疏/描述符、VHD/VHDX 签名、ISO 9660 流探针、截断推断与扩展名诊断）。

## 2. 当前状态 (Current State)
- **编译与全量测试**：
  - 全量 `cargo test` **89 / 89 全部通过 (PASS)**：
    - `src/main.rs` 单元测试: 2 passed
    - `test_api_consistency`: 1 passed
    - `test_archives_extended`: 6 passed
    - `test_budget_adversarial`: 8 passed
    - `test_cfb_doc`: 2 passed
    - `test_cfb_extended`: 7 passed
    - `test_cli_contract`: 12 passed
    - `test_codex_acceptance_regression`: 9 passed
    - `test_databases_extended`: 4 passed
    - `test_disks_extended`: 5 passed (新增)
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
- **后续规划 (P1-B 第十批)**：
  - 常见数据交换格式与开发打包格式：`json`, `xml`, `tar`, `gzip` 等核心解构与深层核验。
