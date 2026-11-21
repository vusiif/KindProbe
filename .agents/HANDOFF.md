# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (8大分卷生成架构演进，全量突破 120+ 格式与 200+ 扩展名)  
> **远端分支**：`origin/main` (当前提交待推送)

---

## 1. [已完成] (Completed)

### 1.1 8大领域声明式分卷架构 (`src/detectors/rules/generated/`)
彻底解决规则库规模化后的代码膨胀问题，所有文件**严格控制在 28~70 行区间**：
- `media.rs` (56 行)：标准音视频（`flv`, `amr`, `ac3`, `au`, `caf`, `drc`, `mng`）
- `audio_chips.rs` (35 行)：芯片音乐与种子（`sid`, `spc`, `vgm`, `torrent`）
- `images.rs` (70 行)：专业图像（`xcf`, `exr`, `fits`, `icns`, `jp2`, `bpg`, `djvu`, `nitf`, `mdi`）
- `netpbm.rs` (28 行)：Netpbm 系列（`pbm`, `pgm`, `ppm`）
- `documents.rs` (63 行)：技术与绘图文档（`dicom`, `postscript`, `rtf`, `dwg`, `dxf_bin`, `dwf`, `wri`, `xar`）
- `archives.rs` (63 行)：高级系统归档（`cpio`, `lz`, `arj`, `zoo`, `cab`, `wim`, `sit`, `sqx`）
- `models.rs` (56 行)：3D 几何模型与 CAD（`blend`, `glb`, `fbx`, `3ds`, `ply`, `step`, `nes`）
- `scientific.rs` (63 行)：科研工程与地理大科学（`hdf5`, `hdf4`, `netcdf`, `shp`, `grib`, `mat5`, `root`, `mrc`）
- `mod.rs` (38 行)：集中汇聚 `GENERATED_RULES` 切片，保持零运行时堆分配（`&'static [SignatureRule]`）。

### 1.2 扩展名单一真实信源 (SSOT) 索引库 (突破 203 项)
- `tools/codegen.py` 自动合并提取新老格式扩展名，按 ASCII 严格单调递增排列于 `src/extension/registry.rs`（收录 **203 项标准扩展名**），提供零哈希分配的 $O(\log N)$ 二分查找。

### 1.3 真实世界样本库与对抗性模糊测试
- `fixtures/real/` 落地 25 组真实世界合规文件切片，端到端测试覆盖真实 OS 句柄探测。
- 确定性 PRNG 变异引擎覆盖 3,000+ 随机破坏流、魔数截断模糊与单比特翻转，守护零崩溃与读取预算锁定不变量。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets` 0 Error。
- **集成测试基线**：全量 `cargo test` **121 / 121 全部通过 (100% PASS)**（覆盖 23 个集成测试套件，耗时 ~1.3 秒）。
- **Git 仓库状态**：准备提交并推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无 (None)。
- **后续优化建议**：
  1. 搭建 Criterion 纳秒级性能基准压测（`benches/`）；
  2. 针对 CLI 扩充批量/目录扫描与流式 JSON Lines 输出。
