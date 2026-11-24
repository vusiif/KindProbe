# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-24  
> **当前版本**：v0.1.0 (完成阶段 6 编译期静态只读查找表与通用向量化；彻底消灭 OnceLock 与 256 次堆分配，未知二进制全扫失配达 21.47x 加速至 231ns 超 431 万 files/s，PDF 达 340ns，文本全线进入 400~500ns 档位，基准测试套件总耗时从 10.30s 缩减至 1.82s 提速 5.65x，全套 49 套件测试 100% 绿标，Clippy 0 警告)  
> **远端分支**：`origin/main` (准备提交阶段 6 优化至主线)

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 6：编译期只读静态查找表、消灭 OnceLock 与抗过拟合通用语义 (Phase 6)
1. **彻底消除运行时 OnceLock 与堆分配 (`src/detectors/rules/static_index.rs` & `dispatch.rs`)**：
   - 将原先在启动时通过 `OnceLock` 动态构建的 256 桶结构、fallback 规则和 by_format 快速通道表，全部预先生成为只读数据段（`.rodata`）的静态切片；
   - 彻底消除 256 次堆分配与多级指针跳转，完全消除冷启动开销，L1 Cache 亲和性达到理论最大值；
   - `dispatch.rs` 简化为直接在静态切片 `BUCKETS[b0]` 上遍历，无任何锁或状态。
2. **语法特征通用向量化与消除过拟合 (`src/detectors/text/source_code.rs` & `tests/test_source_code_and_scripts.rs`)**：
   - 摒弃脆弱的“行首必须是某字符”的过拟合假定；
   - 利用标准库 AVX2/SSE2 优化的 `memchr` (`w_bytes.contains(&b'#')`、`&b'm'`、`&b'p'` 等）快速过滤不可能包含对应语法标记的代码；
   - 完整覆盖带有多行版权注释、文件说明、类定义、属性修饰符、泛型声明和自由 imports 排布的真实代码，新增跨语言前置注释回归测试。
3. **性能基准全量日志与基线指南更新 (`docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`)**：
   - 完整记录阶段 1~6 演化实测数据对比表。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**188+ 个测试用例全部通过 (188+/188+ PASS)**；
  - `cargo clippy --all-targets --all-features -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **微基准实测性能 (Release Profile - 阶段 6 深度编译期静态化)**：
  - **Unknown Binary (Full Scan Miss)**: **231.95 ns** (21.47x 加速, -95.3%, 431.1 万 files/s)
  - **PDF Document (Fast Window)**: **340.32 ns** (13.58x 加速, -92.6%, 293.8 万 files/s)
  - **CAN Matrix SYM (P44)**: **343.37 ns** (6.57x 加速, -84.8%, 291.2 万 files/s)
  - **Blender 3D (P1)**: **353.51 ns** (8.88x 加速, -88.7%, 282.8 万 files/s)
  - **PNG Image (Hardcoded)**: **353.16 ns** (1.49x 突破, -33.0%, 283.1 万 files/s)
  - **Fast Path Hit PNG**: **362.77 ns** (1.65x 突破, -39.4%, 275.6 万 files/s)
  - **LIDAR LAS (Declarative)**: **370.45 ns** (4.66x 加速, -78.5%, 269.9 万 files/s)
  - **FLV Video (P1)**: **380.62 ns** (7.92x 加速, -87.4%, 262.7 万 files/s)
  - **HDF5 Scientific (P2)**: **384.25 ns** (11.83x 加速, -91.5%, 260.2 万 files/s)
  - **Retro Sharp XDF (P59)**: **399.06 ns** (7.31x 加速, -86.3%, 250.5 万 files/s)
  - **Java KeyStore**: **416.88 ns** (3.63x 加速, -72.4%, 239.8 万 files/s)
  - **Fast Path Hit Sharp XDF**: **441.23 ns** (2.75x 加速, -63.6%, 226.6 万 files/s)
  - **JPEG Image (Hardcoded)**: **442.87 ns** (1.35x 突破, -25.8%, 225.8 万 files/s)
  - **Fast Path Mismatch**: **454.46 ns** (2.84x 加速, -64.8%, 220.0 万 files/s)
  - **Python Shebang (Text)**: **482.02 ns** (10.21x 加速, -90.2%, 207.4 万 files/s)
  - **Verilog HDL Source (Text)**: **543.38 ns** (9.68x 加速, -89.7%, 184.0 万 files/s)
  - **C Source Code (Text)**: **549.50 ns** (9.78x 加速, -89.8%, 181.9 万 files/s)
  - **Extension Binary Search**: **42.81 ns** (2336 万 ops/s)
  - **深度容器 DOCX 解析**: **24.72 μs** (1.55x 加速, 超 4.04 万 docs/s)
  - **深度容器 DOC CFB 导航**: **8.29 μs** (1.35x 加速, 超 12.0 万 docs/s)
  - **基准测试套件执行总耗时**: 从 10.30s 缩减至 **1.82s** (5.65x 整体提速, -82.3%)

---

## 3. [待办与下一步] (Next Steps)

1. 提交并推送阶段 6 优化至 GitHub 远端；
2. 确认 GitHub Actions CI 自动化构建与测试通过；
3. 向用户汇报阶段 6 优化突破（全量格式突破亚微秒，未知流跌破 240ns，吞吐突破 430 万/s）；
4. 筹备下一阶深度容器解析优化（阶段 7：DOCX / CFB 轻量流式标记跳步）。
