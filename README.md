# KindProbe

> 独立离线文件真实格式识别工具与 Rust 库（依据内容证据识别格式，推荐规范扩展名）。

**当前开发阶段：P0（路线与复用验证完成）**  
**项目许可证状态：待定（由维护者另行指定，本项目不擅自声明；第三方依赖与数据源许可已在下文及 `research/` 完整披露）。**

---

## 1. 项目定位与边界

KindProbe 是一个离线、本地、只读的 Rust CLI 与库：
- **内容优先**：识别完全基于文件内部结构与魔数证据，绝不依赖扩展名做身份确认。
- **扩展名诊断分离**：扩展名仅在内容识别后独立执行匹配诊断（`match` / `mismatch` / `missing` / `unrecognized` / `undetermined`），错误后缀不改变内容结论。
- **严格资源预算**：所有读取均经过 `BudgetReader` 计数适配器，对头窗（4 KiB）、总源读取（8 MiB）、ZIP 目录窗口（1 MiB）、容器条目数（10,000）、单 XML 解压输出（256 KiB）、总解压输出（1 MiB）、XML 嵌套深度（64）等关键资源设防，杜绝内存耗尽、拒绝服务与 XML 炸弹。
- **拒绝伪确认**：严禁仅凭 `word/` 目录或文件名判定 DOCX；严禁仅凭 CFB 流名判定 DOC（必须验证 MS-DOC FIB `0xA5EC` 与 `nFib >= 0x00C1`）。
- **样本授权与边界**：严格限定测试样本仅为 (a) 本项目通过 `scripts/generate_fixtures.py` 自行合成的无侵权夹具；(b) 公开 HTTPS 已核准并记录哈希的样本。原规格 §11.4 提及的 Toss 本机语料未获授权，禁止读取、复制或引用；严禁访问 Toss、WorDF、FClue 的任何私有或本机样本。
- **库无副作用**：Rust 核心库不打印标准输出、不写文件、不联网、不启动外部运行时。

---

## 2. P0 门槛达成情况 (P0 Gates P0-G0 .. P0-G6)

| 门槛编号 | 门槛要求 | 达成状态 | 验证证据 |
|---|---|---|---|
| **P0-G0** | 完成现成库/工具（`file`, `Siegfried`, `infer`, `file-format`）对照，完成 PRONOM/DROID 等数据源审查，做出复用与许可决策 | **PASS** | `research/candidates.md`<br>`research/baseline-results.json`<br>`research/reuse-decisions.md`<br>`research/third_party_matrix.md`<br>`research/signature_sources.md`<br>`research/rules-manifest.json` |
| **P0-G1** | 真 DOCX 与普通 ZIP、伪 OOXML ZIP 不混淆；真 DOC 与普通 CFB、伪 Word 流不混淆 | **PASS** | 23 组样本实测：`fake_docx_zip.zip` 正确降级为 `container_only` (`ooxml.content_types_missing`)；`fake_doc_cfb.doc` 正确降级 (`cfb.invalid_word_fib`)；`true_docx.docx` 与 `true_doc.doc` 100% 正确确认 |
| **P0-G2** | 固定 P0 测试集中错误具体确认数必须为 0（False Identification = 0） | **PASS** | 23 组正反、畸形、对抗样本实测：**False Identification = 0** |
| **P0-G3** | 触发预算、截断、特性不支持、结构冲突时正确降级并输出稳定诊断码 | **PASS** | `budget_xml_deep.docx` 正确触发 `budget.xml_depth_exceeded` 并保持 `inspection=limited`；截断样本输出 `input.truncated`，空文件输出 `input.empty` |
| **P0-G4** | `detect_path` / `detect_bytes` / `detect_reader` 共用核心；错误后缀不改变内容判断；无写文件与联网副作用 | **PASS** | `tests/test_api_consistency.rs` 与 `tests/test_extension.rs` 均通过 |
| **P0-G5** | 真实 release 构建与样本运行度量（平台、工具链、体积、读取量、耗时、可测内存） | **PASS** | `.sandbox-test/results/release_measurements.json` 与 `.sandbox-test/results/p0_matrix_results.json`：二进制体积 852 KiB，单文件识别耗时 < 1 ms |
| **P0-G6** | 依赖/规则数据/局部源码独立复用决策，及未支持格式/平台清单 | **PASS** | 决策表及能力支持矩阵完整记录于 `research/reuse-decisions.md` 及下文 |

---

## 3. 格式能力与未支持矩阵 (Capability & Support Matrix)

