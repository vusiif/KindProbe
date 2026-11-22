# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-22  
> **当前版本**：v0.1.0 (全局扩展名突破 400 项大关！扩展支持 TeX/DVI/Age/PPK/Pack200/InstallShield/SunRaster/HDR/FLI/BinaryEPS/PVRT/DfuSe/RData，36 大测试套件 159 用例全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 格式矩阵扩展并突破 400 项里程碑
全部分卷与实现文件严格保持在 50~80 行黄金规范区间（67~76 行）：
- **排版系统与学术文档** (`fonts.rs`, 69 行)：支持 TeX Packed 栅格字体 (`.pk`, `\xf7\x59`) 与 TeX 设备无关文档 (`.dvi`, `\xf7\x02`)；
- **现代安全加密与远程凭据** (`crypto.rs`, 76 行)：支持 Filippo 现代 Age 加密容器 (`.age`, `age-encryption.org/v1\n`) 与 PuTTY SSH 专用私钥 (`.ppk`, `PuTTY-User-Key-File-`)；
- **企业分发与嵌入式归档** (`archives.rs`, 75 行)：支持 Java Pack200 压缩 JAR 包 (`.pack`, `\xca\xfe\xd0\x0d`) 与 InstallShield CAB 封包 (`.isc`/`.hdr`, `ISc(`)；
- **经典工作站与高动态渲染** (`netpbm.rs`, 76 行)：支持 Sun Raster 图元 (`.ras`/`.sun`, `\x59\xa6\x6a\x95`) 与 Radiance 高动态范围图像 (`.hdr`, `#?RADIANCE\n`)；
- **矢量图形与序列流式动画** (`vector_graphics.rs`, 75 行)：支持 Autodesk Animator FLI 动画 (`.fli`, 偏移 4 `\x11\xaf`) 与 DOS 二进制封装 PostScript (`.eps`, `\xc5\xd0\xd3\xc6`)；
- **主机专有硬件纹理** (`textures.rs`, 75 行)：扩展支持 Sega Dreamcast PowerVR 纹理容器 (`.pvr`/`.pvrt`, `PVRT`)；
- **微控制器芯片固件** (`robotics_firmware.rs`, 75 行)：支持 STMicroelectronics DfuSe 芯片固件升级包 (`.dfu`, `DfuSe\x01`)；
- **数据科学与统计计算** (`scientific.rs`, 76 行)：支持 R 语言工作区与 RDS 序列化数据 (`.rdata`/`.rds`/`.rda`, `RDX2\n`)。

### 1.2 扩展名索引表突破 400 项 (`src/extension/registry.rs`)
- 扩展名总数由 386 项跃升至 **400 项**；
- 严格按 ASCII 字典升序单调排列，通过 `test_registry_is_strictly_sorted_without_duplicates` 单调性与无重复硬性断言测试。

### 1.3 集成测试与质量保障 (`tests/test_publishing_and_security.rs`)
- 新增 `test_publishing_and_security.rs` 专项测试套件，全面覆盖 TeX/DVI、Pack200、InstallShield、Age、PuTTY Key、Sun Raster、HDR、Binary EPS、DfuSe 与 RData；
- 累计 36 个测试套件，**159 个测试用例全部通过 (159/159 PASS)**；
- `cargo clippy --all-targets -- -D warnings` 零警告。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (159/159 PASS)。
- **规范契约**：所有 20 个垂直分卷行数严密收敛于 67~76 行，零单文件臃肿。
- **Git 仓库状态**：代码与交接文档准备就绪，等待原子提交与推送验证。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 执行原子提交并推送到 GitHub 远程仓库；
  2. 监控 GitHub Actions 确认三大平台 (Ubuntu, macOS, Windows) CI 全绿。
