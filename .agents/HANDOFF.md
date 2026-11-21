# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (路线 B 阶段性跃升：11 大分卷架构成型，扩展名索引达 234 项)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 路线 B：11 大垂直领域声明式规则分卷 (`src/detectors/rules/generated/`)
彻底解决规则库规模化后的代码膨胀问题，所有 11 个代码文件与 1 个汇聚模块**严格控制在 34~69 行黄金轻量区间**：
- `media.rs` (69 行)：音视频与无损音频（新增 `ape` Monkey's Audio、`wv` WavPack）
- `audio_chips.rs` (62 行)：芯片音乐与 Tracker 模块（新增 `s3m`, `xm`, `it`, `mod` Tracker 族谱）
- `images.rs` (69 行)：专业图像与出版格式（`xcf`, `exr`, `fits`, `icns`, `jp2`, `bpg`, `djvu`, `nitf`, `mdi`）
- `netpbm.rs` (55 行)：Netpbm 族谱与图形扩展（新增 `pam` P7、`xbm`, `xpm`, `sgi`/`rgb`/`bw`）
- `documents.rs` (62 行)：技术与绘图文档（`dicom`, `postscript`, `rtf`, `dwg`, `dxf_bin`, `dwf`, `wri`, `xar`）
- `archives.rs` (62 行)：传统系统归档（`cpio`, `lz`, `arj`, `zoo`, `cab`, `wim`, `sit`, `sqx`）
- `models.rs` (55 行)：3D 几何模型与 CAD（`blend`, `glb`, `fbx`, `3ds`, `ply`, `step`，新增 `lwo` LightWave）
- `scientific.rs` (62 行)：科研工程与地理大科学（`hdf5`, `hdf4`, `netcdf`, `shp`, `grib`, `mat5`, `root`, `mrc`）
- `roms.rs` (55 行，新分卷)：经典游戏主机 ROM 与卡带镜像（`nes`, `z64`, `n64`, `v64`, `gba`, `gb`/`gbc`, `nds`）
- `packages.rs` (34 行，新分卷)：压缩包与系统打包格式（`snappy`/`sz`, `z`, `lha`/`lzh`, `ace`）
- `database.rs` (34 行，新分卷)：现代数据库与列式交换格式（`arrow`/`feather`, `kdbx`, `kdb`, `realm`）
- `mod.rs` (41 行)：动态全自动聚合 11 个分卷为 `GENERATED_RULES` 切片，保持零运行时堆分配（`&'static [&'static [SignatureRule]]`）。

### 1.2 扩展名二分查找索引表突破 234 项 (`src/extension/registry.rs`)
- `tools/codegen.py` 自动合并提取全部 11 大垂直领域规则的扩展名；
- 严格按 ASCII 升序排列，并内建单调递增单元测试守卫；
- 提供零堆分配、零哈希开销的 $O(\log N)$ 二分查找。

### 1.3 垂直领域集成测试套件覆盖
- `tests/test_roms_and_trackers.rs` (79 行)：针对游戏 ROM 与 Tracker 音频的端到端探测校验；
- `tests/test_database_and_packages.rs` (69 行)：针对数据库、安全密码库与系统包格式的探测校验；
- 全量 25 个集成测试套件、单元测试全部 100% 通过（**125 / 125 测试全部通过**）。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets -- -D warnings` 严格零警告 (0 Errors, 0 Warnings)。
- **测试覆盖率与基线**：全量 `cargo test --all-targets` 全部通过 (125/125 PASS)。
- **Git 仓库状态**：代码已完善，准备提交并推送验证 CI。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无。
- **下一步动作**：
  1. 提交并推送至 `origin/main`，验证 GitHub Actions 全绿；
  2. 根据用户需要，进一步按垂直领域扩充（如地理 GIS、科学遥感或更多办公容器）。
