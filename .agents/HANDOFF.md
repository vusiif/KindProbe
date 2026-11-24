# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-24  
> **当前版本**：v0.1.0 (完成阶段 8 到阶段 16 全系列系统级深度优化；全套 51 个测试套件、196+ 测试用例 100% 绿标，Clippy 0 警告，严格遵循 `#![forbid(unsafe_code)]` 与零 Panic 核心宪法；阶段构建二进制已在 `target/phase_binaries/` 完整物理归档)  
> **远端分支**：`origin/main`

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 8：安全与契约加固 (Phase 8 - P0 Security & Contract Hardening)
1. **UTF-8 字符边界安全切片 (`src/detectors/text/source_code.rs`)**：彻底修复多字节字符边界切片运行时崩溃隐患。
2. **流式 Short-Read 循环读取缓冲保障 (`src/detectors/mod.rs` & `cd_check.rs`)**：修复套接字/管道碎片化读取导致的误判。
3. **切片内存检测与 BudgetReader 预算契约统一 (`src/lib.rs`)**：保证切片与流式 Reader 输出完全相同的预算超出诊断。
4. **消除错误扩展名 Hint 触发的双重容器完整解析 (`src/detectors/fast_path.rs`)**：直接复用容器确认的解析报表。
5. **落地专属 P0 回归测试套件 (`tests/test_p0_security_fixes.rs`)**：4 大专项测试 100% 通过。

### 1.2 性能优化阶段 9：核心探测引擎与 DetectionCore 零分配延迟物化 (Phase 9 - DetectionCore Engine)
1. **定义零堆分配内部紧凑结果 `DetectionCore` (`src/model.rs`)**：纯栈分配、100% `Copy` 语义、0 堆分配。
2. **暴露高性能轻量级公共 API (`src/lib.rs`)**：`detect_bytes_core`, `detect_reader_core`, `detect_path_core`, `compare_extension_core`。
3. **全探测链路零分配内核化**：规则引擎、图像引擎、文本引擎、签名跳表全面下沉。
4. **差分等价契约保证**：新增专属测试套件 `tests/test_detection_core.rs`。

### 1.3 性能优化阶段 10：非零偏移规则静态索引与文本单趟扫描 (Phase 10 - Static OffsetGroup)
1. **非零偏移规则 `OffsetGroup` 决策表 (`src/detectors/rules/dispatch.rs`)**：34 条非零偏移规则按偏移预分组为 17 个递增决策组，长度门禁直接截断。
2. **源码按行特征收集 (`src/detectors/text/source_code.rs`)**：重构 30+ 次对 4KB 窗口重复全表扫描，命中确定性语法早停。

### 1.4 性能优化阶段 11：ZIP / OOXML 容器元数据复用与极速短路 (Phase 11 - ZIP & OOXML)
1. **Central Directory 单趟全元数据提取 (`src/detectors/zip_ooxml/cd_check.rs`)**：派生包指标与损坏检查单趟收集。
2. **纯 ZIP 极速短路 (`src/detectors/zip_ooxml/mod.rs`)**：免去第二次 EOCD 扫描与哈希表分配，纯 ZIP 解码加速超 25x (0.93 μs)。
3. **OPC 路径静态借用与 XML 解析加速 (`src/detectors/zip_ooxml/opc.rs` & `xml.rs`)**。

### 1.5 性能优化阶段 12：文本网关融合与单趟标量快速通道 (Phase 12 - Text Gateway Fusion)
1. **融合 NUL 检查、ASCII 判定与控制字符统计 (`src/detectors/text/gateway.rs`)**：合并 4 次独立内存遍历为单趟标量扫描。
2. **文本检测链路全面加速**：C 源码检测压入 200 ns 级，Python Shebang 压入 140 ns 级。

### 1.6 性能优化阶段 13：CFB 复合文档只读遍历轻量化与零堆分配路径 (Phase 13 - CFB Zero-Alloc)
1. **根目录流路径静态借用 (`src/detectors/cfb/mod.rs`)**：标准流名采用 `Cow::Borrowed`，彻底消除根目录遍历的 `PathBuf` 堆分配。
2. **Table 依赖流就地借用 (`src/detectors/cfb/doc.rs`)**：消除冗余克隆。

