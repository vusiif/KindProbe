# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (14大垂直领域分卷定型，扩展名索引达 282 项，全量 129 测试全绿)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 14 大垂直领域声明式规则分卷 (`src/detectors/rules/generated/`)
彻底解决规则库规模化后的代码膨胀问题，所有 14 个分卷与聚合模块**严格控制在 34~70 行黄金区间**：
- `media.rs` (69 行)：标准与专业音频（`flv`, `amr`, `ac3`, `au`, `caf`, `drc`, `mng`, `ape`, `wv`）
- `audio_chips.rs` (62 行)：芯片音乐与 Tracker 模块（`sid`, `spc`, `vgm`, `torrent`, `s3m`, `xm`, `it`, `mod`）
- `images.rs` (69 行)：专业图像与出版格式（`xcf`, `exr`, `fits`, `icns`, `jp2`, `bpg`, `djvu`, `nitf`, `mdi`）
- `netpbm.rs` (55 行)：Netpbm 族谱与图形扩展（`pbm`, `pgm`, `ppm`, `pam`, `xbm`, `xpm`, `sgi`）
- `documents.rs` (62 行)：技术与绘图文档（`dicom`, `postscript`, `rtf`, `dwg`, `dxf_bin`, `dwf`, `wri`, `xar`）
- `archives.rs` (62 行)：传统系统归档（`cpio`, `lz`, `arj`, `zoo`, `cab`, `wim`, `sit`, `sqx`）
- `models.rs` (55 行)：3D 几何模型与 CAD（`blend`, `glb`, `fbx`, `3ds`, `ply`, `step`, `lwo`）
- `scientific.rs` (62 行)：科研工程与地理大科学（`hdf5`, `hdf4`, `netcdf`, `shp`, `grib`, `mat5`, `root`, `mrc`）
- `roms.rs` (55 行)：经典游戏主机 ROM 与卡带镜像（`nes`, `z64`, `n64`, `v64`, `gba`, `gb`/`gbc`, `nds`）
- `packages.rs` (41 行)：压缩包与系统打包格式（`snappy`/`sz`, `z`, `lha`/`lzh`, `ace`, 新增 `zpaq`）
- `database.rs` (41 行)：现代数据库与科学点云（`arrow`/`feather`, `kdbx`, `kdb`, `realm`, 新增 `las`/`laz` 激光雷达点云）
- `fonts.rs` (41 行，新分卷)：专业排版与矢量点阵字体（`pcf`, `bdf`, `afm`, `pfb`, `eot`）
- `crypto.rs` (34 行，新分卷)：公钥基础架构与安全证书（`jks` Java KeyStore, `pem`, `der`, `pgp`/`gpg`）
- `disk_images.rs` (34 行，新分卷)：虚拟化与光盘镜像（`vdi` VirtualBox, `chd` MAME, `cso`, `nrg` Nero）
- `mod.rs` (50 行)：全动态聚合 14 大分卷为 `GENERATED_RULES` 切片，保持零运行时堆分配（`&'static [&'static [SignatureRule]]`）。

### 1.2 扩展名二分查找索引表突破 282 项 (`src/extension/registry.rs`)
- 扩展名总数由 258 项跃升至 **282 项**；
- 严格按 ASCII 升序排列，并内建单调递增单元测试守卫；
- 二分查找在 Release 模式下耗时仅需 **36.81 ns / op**，吞吐量高达 **2717 万次 / 秒**。

### 1.3 垂直领域集成测试与微基准压测
- 新增 `tests/test_fonts_crypto_disks.rs` (79 行)：针对字体排版、证书密钥、虚拟光盘的端到端探测校验；
- 全量 27 个集成测试套件、单元测试全部 100% 通过（**129 / 129 测试全部通过**）。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (129/129 PASS)。
- **Git 仓库状态**：已暂存，准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 提交并推送至 `origin/main`，监控 GitHub Actions 确保三大平台全绿；
  2. 向用户汇报 14 大分卷体系与 282 项扩展名扩充成果。
