# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 604 项大关！全量 30 个规则分卷严密收敛于 59~79 行黄金规范，43 大测试套件 175 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `3e59225` GitHub Actions CI 全绿通过)

---

## 1. [已完成] (Completed)

### 1.1 扩展架构至 29 个垂直分卷与 604 项格式支持
不仅维持原有 28 个分卷的高信噪比规则，还全新开辟了汽车总线、芯片设计 EDA 与 FPGA 硬件位流专门分卷，将格式支持正式推向 **604 项** 大关，并将 `src/detectors/rules/generated/mod.rs` 紧凑维持在 62 行黄金规范区间：
- **汽车总线与芯片 EDA 硬件分卷** ([`src/detectors/rules/generated/automotive_and_eda.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/automotive_and_eda.rs), 59 行)：
  1. `mdf`: ASAM MDF4 汽车测量与校准数据格式 (`MDF4    `, `.mdf`, `.mf4`)
  2. `vcd`: IEEE 1364 Value Change Dump 数字电路逻辑仿真波形 (`$timescale`, `.vcd`)
  3. `fsdb`: Synopsys Fast Signal Database 芯片仿真波形 (`FSDB`, `.fsdb`)
  4. `edif`: IEEE 1690 电子设计交换与网表文件 (`(edif `, `.edif`)
  5. `a2l`: ASAM MCD-2 MC 汽车标定描述文件 (`ASAP2_VERSION`, `.a2l`)
  6. `svf`: JTAG 边界扫描测试向量文本 (`FREQUENCY `, `.svf`)
  7. `bit`: Xilinx FPGA 配置位流 (`\x00\x09\x0f\xf0\x0f\xf0\x0f\xf0\x0f\xf0\x00\x00\x01`, `.bit`)
  8. `dbc`: CAN 总线通信网络数据库文本 (`VERSION ""`, `.dbc`)
  9. `ldf`: LIN 汽车总线集群描述文件 (`LIN_description_file;`, `.ldf`)
  10. `bsdl`: IEEE 1149.1 JTAG 芯片边界扫描描述语言 (`entity `, `.bsd`, `.bsdl`)
  11. `jed`: JEDEC 工业逻辑器件可编程熔丝图 (`\x02\r\nNOTE JEDEC`, `.jed`)
- **全局扩展名映射表** ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))：
  - 严格扩充至 **604 项**，单调递增 ASCII 字典排序，0 重复项。
- **全套自动化回归测试套件** ([`tests/test_automotive_and_eda.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_automotive_and_eda.rs))：
  - 新增专用端到端测试，全面覆盖全部 11 个全新工业格式，并验证扩展名双向匹配状态 (`ExtensionStatus::Match`)。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 43 个测试套件，175 个测试用例，**100% 通过 (175/175 PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/detectors/rules/generated/` 下全量 30 个分卷文件行数严格分布在 **59~79 行**区间（严守 50~80 行铁律）；
- **远端 CI 状态**：
  - 前序提交 `3e59225`（591 项）在 GitHub Actions（macOS、Ubuntu、Windows 三端矩阵）成功全部通过。

---

## 3. [待办与阻塞] (Blockers & Next Steps)

- **阻塞项**：无阻塞。
- **下一步动作**：
  1. 执行 Git 原子提交并推送至远程仓库（`feat(detectors): expand format coverage to 604 formats across 29 partitions`）；
  2. 观察并确认 GitHub Actions CI 矩阵通过；
  3. 根据用户进一步指示继续扩展更多专业垂直领域格式或根据需要进行性能微调。