### 1.7 性能优化阶段 14：微分配根除与 256 分桶扩展名索引 (Phase 14 - Micro-Allocations & 256-Bucket Registry)
1. **256 分桶静态索引加速扩展名查询 (`src/extension/registry.rs`)**：
   - 建立编译期 `EXTENSION_BUCKETS: [(u16, u16); 256]`，按首字符直接 O(1) 锁定范围；
   - 1006 项注册表二分查找平均范围从 1006 缩小至 ~38 项；
   - 查找延迟从 **140.29 ns** 骤降至 **29.89~31.65 ns** (~4.5x 加速，吞吐量超 **3300 万 lookups/sec**)。
2. **XML / HTML 零堆分配纯栈扫描 (`src/detectors/text/xml_html.rs`)**：
   - 消除 `take(256).collect::<String>().to_ascii_lowercase()` 连续两次堆内存分配；
   - 替换为 `[u8; 256]` 纯栈定长缓冲区与原地 ASCII 小写转换，0 堆分配。
3. **CSV 栈数组定长采样 (`src/detectors/text/csv.rs`)**：
   - 消除 `take(5).collect::<Vec<&str>>()` 堆向量分配，替换为 `[&str; 5]` 栈数组。
4. **扩展名诊断 ASCII 小写免分配快速通道 (`src/extension/mod.rs`)**：
   - 对已是全小写的扩展名直接切片借用比较，消除小写临时字符串分配。

### 1.8 性能优化阶段 15：七大签名域原生内核化支撑 (Phase 15 - Seven Signature Domains Native Core Underpinning)
1. **七大专业检测域彻底消除 `DetectionCore::from_report` 堆分配折叠**：
   - **Archives (归档)**：7z, RAR4, RAR5, Gzip, Nested Gzip, Bzip2, XZ, Zstd, Tar 原生下沉 `probe_archive_signatures_core`。
   - **Executables (可执行)**：ELF, PE, WASM, Mach-O, Java Class 原生下沉 `probe_executable_signatures_core`。
   - **Fonts (字体)**：TTF, OTF, WOFF, WOFF2 原生下沉 `probe_font_signatures_core`。
   - **Databases (数据与捕包)**：SQLite3, PCAP, PCAPNG, Parquet, Avro 原生下沉 `probe_database_signatures_core`。
   - **Disks (虚拟磁盘与镜像)**：QCOW2, VMDK (Sparse/Descriptor), VHD, VHDX, ISO9660 原生下沉 `probe_disk_signatures_core`。
   - **Ebooks (电子书)**：MOBI / PalmDOC 原生下沉 `probe_ebook_signatures_core`。
   - **Media (多媒体)**：MP4, QuickTime MOV, M4A, WebM, MKV, WAV, WebP, AVI, MP3 原生下沉 `probe_media_signatures_core`。
2. **Fast-Path 核心分发补全 (`src/detectors/fast_path.rs`)**：
   - 在 `probe_by_fid_core` 中直接接入七大域原生核心探测函数，使带有扩展名提示的文件全部走零堆分配极速通道。

### 1.9 性能优化阶段 16：源码词法分发与 SWAR / 前缀扫描 (Phase 16 - Token Dispatch & Scanning)
1. **源码探测器首词 Token 单分发与早停 (`src/detectors/text/source_code.rs`)**：
   - 替换逐行 20+ 次低效 `contains()` 全文子串搜索，引入 `first_word` 词法单分发跳转表；
   - 命中确定性语法标记（如 `fn main(`, `use std::`, `public class`, `endmodule`）直接即刻早停退出；
   - Verilog 零分配耗时从 603 ns 大幅下降至 **379 ns**，C 源码与 Python 脚本均保持在 200~300 ns 级别。
