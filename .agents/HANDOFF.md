# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (格式支持跨越式突破 564 项大关！新增固件取证与专业音频科学两大分卷，全量 27 个规则分卷严密收敛于 56~79 行黄金规范，41 大测试套件 172 用例全绿，Clippy 0 警告)  
> **远端分支**：`origin/main` (前次提交 `8cc7454` GitHub Actions CI 全绿通过)

---

## 1. [已完成] (Completed)

### 1.1 扩展架构至 26 个垂直分卷与 564 项格式支持
不仅维持原有 24 个分卷的高信噪比规则，还全新开辟了两个结构严谨的高价值垂直领域规则分卷，并将 `src/detectors/rules/generated/mod.rs` 精简重构至 56 行黄金规范区间：
- **固件与数字取证分卷** ([`src/detectors/rules/generated/firmware_and_forensics.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/firmware_and_forensics.rs), 69 行)：
  1. `rkfw`: Rockchip 瑞芯微固件镜像 (`RKFW`, `.rkfw`)
  2. `cbfs`: Coreboot 固件文件系统 (`ORBC`, `.cbfs`)
  3. `trx`: Broadcom 路由器 TRX 固件 (`HDR0`, `.trx`)
  4. `ubnt`: Ubiquiti / AirOS 固件包 (`OPEN`, `.ubnt`)
  5. `qnxifs`: QNX 镜像文件系统 (`\xeb\x7e\xff\x00`, `.ifs`)
  6. `simg`: Android Sparse Image 稀疏镜像 (`\x3a\xff\x26\xed`, `.simg`)
  7. `bootimg`: Android Boot Image 引导镜像 (`ANDROID!`, `.bootimg`)
  8. `evtx`: Windows 事件日志 (`ElfFile\x00`, `.evtx`)
  9. `hve`: Windows 注册表配置单元 (`regf`, `.hve`)
  10. `aff`: Advanced Forensic Format 司法取证磁盘镜像 (`AFF\x00`, `.aff`)
  11. `ewf`: Expert Witness Format / EnCase 取证镜像 (`EVF\x09\x0d\x0a\xff\x00`, `.e01`, `.ewf`)
  12. `yac`: 编译后 Yara 恶意代码特征规则二进制 (`YARA`, `.yac`)
  13. `xnb`: Microsoft XNA Game Studio 资产二进制 (`XNBw`, `.xnb`)
- **专业音频合成器与科学计算分卷** ([`src/detectors/rules/generated/pro_audio_and_science.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/generated/pro_audio_and_science.rs), 74 行)：
  14. `gig`: Tascam GigaSampler 采样音色库 (`RIFF....GIG `, `.gig`)
  15. `nki`: Native Instruments Kontakt 乐器预设 (`\x12\x90\xa8\x7f`, `.nki`)
  16. `ksd`: Native Instruments Kore Sound 音色预设 (`\x5c\x12\x34\x78`, `.ksd`)
  17. `fxp`: VST 插件单音色预设 (`CcnK`, `.fxp`)
  18. `fxb`: VST 插件音色库预设 (`CbkB`, `.fxb`)
  19. `rfl`: Propellerhead Reason ReFill 音色扩充包 (`REASON`, `.rfl`)
  20. `pcg`: Korg 工作站合成器程序音色包 (`KORG`, `.pcg`)
  21. `bnk`: Audiokinetic Wwise 游戏音效包 (`BKHD`, `.bnk`)
  22. `unityweb`: Unity Web Player 打包资源 (`UnityWeb\x00`, `.unityweb`)
  23. `e00`: ESRI Arc/Info 拓扑覆盖交换数据 (`EXP  0`, `.e00`)
  24. `envi`: ENVI 高光谱遥感数据头 (`ENVI`, `.envi`)
  25. `sg2`: SEG-2 工程浅层地震勘探波形 (`\x3a\x55`, `.sg2`)
  26. `edf`: 医学生物多导睡眠脑电图 (`0       `, `.edf`)
  27. `sas7bdat`: SAS 统计分析系统数据文件 (`\x00...\xc2\xea\x81\x60`, `.sas7bdat`)
- **全局扩展名映射表** ([`src/extension/registry.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/extension/registry.rs))：
  - 严格扩充至 **564 项**，单调递增 ASCII 字典排序，0 重复项。
- **全套自动化回归测试套件** ([`tests/test_firmware_and_science_expansion.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/tests/test_firmware_and_science_expansion.rs))：
  - 新增专用端到端测试，全面覆盖全部 27 个全新垂直格式，并验证扩展名匹配状态 (`ExtensionStatus::Match`)。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 41 个测试套件，172 个测试用例，**100% 通过 (172/172 PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/detectors/rules/generated/` 下全量 27 个分卷文件行数严格分布在 **56~79 行**区间（严守 50~80 行铁律）；
- **远端 CI 状态**：
  - 前序提交 `8cc7454` 在 GitHub Actions（macOS、Ubuntu、Windows 三端矩阵）成功全部通过。

---

## 3. [待办与阻塞] (Blockers & Next Steps)

- **阻塞项**：无阻塞。
- **下一步动作**：
  1. 执行 Git 提交并推送至远程仓库（`feat(detectors): expand format coverage to 564 formats across 26 partitions`）；
  2. 观察并确认 GitHub Actions CI 矩阵通过；
  3. 根据用户进一步指令，持续扩展更多工业格式或按需实施性能优化。
