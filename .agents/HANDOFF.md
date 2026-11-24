# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-24  
> **当前版本**：v0.1.0 (完成阶段 4 内存切片零拷贝探测、256 桶次字节寄存器过滤与 Fast Path 零分配；未知二进制/失配达 12.62x 加速至 394ns、FLV 达 8.76x 加速至 344ns 超 290 万 files/s、主流格式全线进入 300~400ns 区间，基准套件整体耗时由 10.30s 缩减至 2.40s，全套 49 套件测试 100% 绿标，Clippy 0 警告)  
> **远端分支**：`origin/main` (准备提交阶段 4 优化至主线)

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 4：零拷贝内存切片直探、256 桶次字节寄存器过滤与 Fast Path 零分配 (Phase 4)
1. **内存切片零包装探测 (`detect_slice_non_container` in `src/detectors/mod.rs` & `src/lib.rs`)**：
   - 在 `detect_bytes(bytes, options)` 中直接借用 `&bytes[..n]` 进行非容器探测；
   - 彻底省去了 `Cursor` 包装、`BudgetReader` 原子计数器堆栈初始化、4KB 栈内存拷贝（`read` memcpy）以及无效的 `seek(0)` 寻道开销；
   - 仅当样本前缀确认为深层流式容器（ZIP `PK`、CFB `\xD0\xCF` 或 ISO 卷描述符）时，才回退至带 seek 的 `detect_reader`，完全不破坏复合文档解析。
2. **256 桶内次字节单指令过滤 (`src/detectors/rules/dispatch.rs`)**：
   - 在 `evaluate_indexed_rules` 桶内迭代中，利用 offset 0 且无掩码模式的特征，通过单条 CPU 比较指令瞬时淘汰 **99.6%** 的桶内失配规则，消除多规则桶遍历开销。
3. **Fast Path `Cow` 零分配与直通派发 (`src/detectors/fast_path.rs`)**：
   - 对扩展名提示使用 `Cow<'_, str>` 避免常规小写扩展名的 `String` 堆分配；
   - 对 PNG、JPEG、GIF 直接调用专用单探函数，避免通过多层组函数的层级判断。
4. **性能基准全量日志与基线指南更新 (`docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`)**：
   - 完整记录阶段 1~4 演化实测数据对比表。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**187+ 个测试用例全部通过 (187+/187+ PASS)**；
  - `cargo clippy --all-targets --all-features -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **微基准实测性能 (Release Profile - 阶段 4 深度优化)**：
  - **Unknown Binary (Full Scan Miss)**: **394.54 ns** (12.62x 加速, -92.1%, 253.5 万 files/s)
  - **HDF5 Scientific (P2)**: **468.72 ns** (9.70x 加速, -89.7%, 213.3 万 files/s)
  - **FLV Video (P1)**: **344.24 ns** (8.76x 加速, -88.6%, 290.5 万 files/s)
  - **PDF Document**: **532.86 ns** (8.67x 加速, -88.5%, 187.7 万 files/s)
  - **Python Shebang (Text)**: **616.29 ns** (7.98x 加速, -87.5%, 162.3 万 files/s)
  - **Blender 3D (P1)**: **437.88 ns** (7.17x 加速, -86.0%, 228.4 万 files/s)
  - **Retro Sharp XDF (P59)**: **407.77 ns** (7.16x 加速, -86.0%, 245.2 万 files/s)
  - **Verilog HDL Source (Text)**: **789.27 ns** (6.66x 加速, -85.0%, 126.7 万 files/s)
  - **C Source Code (Text)**: **889.43 ns** (6.04x 加速, -83.5%, 112.4 万 files/s)
  - **PNG Image (Hardcoded)**: **418.72 ns** (1.26x 突破, -20.5%, 238.8 万 files/s)
  - **JPEG Image (Hardcoded)**: **483.13 ns** (1.24x 突破, -19.1%, 207.0 万 files/s)
  - **Fast Path Hit PNG**: **419.52 ns** (1.43x 突破, -29.9%, 238.4 万 files/s)
  - **基准测试套件执行总耗时**: 从 10.30s 缩减至 **2.40s** (4.29x 整体耗时大幅缩减, -76.7%)

---

## 3. [待办与下一步] (Next Steps)

1. 提交并推送阶段 4 优化至 GitHub 远端；
2. 确认 GitHub Actions CI 自动化构建与测试通过；
3. 向用户汇报阶段 4 优化结果，并深入分析探讨后续更极致的微架构优化空间（例如 SIMD 向量化文本扫描、多字节 u32/u64 字对齐匹配、以及零解析静态常量优化等）。