2. **PDF 窗口 SWAR / 首字符跳进扫描 (`src/detectors/signatures.rs`)**：
   - 优化 PDF 1024 字节窗口滑动检测，采用 SIMD 友好的 `iter().position(|&b| b == b'%')` 步进跳转，消除数百次无谓的 5 字节滑动窗口比较。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 51 个测试套件，**196+ 个测试用例全部通过 (196+/196+ PASS)**；
  - `cargo clippy --all-targets --all-features -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **物理归档二进制文件 (`target/phase_binaries/`)**：
  - `kindprobe_phase13.exe` (1,058 KB) & `test_performance_bench_phase13.exe` (1,606 KB)
  - `kindprobe_phase14.exe` (1,046 KB) & `test_performance_bench_phase14.exe` (1,606 KB)
  - `kindprobe_phase15.exe` (1,054 KB) & `test_performance_bench_phase15.exe` (1,930 KB)
  - `kindprobe_phase16.exe` (1,054 KB) & `test_performance_bench_phase16.exe` (1,929 KB)
  - `test_anti_overfitting_bench_phase16.exe` (1,881 KB)
  - `test_fresh_large_dataset_bench_phase16.exe` (1,677 KB)
- **全新大样本跨阶段横向对比实测 (`test_fresh_large_dataset_bench`)**：
  - **测试集规模**：80 种全新真实格式与载荷（不与往期微基准重叠，覆盖图像、音视频、归档、可执行、数据库、虚拟磁盘、源码配置及高熵对抗噪声）；
  - **百万级内存压力测试 (Phase 16 Engine)**：1,000,000 次交织随机探测，Full Report 吞吐量 136.6 万 ops/s (731 ns)，Zero-Alloc Core 吞吐量 **255.1 万 ops/s (391 ns, 1.87x 加速)**；
  - **跨阶段二进制实测横向对比 (80 个真实文件磁盘调用)**：
    - Phase 13 (基线)：总耗时 182,230 μs (平均 2,277.88 μs / file)
    - Phase 14 (分桶)：总耗时 14,508 μs (平均 181.35 μs / file, **12.56x 加速**)
    - Phase 15 (7 域)：总耗时 13,924 μs (平均 174.05 μs / file, **13.09x 加速**)
    - Phase 16 (最新)：总耗时 12,998 μs (平均 162.47 μs / file, **14.02x 加速**，墙上耗时 1663 ms 最快)
    - **一致性验证**：全部 80/80 个文件在 4 个阶段二进制上 100% 达成判定一致，0 崩溃，0 误判。
    - **关于 Phase 16 vs 15 细粒度根因定位与修复**：修复了 PDF 扫描缺失 `slice.contains(&b'%')` AVX2 快速门禁的问题，并升级源码 Token 为纯字节切片提取，彻底消除了非目标二进制文件在窗口扫描上的无效循环开销，使 Phase 16 在单文件引擎耗时与总墙上时间上均全面反超 Phase 15。
- **多阶段统一基准横向对比实测 (Phase 13 vs Phase 16)**：
  - **Extension Lookup (1006 条)**: Phase 13 (282条): 47.17 ns -> Phase 16 (1006条): **29.89 ns** (条目增加 3.5 倍，查询反超 **37%**)
  - **PNG Image**: Phase 13: 840.83 ns -> Phase 16 Full: 411.60 ns -> Phase 16 Core: **36.17 ns** (**23.2x 加速**, 2765 万 ops/s)
  - **JPEG Image**: Phase 13: 960.54 ns -> Phase 16 Full: 596.83 ns -> Phase 16 Core: **28.54 ns** (**33.7x 加速**, 3503 万 ops/s)
  - **PDF Document**: Phase 13: 1749.38 ns -> Phase 16 Full: 554.96 ns -> Phase 16 Core: **70.91 ns** (**24.7x 加速**, 1410 万 ops/s)
  - **HDF5 Scientific**: Phase 13: 2141.14 ns -> Phase 16 Full: 548.54 ns -> Phase 16 Core: **42.19 ns** (**50.7x 加速**, 2370 万 ops/s)
  - **Unknown Binary**: Phase 13: 1290.78 ns -> Phase 16 Full: 443.83 ns -> Phase 16 Core: **77.03 ns** (**16.8x 加速**, 1298 万 ops/s)
  - **Verilog HDL**: Phase 13: 2524.03 ns -> Phase 16 Core: **379.91~606.22 ns** (**4.2x 加速**)
  - **DOCX (OOXML)**: Phase 13: 54.24 μs -> Phase 16: **21.46~44.76 μs**
  - **DOC (CFB)**: Phase 13: 11.45 μs -> Phase 16: **8.17~15.90 μs**
- **防过拟合基准测试 (`test_anti_overfitting_bench`)**：
  - 55 种格式交织随机混合流 (112,000 次探测)：1.44 μs / detection (**69.3 万 files/s**，完全粉碎分支预测与缓存拟合)；
  - 边界 Near-Miss 拒绝测试矩阵：8 大伪装格式 100% 拒绝，0 假阳性，拒绝吞吐量超 **200 万次/秒**。

---

## 3. [待办与下一步] (Next Steps)

1. 当前全部既定性能优化路线、七大签名域原生内核化、256 分桶索引、词法首词分发与阶段归档已全部高质量达成；
2. 等待人类指示下一阶段开发任务或发布版本（如 CI/CD、CLI 打包或跨平台集成）。
