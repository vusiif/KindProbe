# KindProbe

> 独立离线文件真实格式识别工具与 Rust 库（依据内容证据识别格式，推荐规范扩展名）。

**当前开发阶段：P1-A（最小可发布版 API / JSON v1 / CLI / 预算 / 错误与诊断稳定契约）**  
**项目许可证：GNU General Public License v3.0 或任何更新版本（SPDX：`GPL-3.0-or-later`），详见 [LICENSE](LICENSE)。第三方依赖与数据源保留各自许可证，见下文及 `research/`。**

---

## 1. 项目定位与边界

KindProbe 是一个离线、本地、只读的 Rust CLI 与核心库（依规格 `docs/FILE_FORMAT_RUST_SPEC_V1_2.md` 实现）：
- **内容优先**：识别完全基于文件内部结构与魔数证据，绝不依赖扩展名做身份确认。
- **扩展名诊断分离**：扩展名仅在内容识别后独立执行匹配诊断（`match` / `mismatch` / `missing` / `unrecognized` / `undetermined`），错误后缀不改变内容结论。
- **句柄级安全核验**：`detect_path` 先打开 `File` 句柄，再在已打开句柄上执行 `fstat`（`file.metadata()?.is_file()`）核验普通文件，拒绝目录、管道与设备，杜绝路径式检查与打开之间的 TOCTOU 竞态。
- **严格资源预算**：所有读取均经过 `BudgetReader` 计数适配器，对头窗（4 KiB）、总源读取（8 MiB）、ZIP 目录窗口（1 MiB）、容器条目数（10,000）、单 XML 解压输出（256 KiB）、总解压输出（1 MiB）、XML 嵌套深度（64）等关键资源设防，杜绝内存耗尽、拒绝服务与 XML 炸弹。进程级 Docker/OS 时间与内存护栏仅作为测试保护，不虚报为库内置保证。
- **拒绝伪确认**：严禁仅凭 `word/` 目录或文件名判定 DOCX；严禁仅凭 CFB 流名判定 DOC（必须验证 MS-DOC FIB `0xA5EC`、`nFib >= 0x00C1` 及表流存在性）。未实际核验的内部结构绝不虚报证据。
- **库无副作用**：Rust 核心库不打印标准输出、不写文件、不联网、不启动外部异步运行时。
- **明确不做的功能**：不修改扩展名、不重命名、不提取正文、不解压到磁盘、不进行宏执行或安全查毒、不进行无界递归探测、不集成 Toss 私有类型。

---

## 2. 格式能力与未支持矩阵 (Capability & Support Matrix)

| 格式名称 | 格式 ID (`format_id`) | 容器 ID (`container_id`) | 状态 | 验证手段与结构检查限制 |
|---|---|---|---|---|
| **Portable Network Graphics** | `png` | `null` | **Supported** | 8 字节头部签名 + IHDR 块有效性验证；仅有 8 字节头时仅给出 `signature` 证据，不虚报 `IHDR`；不完整签名降级为 unknown |
| **Portable Document Format** | `pdf` | `null` | **Supported** | `%PDF-` BOF 前缀与版本号解析；不执行全量交叉引用表校验与流解码 |
| **ZIP 归档格式** | `zip` | `zip` | **Supported** | EOCD 尾部搜索 + 中央目录完整性校验；在无任何 OOXML 部件时确认为纯 ZIP |
| **Microsoft Word Open XML** | `docx` | `zip` | **Supported** | ECMA-376 OPC 完整验证：`[Content_Types].xml` 内容类型 + `_rels/.rels` 包关系正规化解析 + 主部件实际存在 + `<w:document>` 命名空间根元素；支持 Transitional 与 Strict |
| **疑似 OOXML 未确认** | `null` | `zip` | **Container Only** | 存在 OOXML 特征但缺少关系或主部件时，降级为 `container_only`，不假报 DOCX 也不退为普通 ZIP |
| **Compound File Binary** | `null` | `cfb` | **Container Only** | OLE2 复合文件结构；扇区链循环保护；无 WordDocument 流时确认为纯容器 |
| **Microsoft Word 97-2003** | `doc` | `cfb` | **Supported** | 验证 CFB 容器 + 根目录 `WordDocument` 流 + MS-DOC FIB 头（`wIdent == 0xA5EC`, `nFib >= 0x00C1`）+ 表流验证；检测加密标志；虚假流/嵌套流降级为 `container_only` |
| **Office 宏变体 (DOCM / XLSM / PPTM)** | - | - | **Not Supported (P1-B)** | 宏结构识别保留至 P1-B；遇到时不误判为纯 DOCX |
| **Spreadsheets / Presentations (XLSX / PPTX / XLS / PPT)** | - | - | **Not Supported (P1-B)** | 在 P1-A 遇到时安全报告对应容器（`zip` 或 `cfb`），不误判为 Word |
| **归档格式 (7z / RAR)** | - | - | **Not Supported (P1-B)** | 7z/RAR 内嵌 DOCX 不得误判为外层 DOCX |
| **音视频媒体 (MP4 / MP3 / WAV / WebM)** | - | - | **Not Supported (P1-B)** | 音视频识别属于 P1-B 阶段 |

**支持平台验证状态**：
- Linux x86_64 (`Debian trixie / Linux 6.6`): **Verified (PASS)**
- Windows / macOS: **Not Verified (未验证)**

---

## 3. Rust API 快速上手

