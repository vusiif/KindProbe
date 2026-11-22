# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (扩展支持车载BLF/PCB Gerber/CBOR/LZO/MiniDump/Keybox/世嘉Genesis/C64/TIM2/TPL/FLIC，全局扩展名达 386 项，35 大测试套件 156 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 垂直领域与专有格式全景扩展
全部分卷与实现文件严格保持在 50~80 行黄金规范区间（55~76 行）：
- **车载遥测与电子工程** (`robotics_firmware.rs`, 68 行)：支持 Vector CAN/LIN/FlexRay 二进制日志 (`.blf`, `LOGG`) 与 RS-274X Extended Gerber PCB 印刷电路制程格式 (`.gbr`/`.ger`, `%FS`)；
- **物联网与二进制序列化** (`database.rs`, 75 行)：支持 RFC 8949 自描述二进制结构化对象 (`.cbor`, `\xd9\xd9\xf7`)；
- **实时高吞吐压缩归档** (`packages.rs`, 76 行)：支持 lzop 归档包 (`.lzo`, `\x89LZO\x00\r\n\x1a\n`)；
- **系统崩溃转储与诊断** (`disk_images.rs`, 72 行)：支持 Windows MiniDump 崩溃转储 (`.dmp`/`.mdmp`, `MDMP\x93\xa7`)；
- **密码学凭据与安全密钥库** (`crypto.rs`, 62 行)：支持 GnuPG Keybox 密钥环格式 (`.kbx`, 偏移 8 `KBXf`)；
- **经典动画与视频** (`vector_graphics.rs`, 61 行)：支持 Autodesk Animator FLIC 序列动画 (`.flc`/`.fli`, 偏移 4 `\x12\xaf`)；
- **经典主机 ROM 与数字资产** (`game_assets.rs`, 75 行 & `roms.rs`, 76 行)：支持世嘉五代 Sega Genesis / Mega Drive (`.gen`/`.smd`, 偏移 256 `SEGA`)、Commodore 64 磁带镜像 (`.t64`, `C64 tape image file`) 与 Creative Voice 游戏音频 (`.voc`, `Creative Voice File\x1a`)；
- **主机平台专有硬件纹理** (`textures.rs`, 68 行)：支持 PlayStation 2 TIM2 纹理 (`.tm2`, `TIM2`) 与 Nintendo GameCube / Wii TPL 调色板纹理 (`.tpl`, `\x00\x20\xaf\x30`)。

### 1.2 扩展名索引表扩至 386 项 (`src/extension/registry.rs`)
- 严格按 ASCII 字典单调升序排序，新增 `blf`, `cbor`, `dmp`, `flc`, `fli`, `gbr`, `gen`, `ger`, `kbx`, `lzo`, `mdmp`, `smd`, `t64`, `tm2`, `tpl`, `voc`；
- 通过 `test_registry_is_strictly_sorted_without_duplicates` 守卫测试，二分查找保持零堆分配与 ~36 ns / op。

### 1.3 集成测试与全量验证 (`tests/test_telemetry_and_consoles.rs`)
- 新增 `test_telemetry_and_consoles.rs` 专项测试套件，全面覆盖车载遥测、PCB制程、崩溃转储、CBOR、LZO、主机ROM与硬件纹理；
- 累计 35 个测试套件，**156 个测试用例全部通过 (156/156 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 零警告。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (156/156 PASS)。
- **文件行数规范**：全部 20 大规则分卷严格保持在 55~76 行黄金区间。
- **Git 仓库状态**：已准备原子提交并推送远程仓库。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) CI 全绿。
