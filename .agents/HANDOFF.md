# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-24  
> **当前版本**：v0.1.0 (完成阶段 5 文本引擎字节级门禁、代码语法前缀路由与寄存器字比对；未知二进制/失配达 16.33x 加速至 304ns、PDF 达 12.83x 加速至 360ns、HDF5 达 11.72x 加速至 387ns、C/Verilog/Python 文本全线降至 600ns 档位，DOCX 达 26μs，DOC 突破 8.26μs 超 12 万 docs/s，基准套件整体耗时缩减至 2.13s，全套 49 套件测试 100% 绿标，Clippy 0 警告)  
> **远端分支**：`origin/main` (准备提交阶段 5 优化至主线)

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 5：文本引擎字节级门禁、语法前缀路由与寄存器字比对 (Phase 5)
1. **文本网关极速退出与 ASCII 字节直接统计 (`examine_text_gateway` in `src/detectors/text/gateway.rs`)**：
   - 增加单指令首字节判定：若首字节不是空白符、打印 ASCII 或合法 UTF-8 前缀，瞬时退出；
   - 对于占绝大多数的 ASCII 文本，直接在字节层做统计并提前终止超过 1% 控制字符的流，彻底消除 UTF-8 标量值逐字符解码开销。
2. **代码语法前缀导向路由 (`probe_source_code` in `src/detectors/text/source_code.rs`)**：
   - 根据行首字符（`#` 专属于 Shebang / C include / Markdown，`m` 专属于 Verilog，`d` 专属于 diff，`p` 专属于 package 等）进行针对性快速分支；
   - 避免普通纯文本或无关源文件反复进行 10 余次全文子串线性扫描。
3. **单指令字对齐寄存器匹配与多字节过滤 (`src/detectors/rules/`)**：
   - 在 `engine.rs` 中为 2、4、8 字节无掩码规则引入 `u32::from_ne_bytes` 和 `u64::from_ne_bytes` 寄存器单指令比对；
   - 在 `dispatch.rs` 中为 4 字节前缀规则提供桶内前缀整数比对，并为 fallback 规则增加偏移首字节失配过滤。
4. **性能基准全量日志与基线指南更新 (`docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`)**：
   - 完整记录阶段 1~5 演化实测数据对比表。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**187+ 个测试用例全部通过 (187+/187+ PASS)**；
  - `cargo clippy --all-targets --all-features -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **微基准实测性能 (Release Profile - 阶段 5 极速调优)**：
  - **Unknown Binary (Full Scan Miss)**: **304.93 ns** (16.33x 加速, -93.9%, 327.9 万 files/s)
  - **PDF Document**: **360.09 ns** (12.83x 加速, -92.2%, 277.7 万 files/s)
  - **FLV Video (P1)**: **361.88 ns** (8.34x 加速, -88.0%, 276.3 万 files/s)
  - **Blender 3D (P1)**: **380.07 ns** (8.26x 加速, -87.9%, 263.1 万 files/s)
  - **HDF5 Scientific (P2)**: **387.69 ns** (11.72x 加速, -91.5%, 257.9 万 files/s)
  - **PNG Image (Hardcoded)**: **402.75 ns** (1.31x 突破, -23.6%, 248.2 万 files/s)
  - **JPEG Image (Hardcoded)**: **444.51 ns** (1.34x 突破, -25.6%, 224.9 万 files/s)
  - **Fast Path Mismatch**: **466.11 ns** (2.77x 加速, -63.9%, 214.5 万 files/s)
  - **Python Shebang (Text)**: **659.39 ns** (7.46x 加速, -86.6%, 151.6 万 files/s)
  - **C Source Code (Text)**: **667.25 ns** (8.06x 加速, -87.6%, 149.8 万 files/s)
  - **Verilog HDL Source (Text)**: **692.42 ns** (7.60x 加速, -86.8%, 144.4 万 files/s)
  - **深度容器 DOCX 解析**: **26.82 μs** (1.43x 加速, 3.73 万 docs/s)
  - **深度容器 DOC CFB 导航**: **8.26 μs** (1.36x 加速, 12.1 万 docs/s)
  - **基准测试套件执行总耗时**: 从 10.30s 缩减至 **2.13s** (4.84x 整体耗时大幅缩减, -79.3%)

---

## 3. [待办与下一步] (Next Steps)

1. 提交并推送阶段 5 优化至 GitHub 远端；
2. 确认 GitHub Actions CI 自动化构建与测试通过；
3. 向用户汇报阶段 5 优化突破，并按从易到难路线筹备下一阶（阶段 6：构建期静态不可变桶消除 OnceLock，阶段 7：深层复合容器流式解析）。