```rust
use kindprobe::{detect_path, detect_bytes, detect_reader, DetectOptions};
use std::path::Path;

// 1. 路径探测（安全打开句柄 + fstat 校验普通文件 + 独立扩展名诊断）
let options = DetectOptions::default();
let report = detect_path(Path::new("tests/sample.docx"), &options)?;
assert_eq!(report.format_id.as_deref(), Some("docx"));
assert_eq!(report.schema_version, 1);

// 2. 内存切片探测
let bytes = b"\x89PNG\r\n\x1a\n...";
let report_bytes = detect_bytes(bytes, &options)?;

// 3. Read + Seek 流探测
let mut cursor = std::io::Cursor::new(bytes);
let report_stream = detect_reader(&mut cursor, &options)?;
```

---

## 4. CLI 用法与退出码

### 4.1 基本命令
```bash
# 文本诊断输出
kindprobe document.docx

# 机器可读 JSON v1 输出
kindprobe --json document.docx

# 支持 -- 安全分隔以处理前导连字符文件名
kindprobe --json -- -document.docx

# 帮助与版本
kindprobe --help
kindprobe --version
```

### 4.2 退出码说明
| 退出码 | 含义 | 示例 |
|---|---|---|
| `0` | 格式确认 (`identification=format`) 且检查完整 (`inspection=complete`) | 校验合法的 PNG, DOCX, DOC, 普通 ZIP |
| `2` | 命令行参数错误 | 缺少文件路径、未知命令行选项 |
| `3` | 未达到完整格式确认 | 未知格式、仅容器确认 (`container_only`)、预算耗尽 (`limited`)、多候选冲突 (`ambiguous`) |
| `4` | 真实底层 I/O 错误或不支持的文件对象 | 文件不存在、无读取权限、目录、FIFO/管道、字符/块设备 |
| `1` | 非预期程序缺陷 | 内部异常 |

### 4.3 JSON v1 结构规范

正常识别输出信封（退出码 0 或 3）：
```json
{
  "schema_version": 1,
  "format_id": "docx",
  "category": "document",
  "container_id": "zip",
  "identification": "format",
  "verification": "structure",
  "inspection": "complete",
  "recommended_extension": "docx",
  "accepted_extensions": ["docx"],
  "extension_status": "match",
  "evidence": [
    {"code": "zip.central_directory"},
    {"code": "ooxml.content_types"},
    {"code": "ooxml.relationship.office_document"},
    {"code": "ooxml.word_main_part"},
    {"code": "ooxml.word_document_root"}
  ],
  "warnings": [],
  "stats": {
    "source_bytes_read": 16384,
    "seek_count": 4,
    "entries_inspected": 6,
    "decompressed_bytes": 2048,
    "duration_micros": 450
  }
}
```

CLI 错误输出信封（退出码 4 或 1，使用 `--json` 时输出）：
```json
{
  "schema_version": 1,
  "error": {
    "code": "io.not_regular_file",
    "message": "Path 'fixtures' is not a regular file (e.g. directory or device)"
  }
}
```

---

## 5. 依赖与第三方许可清单

| 依赖 crate | 版本 | 许可证 | 用途 |
|---|---|---|---|
| `zip` | 2.6.1 | MIT | ZIP 中央目录解析（受 `BudgetReader` 严格限制） |
| `quick-xml` | 0.36.2 | MIT | 流式 XML pull parser（限制深度 <= 64） |
| `cfb` | 0.10.0 | MIT | OLE2 扇区与流导航（附带 FIB 自研校验层） |
| `serde` | 1.0.229 | MIT / Apache-2.0 | 结构化模型序列化 |
| `serde_json` | 1.0.154 | MIT / Apache-2.0 | CLI JSON v1 输出格式 |

第三方数据源与调研工具：
- **PRONOM / DROID**: Open Government Licence 2.0 (OGL-v2.0)，仅作为离线对照基准，不直接打包未审查规则。
- **Siegfried**: Apache-2.0，仅作为测试期间独立复测对照工具。
- **Microsoft MS-DOC / MS-CFB**: 依据 Microsoft Open Specifications 技术公开规范查证实现。

---

## 6. 构建与回归测试验证指南 (Clean Checkout)

在普通的 clean checkout 仓库下，全部测试与夹具均已受控保存在仓库中，不依赖任何 `.sandbox-test` 历史绝对路径。

### 6.1 离线 Docker 沙箱测试契约
- 镜像：`local/rust-test:1.99.1-trixie`
- 镜像 ID：`sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3`

### 6.2 执行测试命令
```bash
# 1. 运行核心集成测试（包含 9 项失败回归、CLI 契约、预算对抗、API 一致性）
docker run --rm --name kindprobe-test \
  --pull never --network none --read-only \
  --cap-drop ALL --security-opt no-new-privileges \
  --user $(id -u):$(id -g) --group-add 1004 \
  --cpus 2 --memory 2g --pids-limit 128 \
  --tmpfs /tmp:rw,nosuid,size=256m,mode=1777 \
  -v $(pwd):/workspace:ro \
  -v $(pwd)/.sandbox-test:/output:rw \
  local/rust-test:1.99.1-trixie \
  cargo test --offline --manifest-path /workspace/Cargo.toml

# 2. 运行仓库自带的 52 组正反/对抗矩阵回归
python3 scripts/run_52_fixtures_regression.py

# 3. 运行仓库自带的 Codex 18 项全量验收测试
python3 scripts/run_acceptance.py
```
