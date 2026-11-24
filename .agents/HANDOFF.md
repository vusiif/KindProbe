# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (完成阶段 1 首字节 256 桶调度与阶段 2 栈缓存0堆分配+文本门禁+PDF快路+ISO剪枝；PDF 实现 6.86 倍加速、文本提速 5.12 倍、PNG 突破 500ns 达 204 万 files/s，全套测试 100% 绿标，Clippy 0 警告)  
> **远端分支**：`origin/main` (前序阶段 1 提交 `99242bb` 已在主线)

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 2：栈缓存 0 堆分配、首字符文本门禁与特征快路 (Zero-Alloc Stack Header & Text Gating)
1. **栈缓存 0 堆分配 ([`src/detectors/mod.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/mod.rs))**：
   - 将每次检测固定调用的 `vec![0u8; 4096]` 堆分配重构为本地 4KB 栈上固定缓冲（`[0u8; 4096]`），仅当配置大于 4KB 时才回退至堆；
   - 彻底消除了每次探测的 `malloc` / `free`，极轻格式（PNG/JPEG）单次耗时直接突破 500ns（PNG 489ns，吞吐量超越 204 万文件/秒）。
2. **首字符快速文本门禁 ([`src/detectors/text/mod.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/text/mod.rs))**：
   - 依据首个非空白字符（`{`, `[`, `<`, `#`, `-`, 字母）进行精准模式分流；
   - 杜绝纯文本/源码空转尝试 JSON/XML/HTML/CSV 等不相关解析器（避免了 CSV 行分割带来的隐式堆分配）；
   - C 源码延迟降至 1,049 ns（5.12x 加速），Verilog 降至 1,096 ns（4.80x 加速），Python 降至 1,165 ns（4.22x 加速）。
3. **PDF 极速快路与 ISO 边界剪枝**：
   - PDF 优先比对 `%PDF-` 前缀，仅在含 `%` 字符时才滑动窗口，单次耗时从 4,621 ns 骤降至 673 ns（**6.86x 加速**）；
   - ISO 9660 在流大小小于 4KB 初始窗口时物理剪枝，彻底省去无意义的 `reader.seek(32768)`。
4. **多 rule 同 format_id 快速通道二分修复 ([`src/detectors/rules/dispatch.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/dispatch.rs))**：
   - 消除 `dedup_by_key` 剪除备选变体的问题，通过区间扫描完整覆盖多规则变体（如 TIFF LE/BE、GBA）；
   - 对回退规则按 offset 排序，探测时达到文件长度边界即时 `break` 剪枝。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**187+ 个测试用例全部通过 (187+/187+ PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **工作区未提交改动**：
  - `src/detectors/mod.rs`（栈缓存 4KB 0 堆分配 + ISO 边界剪枝）；
  - `src/detectors/signatures.rs`（PDF 极速前缀快路）；
  - `src/detectors/text/mod.rs`（首字符快速文本门禁）；
  - `src/detectors/rules/dispatch.rs`（回退规则 offset 排序剪枝 + 多变体二分修复）；
  - `docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`（三期对比数据全景更新）。

---

## 3. [待办与下一步] (Next Steps)

1. 向用户汇报阶段 2 优化成果与全景数据；
2. 待用户确认后执行原子提交与远端推送。
