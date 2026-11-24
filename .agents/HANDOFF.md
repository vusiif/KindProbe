# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-23  
> **当前版本**：v0.1.0 (完成首字节 256 桶调度与 Fast Path 二分索引，未知文件实测加速 7.87 倍，全格式提速 4~8 倍，全套测试 100% 绿标，Clippy 0 警告)  
> **远端分支**：`origin/main` (安全加固提交 `42a84e6` 已在远端主线)

---

## 1. [已完成] (Completed)

### 1.1 性能优化阶段 1：首字节 256 桶调度与快速通道二分索引 (First-Byte Bucket Dispatch & FastPath Index)
1. **首字节 256 桶调度表** ([`src/detectors/rules/dispatch.rs`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/src/detectors/rules/dispatch.rs))：
   - 将全局 790 条声明式与分卷规则按首字节进行 $O(1)$ 桶划分：756 条（95.7%）规则完美归入 256 桶，133 个桶完全为空（输入可 $O(1)$ 瞬间驳回全部二进制规则，直达纯文本或未知），非空桶平均仅 6.15 条规则；
   - 34 条 `offset > 0` 特殊规则安全归入紧凑回退数组，保障完全等价判定与 0 回归；
   - 快速通道按 `format_id` ASCII 建立已排序切片，实现 $O(\log N)$ 二分查找快速匹配。
2. **实测性能突破** (经 Release Profile 严密基准对比)：
   - **Unknown Binary 最坏情况失配**：从 4,980 ns 降至 **632 ns** (**7.87 倍加速，耗时骤降 87.3%**)；
   - **末尾分卷 Retro Sharp XDF (P59)**：从 2,917 ns 降至 **631 ns** (**4.62 倍加速**)；
   - **纯文本与源码探测 (C/Python/Verilog)**：从 4,919~5,375 ns 降至 **1,211~1,287 ns** (**4.1~4.2 倍加速**)；
   - **早期分卷 Blender/HDF5/FLV**：从 3,016~4,545 ns 降至 **685~697 ns** (**4.4~6.6 倍加速**)；
   - **基准测试总耗时**：从 10.30 秒压降至 **3.38 秒**（整体基准提速 3 倍）。
3. **测试规范与基准追踪归档**：
   - 建立永久基线归档 [`research/benchmark_baseline_v1_original.md`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/research/benchmark_baseline_v1_original.md)；
   - 建立全景优化对比追踪文档 [`docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`](file:///c:/Users/IceLolly/Music/Rust/KindProbe/docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md)。

---

## 2. [当前状态] (Current State)

- **本地测试状态**：
  - 全套 49 个测试套件，**187+ 个测试用例全部通过 (187+/187+ PASS)**；
  - `cargo clippy --all-targets -- -D warnings` **0 警告，0 错误**；
  - `src/lib.rs` 强约束 `#![forbid(unsafe_code)]`；
  - `src/` 全代码库保持 0 `unwrap()` / 0 `expect()` / 0 `panic!()`。
- **工作区变更**：
  - 新增 `src/detectors/rules/dispatch.rs`（256 桶调度与二分查找实现）；
  - 更新 `src/detectors/rules/mod.rs` 与 `src/detectors/signatures.rs`（接入索引调度）；
  - 更新 `tests/test_performance_bench.rs` 与 `docs/PERFORMANCE_BENCHMARK_AND_OPTIMIZATION.md`。

---

## 3. [待办与下一步] (Next Steps)

1. 向用户汇报阶段 1 优化数据成果与加速比对比；
2. 待用户确认后执行原子提交与远端推送；
3. 根据需要继续推进阶段 3（高频格式与 ASCII 快速前置门禁）。
