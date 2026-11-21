# KindProbe 架构与工程交接文档 (HANDOFF)

> **单一真实信源 (Single Source of Truth - SSOT)**  
> **更新时间**：2026-11-21  
> **当前版本**：v0.1.0 (真实样本库与确定性对抗模糊测试落地)  
> **远端分支**：`origin/main` (当前提交待推送)

---

## 1. [已完成] (Completed)

### 1.1 真实世界文件样本矩阵 (`fixtures/real/` & `test_real_fixtures_matrix`)
- **生成脚本**：建立 `tools/generate_real_fixtures.py`，生成 25 种典型真实世界合规文件切片。
- **真实文件句柄端到端核验**：通过 `tests/test_real_fixtures_matrix.rs` 执行端到端真实 OS 文件句柄 `detect_path` 探测、格式判定、验证级别核对与双向扩展名诊断。

### 1.2 差异对抗性模糊测试套件 (`tests/test_adversarial_fuzzing.rs`)
- **确定性 PRNG 变异引擎**：手写零外部依赖的 `FastRng`（XorShift64），支持种子可重现测试。
- **混沌注入测试 (Chaos Fuzzing)**：测试 3,000+ 个随机字节序列（长度 0..8192 字节），断言 **零 Panic**、**读取预算绝对锁定**、**幂等性**与**扩展名独立性**。
- **魔数前缀截断模糊 (Prefix Truncation)**：针对核心格式魔数进行任意切断，修复了 `sqlite3`、`rar`、`gif`、`png` 等探测器中 `starts_with` 判定边界 Bug，确保截断准确回显 candidate。
- **位翻转变异 (Bit-flipping)**：随机翻转头部 1..3 个比特，证明系统在极端文件头损坏下安全降级且无挂起。

### 1.3 扩展名单一真实信源 (SSOT) 索引库
- 自动提取全部新老扩展名映射生成 `src/extension/registry.rs`（收录 161 个标准扩展名），严格按 ASCII 升序排列并去重，提供零哈希分配的 $O(\log N)$ 二分查找。

---

## 2. [当前状态] (Current State)

- **编译构建**：`cargo build` 成功。
- **代码质量与静态检查**：`cargo clippy --all-targets` 0 Error。
- **集成测试基线**：全量 `cargo test` **118 / 118 全部通过 (100% PASS)**（耗时 ~1.3 秒）。
- **Git 仓库状态**：准备提交并推送。

---

## 3. [待办与建议] (Blockers & Next Steps)

- **阻碍 (Blockers)**：无 (None)。
- **后续优化建议**：
  1. 可进一步通过 Codegen 接入 3D/CAD（STL, 3DS, DXF）与经典多媒体（MOD, XM, S3M）格式；
  2. 搭建 Criterion 纳秒级性能基准压测。