| 格式名称 | 格式 ID (`format_id`) | 容器 ID (`container_id`) | 状态 | 验证手段与局限 |
|---|---|---|---|---|
| **Portable Network Graphics** | `png` | `null` | **Supported** | 8 字节头部签名 + IHDR 块有效性验证；不完整签名降级为 unknown |
| **Portable Document Format** | `pdf` | `null` | **Supported** | `%PDF-` BOF 前缀与版本号解析；不执行全量交叉引用表校验 |
| **ZIP 归档格式** | `zip` | `zip` | **Supported** | EOCD 尾部搜索 + 中央目录完整性校验；在无任何 OOXML 部件时确认为纯 ZIP |
| **Microsoft Word Open XML** | `docx` | `zip` | **Supported** | ECMA-376 OPC 完整验证：`[Content_Types].xml` 内容类型 + `_rels/.rels` 包关系正规化解析 + 主部件实际存在 + `<w:document>` 命名空间根元素；支持 Transitional 与 Strict |
| **疑似 OOXML 未确认** | `null` | `zip` | **Container Only** | 存在 OOXML 特征但缺少关系或主部件时，降级为 `container_only`，不假报 DOCX 也不退为普通 ZIP |
| **Compound File Binary** | `null` | `cfb` | **Container Only** | OLE2 复合文件结构；扇区链循环保护；无 WordDocument 流时确认为纯容器 |
| **Microsoft Word 97-2003** | `doc` | `cfb` | **Supported** | 验证 CFB 容器 + `WordDocument` 流 + MS-DOC Section 2.5.1 FIB 头（`wIdent == 0xA5EC`, `nFib >= 0x00C1`）；检测加密标志；虚假流降级为 `container_only` |
| **Office 宏变体 (DOCM / XLSM / PPTM)** | - | - | **Not Supported (P1-B)** | 宏结构识别保留至 P1-B |
| **Spreadsheets / Presentations (XLSX / PPTX / XLS / PPT)** | - | - | **Not Supported (P1-B)** | 在 P0 遇到时安全报告对应容器（`zip` 或 `cfb`），不误判为 Word |
| **归档 (7z / RAR)** | - | - | **Not Supported (P1-B)** | 7z/RAR 内嵌 DOCX 不得误判为外层 DOCX |
| **媒体格式 (MP4 / MP3 / WAV / WebM)** | - | - | **Not Supported (P1-B)** | 音视频识别属于 P1-B 阶段 |

**支持平台验证状态**：
- Linux x86_64 (`Debian trixie / Linux 6.6`): **Verified (PASS)**
- Windows / macOS: **Not Verified (未验证)**

---

## 4. 依赖与第三方许可清单

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

## 5. 本地复验与复跑说明

### 5.1 环境约定
测试环境合同：
- 本地 Docker 镜像：`local/rust-test:1.99.1-trixie`
- 镜像 ID：`sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3`
- 运行边界：`--pull never --network none --read-only --cap-drop ALL --security-opt no-new-privileges --user 1000:1000 --group-add 1004`
- 源码挂载：`/srv/projects/rust/KindProbe:/workspace:ro`（只读）
- 输出挂载：`/srv/projects/rust/KindProbe/.sandbox-test:/output:rw`（可写）

### 5.2 一键独立复验脚本
在宿主机项目根目录执行：
```bash
python3 /srv/projects/rust/KindProbe/scripts/verify_runner.py
```
该脚本将依次：
1. 校验 Docker 镜像与 ID；
2. 在受控沙盒内执行 `cargo test --offline`；
3. 在受控沙盒内执行 `cargo build --release --offline`；
4. 运行 23 组样本的实测矩阵，验证 `false_identifications == 0`，记录资源度量；
5. 全部门槛通过退出码为 0，否则非 0。

### 5.3 单独运行 CLI
```bash
# 查看帮助
docker run --rm --pull never --network none --read-only \
  --cap-drop ALL --security-opt no-new-privileges --user 1000:1000 --group-add 1004 \
  --cpus 2 --memory 2g --pids-limit 100 --tmpfs /tmp:rw,nosuid,size=256m,mode=1777 \
  -v /srv/projects/rust/KindProbe:/workspace:ro \
  -v /srv/projects/rust/KindProbe/.sandbox-test:/output:rw \
  local/rust-test:1.99.1-trixie \
  /output/target/release/kindprobe --help

# 识别 DOCX
docker run --rm --pull never --network none --read-only \
  --cap-drop ALL --security-opt no-new-privileges --user 1000:1000 --group-add 1004 \
  --cpus 2 --memory 2g --pids-limit 100 --tmpfs /tmp:rw,nosuid,size=256m,mode=1777 \
  -v /srv/projects/rust/KindProbe:/workspace:ro \
  -v /srv/projects/rust/KindProbe/.sandbox-test:/output:rw \
  local/rust-test:1.99.1-trixie \
  /output/target/release/kindprobe --json /workspace/fixtures/true_docx.docx
```
