# KindProbe 性能基准测试基线记录 (Original Baseline v1.0)

> **测试时间**：2026-11-23  
> **测试环境**：Windows 11, Release Profile (`cargo test --release --test test_performance_bench -- --nocapture`)  
> **代码版本**：v0.1.0 (1006 格式扩展名索引，59 个规则分卷，800+ 声明式规则)  
> **测试目标**：建立原版（未开启首字节桶调度、未开启 Fast Path format_id 二分查找）的速度对比绝对基准。

---

## 1. 扩展名索引检索基线 (Extension Binary Search)

- **总条目目数**：1006 项全局扩展名（ASCII 字典递增）
- **测试样本**：`["png", "jpg", "docx", "blend", "h5", "pdf", "flv", "zip", "exe", "xyz123"]`
- **迭代轮数**：500,000 次
- **单次延迟**：**47.31 ns**
- **吞吐量**：**21.14 M ops/sec** (2114 万次/秒)

---

## 2. 内存切片内容探测基线 (`detect_bytes` - 无扩展名提示全量扫描)

- **迭代轮数**：每格式 200,000 次

| 测试分类 | 测试载荷名称 | 单次延迟 (Latency) | 吞吐量 (Throughput) | 当前判定逻辑 |
|---|---|---|---|---|
| 早期硬编码 | PNG Image | **526.95 ns** (0.527 μs) | 1,897.7 k files/sec | 顶层专用图片解析器立即命中 |
| 早期硬编码 | JPEG Image | **597.24 ns** (0.597 μs) | 1,674.4 k files/sec | 顶层专用图片解析器立即命中 |
| 早期声明式 | Java KeyStore | **1,513.16 ns** (1.513 μs) | 660.9 k files/sec | DECLARATIVE_RULES 早期命中 |
| 早期声明式 | LIDAR LAS | **1,725.03 ns** (1.725 μs) | 579.7 k files/sec | DECLARATIVE_RULES 早期命中 |
| 中期分卷 | CAN Matrix SYM (P44) | **2,255.80 ns** (2.256 μs) | 443.3 k files/sec | GENERATED_RULES 第 44 分卷命中 |
| 末尾分卷 | Retro Sharp XDF (P59) | **2,917.91 ns** (2.918 μs) | 342.7 k files/sec | GENERATED_RULES 最后一个分卷命中 |
| 早期分卷 | FLV Video (P1) | **3,016.34 ns** (3.016 μs) | 331.5 k files/sec | GENERATED_RULES 规则匹配耗时 |
| 早期分卷 | Blender 3D (P1) | **3,138.03 ns** (3.138 μs) | 318.7 k files/sec | GENERATED_RULES 规则匹配耗时 |
| 早期分卷 | HDF5 Scientific (P2) | **4,545.61 ns** (4.546 μs) | 220.0 k files/sec | GENERATED_RULES 规则匹配耗时 |
| 窗口扫描 | PDF Document | **4,621.01 ns** (4.621 μs) | 216.4 k files/sec | 1024 字节窗口滑动匹配 |
| 纯文本引擎 | Python Shebang | **4,919.21 ns** (4.919 μs) | 203.3 k files/sec | 扫描完全部 800+ 二进制规则后命中 |
| 未知二进制 | Unknown Binary | **4,980.56 ns** (4.981 μs) | 200.8 k files/sec | 扫描完全部 800+ 规则及文本引擎后失配 |
| 纯文本引擎 | Verilog HDL Source | **5,260.24 ns** (5.260 μs) | 190.1 k files/sec | 扫描完全部 800+ 二进制规则后命中 |
| 纯文本引擎 | C Source Code | **5,375.96 ns** (5.376 μs) | 186.0 k files/sec | 扫描完全部 800+ 二进制规则后命中 |

---

## 3. 快速通道基线 (Fast Path - 带扩展名提示)

- **迭代轮数**：每格式 200,000 次

| 场景 | 测试载荷与提示扩展名 | 单次延迟 (Latency) | 吞吐量 (Throughput) | 当前判定逻辑 |
|---|---|---|---|---|
| Fast Path Hit | PNG (`hint: png`) | **598.57 ns** (0.599 μs) | 1,670.6 k files/sec | 快速通道命中硬编码探测器 |
| Fast Path Hit | Blender 3D (`hint: blend`, P1) | **951.10 ns** (0.951 μs) | 1,051.4 k files/sec | 快速通道线性扫描到 P1 命中 |
| Fast Path Hit | Sharp XDF (`hint: xdf`, P59) | **1,213.81 ns** (1.214 μs) | 823.9 k files/sec | 快速通道线性扫描到 P59 命中 |
| Fast Path Mismatch | PNG 数据 (`hint: blend`) | **1,289.51 ns** (1.290 μs) | 775.5 k files/sec | 快速通道失配后回退到全量扫描 |

---

## 4. 真实磁盘文件探测基线 (`detect_path` - 含系统 open + fstat + 预算读取)

- **迭代轮数**：每文件 2,000 次

| 真实文件 | 单次耗时 | 吞吐量 (Throughput) |
|---|---|---|
| `sample.pbm` | **24.81 μs** | 40,300 files/sec |
| `sample.fits` | **26.43 μs** | 37,834 files/sec |
| `sample.flv` | **27.96 μs** | 35,759 files/sec |

---

## 5. 深层复合容器结构解析基线 (Deep Container Structural Parsing)

- **迭代轮数**：每文档 5,000 次

| 文档类型 | 解析流程 | 单次耗时 | 吞吐量 (Throughput) |
|---|---|---|---|
| **DOCX** | ZIP EOCD 定位 + 中央目录解析 + XML Pull Parser 解压流探测 | **33.66 μs** | 29,709 docs/sec |
| **DOC** | CFB 扇区链遍历 + WordDocument 流提取 + FIB 结构校验 | **11.83 μs** | 84,516 docs/sec |
