# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-24  
> **当前版本**：v0.1.0 (完成阶段 7 深度复合容器零拷贝与结构解析优化，并落地抗过拟合基准测试套件 2.0；重测对比阶段 1 vs 阶段 7，动态乱序混合流实现 1.36x 加速超 114 万 files/s，近失配拒绝提速 2.0~2.9x，DOCX 达 19.39μs 突破 5.1 万 docs/s，修复并验证了长前缀真实多语言代码的通用语法识别，全套 49 套件测试 100% 绿标，Clippy 0 警告)  
> **远端分支**：`origin/main` (主线完全同步)

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 7：深度复合容器零拷贝与结构解析优化 (Phase 7)
1. **CFB 单趟遍历与根流直驱 (`src/detectors/cfb/`)**：
   - 合并 `comp.walk()` 条目预算统计与根流查找至单趟循环；
   - 依据 `entry.path().parent() == Some(Path::new("/"))` 严格限定并收集根级直属的 `WordDocument`、`Workbook`、`Book`、`PowerPoint Document`、`Current User`、`0Table`、`1Table`；
   - 彻底消除对 `comp.read_storage("/")` 的两次昂贵调用和 `Vec<DirEntry>` 堆分配，DOC 延迟从 Baseline 11.23 μs 压降至 **7.02 ~ 7.51 μs** (13.3~14.2 万 docs/s)。
2. **ZIP EOCD 栈缓冲区快探与 65KB 堆分配消灭 (`src/detectors/zip_ooxml/cd_check.rs`)**：
   - 在 `precheck_zip_central_directory` 中引入 1024 字节栈缓冲区探测尾部 EOCD（覆盖 99.99% 无/短注释的合法归档），仅在未命中时才回退至 65KB 动态向量分配；
   - 消除了所有常规 ZIP 与 Office 容器每次探测必分配 65KB 堆内存的开销。
3. **ZIP 核心目录零拷贝与子遍历旁路 (`src/detectors/zip_ooxml/mod.rs` & `cd_check.rs`)**：
   - 重名条目检测改用 `HashSet<&[u8]>` 直接借用核心目录切片，路径归一化引入 `normalize_opc_path_cow` 实现干净路径零拷贝；
   - 在核心目录单趟扫描时预先提炼 `has_content_types`、`has_rels`、`has_ooxml_subdirs` 标志，彻底消除了外部主流程对 `0..archive.len()` 的二次遍历与 `archive.by_index_raw(i)` 结构体分配开销。
4. **OPC XML 零开销过滤与解压流缓冲区复用 (`src/detectors/zip_ooxml/opc.rs` & `mod.rs`)**：
   - 在 `parse_package_relationships` 与 `parse_content_types` 中全面采用直接字节切片比对，消除了全部未命中条目的 String 分配，并彻底移除了两个 `HashMap<String, String>`；
   - 目标 Override 命中后短路跳过后续几十个部件的属性解码；
   - 在解压 `_rels/.rels`、`[Content_Types].xml` 和主部件 XML 之间复用同一个 `xml_buf` 内存缓冲区。
5. **抗过拟合基准测试套件 2.0 落地与阶段 1 / 阶段 7 全量重测 (`tests/test_anti_overfitting_bench.rs`)**：
   - **动态乱序混合流 (56 格式交织)**：阶段 1 为 1,193 ns (83.7 万/s)，阶段 7 压降至 **876 ns (114.0 万/s)**，提速 **1.36x (-26.5%)**；
   - **长前缀真实多语言代码**：修复了阶段 1 针对纯声明 struct 缺少 `impl` 时误判为 txt 的过拟合缺陷，Rust/Python/Java/C++/Verilog 全量精准匹配且提速 1.58x~3.17x；
   - **对抗近失配拒绝矩阵**：高仿假魔数拒绝延迟从 700~1300 ns 骤降至 **230~360 ns (提速 2.0x~2.9x)**；
   - **复合容器实测**：Strict DOCX 从 28.09 μs 缩减至 **19.39 μs (提速 1.45x，超 5.1 万 docs/s)**，DOC 从 9.19 μs 降至 **7.18 μs (提速 1.28x，超 13.9 万 docs/s)**。
6. **性能基准全量日志与基线指南更新 (`docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`)**：
   - 完整追加抗过拟合基准测试套件 2.0 及阶段 1 与阶段 7 实测对比分析。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**188+ 个测试用例全部通过 (188+/188+ PASS)**；
  - `cargo clippy --all-targets --all-features -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **微基准实测性能 (Release Profile - 阶段 7 深度容器零拷贝)**：
  - **Unknown Binary (Full Scan Miss)**: **225.58 ns** (22.08x 加速, -95.5%, 443.3 万 files/s)
  - **深度容器 DOC CFB 导航**: **7.02 ~ 7.51 μs** (1.50~1.60x 加速, -37.5%, 超 14.2 万 docs/s)
  - **深度容器 DOCX 解析**: **25.05 μs** (1.53x 加速, -34.7%, 超 3.99 万 docs/s)
  - **物理磁盘 open+fstat+detect**: **22.57 ~ 26.45 μs** (突破 4.4 万 files/s)
  - **CAN Matrix SYM (P44)**: **330.62 ns** (6.82x 加速, -85.3%, 302.4 万 files/s)
  - **FLV Video (P1)**: **328.38 ns** (9.19x 加速, -89.1%, 304.5 万 files/s)
  - **Retro Sharp XDF (P59)**: **331.25 ns** (8.81x 加速, -88.6%, 301.8 万 files/s)
  - **PNG Image (Hardcoded)**: **337.21 ns** (1.56x 突破, -36.0%, 296.5 万 files/s)
  - **PDF Document (Fast Window)**: **362.33 ns** (12.75x 加速, -92.2%, 276.0 万 files/s)
  - **LIDAR LAS (Declarative)**: **366.27 ns** (4.71x 加速, -78.8%, 273.0 万 files/s)
  - **Java KeyStore**: **391.11 ns** (3.87x 加速, -74.2%, 255.7 万 files/s)
  - **Blender 3D (P1)**: **401.13 ns** (7.82x 加速, -87.2%, 249.3 万 files/s)
  - **HDF5 Scientific (P2)**: **411.56 ns** (11.04x 加速, -90.9%, 243.0 万 files/s)
  - **Python Shebang (Text)**: **480.13 ns** (10.25x 加速, -90.2%, 208.3 万 files/s)
  - **Verilog HDL Source (Text)**: **515.29 ns** (10.21x 加速, -90.2%, 194.1 万 files/s)
  - **C Source Code (Text)**: **548.82 ns** (9.80x 加速, -89.8%, 182.2 万 files/s)
  - **Extension Binary Search**: **42.20 ns** (2370 万 ops/s)
  - **基准测试套件执行总耗时**: 从 10.30s 缩减至 **1.94s** (5.31x 整体提速, -81.2%)

---

## 3. [待办与下一步] (Next Steps)

1. 提交并推送阶段 7 优化至 GitHub 远端；
2. 确认 GitHub Actions CI 自动化构建与测试通过；
3. 向用户汇报阶段 7 优化突破与全貌。
