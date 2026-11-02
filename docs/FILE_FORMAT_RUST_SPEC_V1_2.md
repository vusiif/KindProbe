# 独立 Rust 文件真实格式识别工具：开发规格与接口约定（v1.2）

规格版本：**v1.2 · 2026-10-31**（基于 v1.1 迭代）。本文是需求、实现边界与验收约定，**不代表工程已创建、依赖已选定或功能已通过测试**。

**项目定位：独立仓库中的离线 CLI + Rust 库，依据文件内容识别真实格式并提供推荐扩展名。Toss 是未来可能的使用方之一，不是本项目主体。**

临时命令名 `filekind` 只用于接口举例；正式项目名、crate 名、许可证由维护者另定，本文不保证名称可用。

**本次修订原则**：完整继承 v1.1 的识别路线、安全约束、预算、公开接口和阶段验收；正式增加“先测现成工具/库 → 决策封装、局部复用或自研”，以及“研究权威文件签名数据库 → 审核来源/许可证 → 选择性导入 → 反向测试”两条前置工作流。**项目不是从零造所有 Magic Number，也不是把上千条未知质量的规则一股脑集成进产品。** 本文数值仍是 P0 待实测默认候选；文内的第三方候选与公开资料链接是截至 2026-10-31 的调研起点，**不代表源码审计、依赖决策或运行测试已完成**。

## 0. 接手范围、开发阶段与停止条件

独立开发窗口必须在自己的仓库内工作，**不得修改 Toss 源码、依赖、锁文件、识别策略或发布产物**。本规格不构成继续开发 Toss 的授权。

| 阶段 | 核心问题 | 最低交付 | 进入下一阶段的条件 |
|---|---|---|---|
| **P0：路线与复用验证** | 现成库/规则源能复用多少？内容证据能否可靠区别 ZIP/OOXML、CFB/DOC，并受预算约束？ | 现成工具/库基线、签名数据源清单与许可证筛查、最小 CLI/库原型；PNG/PDF、真实与伪造 DOCX/DOC、资源/体积实测、依赖与规则决策 | §11.2 P0 门槛全部满足，固定测试集错误具体确认数为零；不满足则换方案或缩范围 |
| **P1-A：最小可发布版** | 对外接口、报告、CLI 与错误处理是否稳定？ | 稳定 API/JSON v1、可执行预算、单文件 CLI、来源与回归文档、依赖锁定；以 P0 已验证格式为基础 | §11.3 P1-A 门槛满足；至少一个真实目标平台 release 验证通过 |
| **P1-B：扩展格式与兼容性** | 能否在不破坏准确性和资源约束的条件下增加识别范围？ | 补充 OFFICE 宏变体和其他家族、7z/RAR、音视频、跨平台测试；逐格式标注覆盖情况 | 逐格式通过 §11 的共同门槛；无必要时可推迟，不阻塞 P1-A |
| **P2：使用需求驱动** | 进一步的功能是否真的有人需要？ | 可选批量/目录扫描、复杂歧义和 polyglot 探测、可选重命名等独立功能 | 单独立项，默认不纳入首个可发布版本 |

**阶段门槛不是日期承诺**。P0 先完成有限的“R：调研与对照”子阶段，再进行“E：受限实现与实测”子阶段；研究必须尽快落到可执行对照，不能无限期只搜集资料。P0 可以在验证失败后停止、替换依赖或收窄支持范围，不得为了填满格式清单而放弃预算和正确性。允许在 P0 用部分 P1-A 报告字段方便测试，但不能因此声称 API 已稳定。

跨平台目标为 Windows、Linux、macOS；**先完成一个真实平台上的 P0 和 P1-A**，未运行的平台写明“未验证”，不以三平台全部完成阻碍首次路线验证。P1-B 再系统补齐平台覆盖。

成功标准按优先级为：**避免错误的具体格式确认 > 正确表达证据不足 > 控制读取/内存成本 > 许可与来源可追溯 > 扩大格式覆盖数**。不是追求支持上千扩展名。**能可靠复用时不重造轮子；不能可靠复用时宁可只复用格式知识，不复用不受控的解析路径。**

## 1. 产品边界

### 1.1 P1-A 必须具备（已纳入 P0 验证的能力先行）

- 内容优先；名字和扩展名仅用于独立的匹配诊断，不提高内容证据等级。
- 返回机器可用的格式 ID、类别、容器、识别层级、推荐扩展名、证据、警告和读取统计。
- 能区分“识别到具体格式”“只知道容器”“未知”，并报告检查因预算或不支持的容器特性而未完成。
- 文件路径、字节切片和 `Read + Seek` 输入可复用同一识别核心。
- 离线、本地、只读；不要求系统安装 Office、播放器、浏览器或外部格式识别命令。构建时可以使用经审核、固定版本的来源规则并编译进二进制；**运行时不联网更新或下载规则**。
- CLI 默认文本输出，可选择 JSON；不打开关联软件、不播放媒体、不解压到磁盘。
- Rust 库不得打印、退出进程、弹窗、写配置或自行联网。

### 1.2 P1-A 不做

- 修改扩展名、重命名、修复文件、提取正文、转换 PDF、完整解码与渲染。
- 文件杀毒、安全认证、宏检测的完整承诺、DRM 解密、口令破解。
- 递归扫描压缩包中的文件、无限嵌套识别、猜测裸数据流的编码参数。
- 根据机器学习给格式打任意百分比；没有校准数据，不输出虚构的“99% 可信度”。
- 把内容签名匹配或容器元数据一致，称为“文件完整有效”或“可以安全打开”。

## 2. 为什么不能只读文件头

Toss 当前源码中，自动识别先看已知扩展名，只有扩展名未知才读取前 16 字节；Word 处理器进一步按 `.doc` 选择旧 DOC 解析器。实际限制是：

- PNG 改成 `.pdf` 会被送往 PDF 处理器。
- DOCX 去掉扩展名，会按 ZIP 分流到解压。
- DOC 去掉扩展名，当前没有 CFB 识别兜底。
- DOC 与 DOCX 互换后缀，仍可能选择错误解析器。

这些是本次规格的项目背景，不要求独立工具复用 Toss 的代码、类型或枚举。

新工具应分层识别：**内容签名 → 必要的容器探测 → 格式内部标记一致性 → 报告证据与限制**。后缀比较在内容识别之后独立执行。文件签名数据库只能提供**候选规则及其来源**；命中一条规则不自动跨越后续容器/结构检查。

## 3. 格式范围及阶段归属

| 类别 | P0 必测的最小范围 | P1-A / P1-B 扩展方式 | 必须避免的误判 |
|---|---|---|---|
| 普通签名 | P0 至少有 PNG、PDF；按验证进度增加 JPEG、GIF、BMP | P1-A 固化已验证格式，剩余可到 P1-B | 仅一段短签名即声称整份文件有效 |
| ZIP / OOXML | ZIP 容器；DOCX 与普通 ZIP、伪 DOCX ZIP 的区分 | P1-B 增加 DOCM、XLSX、XLSM、PPTX、PPTM 等；非目标 Office 家族至少不能被认成 DOCX | 仅凭 `word/`、单个 `document.xml` 或扩展名确认 DOCX |
| CFB / Word | CFB 容器、Word97+ 二进制 DOC 的最小可靠识别及负向用例 | P1-B 扩充加密 DOC、其他 Office CFB 和旧版本覆盖；未实现情形必须降级 | 仅凭 CFB 签名或 `WordDocument` 流名确认 DOC |
| 归档格式 | ZIP 是 P0 必测 | 7z、RAR4/RAR5 至 P1-B；确认范围写入能力表 | 将 7z/RAR 内嵌 DOCX 识别成外层 DOCX |
| 音视频 | 非 P0 阻塞项 | P1-B：MP4/MOV 家族、Matroska/WebM、AVI、WAV、FLAC、MP3 | RIFF 不检查表单类型、EBML 不读 DocType、ID3 即认 MP3 |

P1-A **不承诺覆盖上表 P1-B 所有格式**。每种格式必须在能力表注明：`supported`、`container_only`、`not_supported`，及对应的结构检查和局限。未验证格式不能仅因文档列出就宣传已支持。

对于无法可靠区分的子类型，返回家族/容器并允许 `recommended_extension=null`。例如 ISO BMFF 品牌不明确时不依后缀强判 MP4；未知 EBML DocType 不能强报 WebM。

稳定 ID 采用小写字符串，例如 `png`、`jpeg`、`pdf`、`zip`、`docx`、`docm`、`doc`、`iso_bmff`、`matroska`、`webm`、`mp3`。**`cfb` 在本版主要作为 `container_id`，不是声称已确认 Word 格式的 `format_id`**。最终注册表与证据规则在 P0 决策后冻结为 P1-A v1；对外 ID 与 Rust 枚举名称分离。

文件签名数据库可以包含大量**超出支持范围**的条目，但只有进入本项目格式注册表、并通过对应正反测试的规则才算 `supported`。不得把数据源拥有的格式数当成产品支持数。

纯文本、JSON、XML、HTML、RTF 可列入 P2；若 P0 加入 RTF 签名作为 DOC 假后缀的负例，应独立报告 RTF，不伪装为二进制 DOC。未知内容不能因为可打印就默认 TXT。

## 4. ZIP / OOXML 识别要求

### 4.1 检查路线

1. 读取受限尾部和 ZIP 目录；不要为了找一个 XML 解压整个文件。
2. 在预算内检查根部 `[Content_Types].xml` 和包关系 `_rels/.rels`。
3. 按包关系定位主部件，结合其实际内容类型、部件存在性和必要的 XML 根标记判断家族及宏变体。
4. 使用 OOXML 允许的相对关系目标规范化规则定位包内路径；不因合法 `..` 段一律拒绝，也不允许归一化后越出包根。只访问容器内成员，不映射到宿主路径。
5. 明确是否支持 Transitional/Strict 命名空间；只支持其中一类时，另一类保留 ZIP 容器并给出检查限制。

不能仅凭 `word/` 目录、一个 `word/document.xml` 文件名或文件后缀判 DOCX。只放入这些假名字的普通 ZIP 应有负向测试。

### 4.2 普通 ZIP 与疑似 OOXML 的判定边界

- **普通 ZIP**：中央目录和当前版本规定的 ZIP 必要结构检查完成，未发现 OOXML 候选、冲突或其他待解决的专用容器标记时，可报告 `identification=format`、`format_id=zip`、`category=archive`，推荐 `zip`。这是“已确认 ZIP 归档格式”，**不是证明所有 ZIP 成员都完好**。
- **疑似 OOXML 但证据不足**：已经出现内容类型、主部件或关系等 OOXML 迹象，但缺失必需证据、出现冲突或不支持的特性时，保留 `container_id=zip`、`identification=container_only`、`format_id=null`；不得静默退回“普通 ZIP 已确认”，也不得推荐唯一的 `docx`。
- **预算/结构不完整**：只确认了 ZIP 特征或无法完成必要的目录检查时，返回 `container_only` + `limited`/相应证据；只有短签名而不足以确立容器时可返回 `unknown`。不得将文件损坏、未支持和预算耗尽混为一谈。
- P0 固定两类独立负例：① 普通 ZIP 放入伪造的 `word/document.xml`；② 有部分 OOXML 标记但必要关系不完整的 ZIP。预期结果不同，均不得确认为 DOCX。

### 4.3 歧义与降级

- **对疑似 OOXML**：缺失必要证据、XML 无法解析或关系指向缺失部件时，保留已确认 ZIP 容器，给出 `container_only` 及诊断，不假报 DOCX；普通 ZIP 的处理遵循 §4.2。
- 重复关键成员、规范化后重名、主关系互相冲突、多套主文档标记：报告歧义，不采用“最后一个覆盖前一个”。
- 加密成员、分卷、当前实现不支持的压缩方法/ZIP64 等：说明具体限制。若只支持普通 ZIP，不能把“不支持 ZIP64”说成损坏。
- 7z/RAR 中恰好含有 DOCX 文件，不将外层报告成 DOCX。
- 自解压文件可能同时具有可执行文件前缀与归档特征。P1-A/P1-B 均不要求支持 SFX；不得无界扫描全文件找签名。
- 内容类型、宏标记的识别不能被描述为“证明不含宏”。不执行或加载任何宏。

## 5. CFB / 二进制 DOC 识别要求

- 先验证必要 CFB 头和目录结构；使用有界解析，处理目录/扇区链循环和越界。
- 检查真正的流，不让同名 storage 或任意同名空文件伪装成 Word 流。
- Word97+ 候选需检查 `WordDocument` 流的 FIB 必要字段、版本、加密标记，以及按版本/标志所需的表流证据；具体版本判定规则由实现查证并写入测试，本文不提供未经验证的常量表。
- 不能仅凭 CFB 或 `WordDocument` 名称判为已确认 DOC。
- CFB 中的 Excel/PowerPoint 流、嵌入 OLE、加密 OOXML 包不能被当 DOC。当前阶段无法深入识别时仅报告 CFB 容器，允许结构化提示家族候选，但不要猜扩展名。
- `EncryptionInfo` / `EncryptedPackage` 类型的加密包，在没有足够证据时仅说明是加密 Office 包，不能从 `.docx` 后缀反推出内部一定是 DOCX。
- 已能从未加密元数据识别的加密 DOC，可以报告 `doc` 并提示受保护；识别到格式与支持读取内容是两回事。
- Word6/95、非 CFB 更早版本应分别说明实际覆盖；未实现时返回家族/容器/未知及限制，不能承诺支持所有 `.doc`。

## 6. 报告模型、合法状态与证据语义

P0 可内部使用较小模型，但必须保留迁移说明；**P1-A 发布 JSON `schema_version=1`**，下述语义视作外部契约，不随新增格式随意更改。

### 6.1 字段约定

| 字段 | P1-A 约定 |
|---|---|
| `schema_version` | 数字，P1-A 固定为 `1` |
| `format_id` | 稳定的小写格式 ID；仅当 `identification=format` 时非 null；否则为 null |
| `category` | `image/document/archive/media/container/unknown` 等稳定通用类；分类不代表解析能力 |
| `container_id` | 可空，例如 DOCX 的 `zip`、DOC 的 `cfb`；仅知道 ZIP/CFB 时依然填容器 ID |
| `identification` | `format` / `container_only` / `unknown`，代表身份识别的结论 |
| `verification` | `signature` / `structure` / `none`，代表实际取得的**最高证据层**；不宣称文件完整或可安全打开 |
| `inspection` | `complete` / `limited` / `ambiguous`，代表**本工具规定的检查**是否完成，不代表完整解码成功 |
| `recommended_extension` | 无前导点的小写扩展名或 null；身份/映射证据不足时必须为 null |
| `accepted_extensions` | 已确认格式接受的同义扩展名数组，如 `jpg/jpeg`；否则空数组 |
| `extension_status` | `match` / `mismatch` / `missing` / `unrecognized` / `undetermined` |
| `evidence` | 稳定的 `code` 与可选有界结构化详情；不得放正文、密码、整个 XML 或绝对路径 |
| `warnings` | 稳定的 `code`、英文 `message` 和可选有界详情 |
| `stats` | 实际来源读取字节、seek 次数、检查条目数、解压产出字节等；含义需统一 |
| `candidates`（可选） | 证据足够记录但尚未消除歧义的候选；不代表已确认为相应格式 |

`format_id` 与 `container_id` 不应互相替代。例如：明确的普通 ZIP 可以是 `format_id=zip, container_id=zip`；疑似 OOXML 却证据不足的 ZIP 是 `format_id=null, container_id=zip`；已确认 DOCX 是 `format_id=docx, container_id=zip`；已确认 DOC 是 `format_id=doc, container_id=cfb`。

### 6.2 合法状态组合与推荐扩展名

| `identification` | `inspection` | `format_id` | `container_id` | 推荐扩展名 | CLI 退出码 | 解释 |
|---|---|---|---|---|---|---|
| `format` | `complete` | 非 null | 可空 | 有唯一合理映射时可给出 | `0` | 格式必要检查完成 |
| `format` | `limited` | 非 null | 可空 | 仅在身份已达到充分证据、映射唯一时可给出 | `3` | 格式已确认，但定义内的其他检查因预算/特性受限；不得撤销充分证据 |
| `container_only` | `complete` | null | 非 null | null | `3` | 完整执行了可用检查，仍无法确认具体格式 |
| `container_only` | `limited` | null | 非 null | null | `3` | 只确认容器，部分必要检查未完成 |
| `container_only` | `ambiguous` | null | 非 null | null | `3` | 容器确认，内部格式候选互相冲突 |
| `unknown` | `complete` | null | null | null | `3` | 规定范围内检查完毕，无足够身份结论 |
| `unknown` | `limited` | null | null | null | `3` | 连容器也无法可靠确定，检查已受限 |
| `unknown` | `ambiguous` | null | null | null | `3` | 顶层证据冲突、无法确认唯一身份或容器 |

**禁止** `identification=format` 与 `inspection=ambiguous` 同时出现。若存在互相冲突的有效候选，撤销唯一格式确认，保留能共同确定的容器或返回未知。`verification=none` 不得与已确认格式/容器并存。`complete` 只表示规定的证据路径已经检查完毕，绝不表示已校验所有 ZIP CRC、PDF 交叉引用或媒体完整性。

`format + limited` 仅用于**格式身份所必需的证据已完整取得**，但可选或后续规定检查受限的情形；若身份必需步骤受限，应降为 `container_only` 或 `unknown`，不得保留伪确认。P0 必须为两种情形分别配置测试。

只识别到某个家族、尚不能唯一推荐扩展名时，优先用共同容器/候选证据表达，而不是虚构一个具体扩展名。P1-A 不要求通用 polyglot 检测，不对恶意多格式文件作完整保证。

### 6.3 扩展名比较独立且有固定优先级

内容探测**完全不使用文件名或后缀来确认身份**。比较逻辑仅作用于报告诊断，建议按以下次序执行：

1. 调用方没有提供可用后缀：`missing`，无论是否已确认格式。
2. 已有后缀，但 `identification != format`，或该格式无法确定同义扩展名关系：`undetermined`。
3. 已确认格式，后缀在其 `accepted_extensions` 中：`match`；比较时规范化 ASCII 大小写，非 UTF-8 不得 panic 或做有损确定性匹配。
4. 已确认格式，后缀是注册表中属于其他**不兼容**格式的已知后缀：`mismatch`。
5. 已确认格式，但后缀不属于任何已登记可判断的后缀：`unrecognized`；已知家族共享/历史歧义后缀导致结论不足时用 `undetermined`。

`mismatch` 是内容与文件名的冲突告警，不影响识别结果，也**不自动修改文件**。同一内容换成错误后缀，除匹配诊断外的身份与证据应相同。

### 6.4 JSON 示例（规范示例，非实测）

已确认的 `report.zip` 实际为 DOCX：

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
  "extension_status": "mismatch",
  "evidence": [{"code": "ooxml.word_main_part"}],
  "warnings": [],
  "stats": {
    "source_bytes_read": 16384,
    "seek_count": 4,
    "entries_inspected": 12,
    "decompressed_bytes": 4096
  }
}
```

疑似 OOXML，但主关系缺失：

```json
{
  "schema_version": 1,
  "format_id": null,
  "category": "container",
  "container_id": "zip",
  "identification": "container_only",
  "verification": "structure",
  "inspection": "complete",
  "recommended_extension": null,
  "accepted_extensions": [],
  "extension_status": "undetermined",
  "evidence": [{"code": "zip.central_directory"}],
  "warnings": [{"code": "ooxml.required_relationship_missing", "message": "Required OOXML package relationship is missing"}],
  "stats": {
    "source_bytes_read": 8192,
    "seek_count": 3,
    "entries_inspected": 6,
    "decompressed_bytes": 1024
  }
}
```

第二例假定原始文件**带有某个后缀**；如果没有后缀，`extension_status` 应为 `missing`。损坏候选、预算中断、未知格式与 I/O 失败的 JSON 也要分别有稳定测试。示例数值均非实测。

### 6.5 诊断码稳定性

- `warnings[].code`、`evidence[].code` 是机器可用标识，`message` 是给人看的诊断，不作为解析 API。
- 识别规则应在内部保留稳定 `rule_id`、来源、原始规则定位、规则集版本及本项目复核记录；P1-A 允许在 `evidence[].details` **有界地**输出本项目稳定规则标识（不是隐含承诺第三方 ID 永远不变），默认不得泄漏整个规则数据库/输入片段。
- 推荐按稳定前缀组织，如 `zip.*`、`ooxml.*`、`cfb.*`、`budget.*`、`io.*`、`input.*`；P0 可调整，P1-A 冻结已公开的语义。
- 预算类至少区分：`budget.source_read_exceeded`、`budget.zip_directory_window_exceeded`、`budget.entries_exceeded`、`budget.xml_output_exceeded`、`budget.total_decompressed_exceeded`、`budget.xml_depth_exceeded`、`budget.structure_capacity_exceeded`（名称可在 P0 统一一次）。
- 非预算类至少区分：`zip.unsupported_feature`、`zip.conflicting_entries`、`ooxml.required_relationship_missing`、`cfb.invalid_chain`、`cfb.unsupported_word_version`、`input.truncated`；不要把“当前不支持”说成“损坏”。
- 诊断详情有大小/数量上限。P1-A 建立每种代码的触发测试；增补代码可以兼容，删除或改变已有代码含义应走版本规则。

## 7. Rust 库接口

P1-A 最小公开接口形状如下。类型/函数名可在 P0 调整一次，固化后按正常版本规则维护；这是签名约定，不是已经实现的代码：

```rust
pub fn detect_path(
    path: &std::path::Path,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError>;

pub fn detect_bytes(
    bytes: &[u8],
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError>;

pub fn detect_reader<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    options: &DetectOptions,
) -> Result<DetectionReport, DetectError>;
```

- 内容探测接口不接受 Toss 请求对象、UI 回调或 Word/PDF 转换器内部模型。
- `detect_bytes` 不复制整个输入；不要求调用方为了路径输入先读成 Vec。
- reader 输入约定逻辑文件起点是绝对 offset0，探测结束后位置未指定；调用方需要保留位置可自行保存恢复。不要为了支持流式 stdin 立即实现全文件缓存。
- 扩展名比较建议为独立纯函数，例如 `compare_extension(report, Option<&OsStr>)`；`detect_path` 可在内容识别后调用它填充诊断。非 UTF-8 后缀不得 panic 或强行替换后作确定性匹配。
- 保留结构化错误：打开/读取/seek 失败、不是普通文件、参数非法等。不要用一个字符串让调用方反向解析。
- 未知格式是正常报告；已读内容显示格式不完整/歧义，是有诊断的报告；真实 I/O 失败是 Err。达到内部预算返回 limited 报告，不伪装成 I/O 错误。
- 路径接口只接受普通文件，可跟随文件符号链接；文件打开后检查句柄属性，目录/管道/设备拒绝。不得声称跨平台都具有不可绕过的特殊文件防护；平台限制需说明。
- 外部 `Read + Seek` 实现可能阻塞；库不能承诺硬实时超时。库不擅自创建线程、子进程或异步 runtime。
- 多次调用、并发独立调用结果稳定，不要求全局可变注册表。公开类型扩展策略在 P1-A 确定，避免枚举一增加变体就意外破坏下游。
- 候选规则必须只读、可复现；同一固定版本的嵌入规则集不得受主机安装的 `magic.mgc`、系统 MIME、当前网络或文件名提示影响内容身份判断。


### 7.1 实现分层与可测试性

- 内容识别核心只接收字节/可寻址 Reader 与选项，不接收扩展名；独立纯函数做后缀诊断。`detect_path` 负责打开与元数据验证，再调用同一核心并追加后缀诊断。
- 所有来源读取通过统一的**有预算计数适配器**；即使底层 ZIP/CFB 库自己 seek/read，也必须被覆盖。`stats.source_bytes_read` 统计探测操作实际返回的来源数据字节及缓存预读，不假称是操作系统磁盘物理读取量。
- 外部 `Read + Seek` 可能是缓慢或恶意实现；该 API 不保证硬实时中断。除 `detect_path` 的本地文件对象检查外，不承诺防止任意外部 Reader 副作用。
- 同一文件在探测中被外部进程修改不保证快照一致性；每次识别使用同一已打开句柄。无需因此增加全文件复制。

## 8. CLI 及退出码

P1-A 仅要求**单文件**：

```text
filekind input.bin
filekind --json input.bin
filekind --json -- input-with-leading-dash
filekind --help
filekind --version
```

- 默认文本输出明确区分实际格式、容器、证据等级、检查状态、推荐后缀及冲突，不把“签名匹配”说成文件安全/完整。
- `--json` 时 stdout 必须恰好是一个完整 JSON 对象加换行；不得混入横幅、进度、调试信息。错误也使用 JSON 错误信封，允许非零退出。
- 暂不支持 stdin 时在 help 中说明；`-` 不得隐式触发无界缓存读取。
- Windows 中文/空格路径、Unix 非 UTF-8 路径要有测试；不得把完整路径编码为识别核心的必要输入。默认 JSON 不包含绝对路径、正文或成员清单。
- 不做重命名、覆盖、递归扫描、解压到磁盘或联网更新规则库。

| 退出码 | 含义 | 例子 |
|---|---|---|
| `0` | `identification=format` 且 `inspection=complete` | PNG、足够证据的 DOCX、规定范围内完整检查的普通 ZIP；后缀冲突仍为 0 |
| `2` | CLI 参数错误 | 无文件参数、非法选项 |
| `3` | 获得报告但**未达到完整格式确认** | `unknown`、`container_only`、任何 `limited`、任何 `ambiguous`，包括 `format+limited` |
| `4` | 真实打开/读取/seek 失败或不支持的文件对象 | 权限失败、目录、设备、I/O 错误 |
| `1` | 非预期内部故障 | 程序缺陷；正常截断或恶意文件不得导致 panic |

`--help` / `--version` 返回 0，但不输出识别 JSON。`--json` 错误使用固定信封，如 `{"schema_version":1,"error":{"code":"io.open_failed","message":"..."}}`，不得混用 JSON/纯文本；CLI 错误消息可能含本地路径，不自动遥测或上传。

## 9. 资源预算与边界保护

下表是**P0 待校准默认候选值**；必须用真实样本验证，修改默认值须记录前后结果，不得为通过测试默默关掉护栏。

| 项目 | 候选默认值 | 计数/触发语义 |
|---|---|---|
| 首次头部窗口 | 4 KiB | 限定初次探测窗口；不是每次调用额外赠送额度 |
| 整次来源读取总量 | 8 MiB | 含重复读取和显式缓存预读，按适配器实际读取字节累计 |
| ZIP 尾部/中央目录专用读取窗口 | 至多 1 MiB | 同时计入来源读取总量；合法大目录无法完成时 `limited` |
| 容器目录/流条目检查数量 | 10,000 | 检查条目总量；并给目录名称、单条记录、候选记录数量设上限 |
| 单个探测 XML 解压输出 | 256 KiB | 按实际输出计数，超限停止，不按未可信的声明长度决定 |
| 整次探测解压输出 | 1 MiB | 跨所有需要探测的成员累计 |
| XML 元素深度 | 64 | 同时限制节点数、属性数、名称/值长度以及解析器缓存 |
| 容器嵌套 | 仅外层和定义必要的部件 | 不递归识别压缩包内的任意文件 |

### 9.1 预算的真实含义

- **8 MiB 不是文件大小上限**；允许定位大文件尾部/内部结构而不完整读入。
- 所有 I/O 必须经过可计数边界，包括解析库内部发起的读取；`source_bytes_read` 是**用户态探测来源读取量**，不是操作系统物理磁盘吞吐量。重复范围读取仍计数。
- 预算检查应发生在大规模分配或解压前；解压计数以实际输出为准，不能相信 ZIP 报告的大小/压缩比。
- 目录项、文件名、XML 节点/属性、列表、索引和诊断详情必须分别有界；**读取限制不能代替内存上限**。P0 不保证严格硬内存上限，需用受限独立进程测试并披露风险。
- 禁用 XML 外部实体、网络解析和无界实体展开；不能把恶意 XML 交给不受控的 DOM 构建。
- 默认只检查必须的文件内部部件，不能遍历所有解压成员；合法大 ZIP 目录超预算不能被错误认定为损坏。
- **规则引擎也受预算约束**：规则数量、比较次数、单规则读取范围、BOF/EOF 偏移、搜索窗口、回溯/重复匹配、跳转链、候选列表及构建期规则膨胀均须有界；不能因为规则来自公共数据库就放行无界 `search`、EOF 扫描或间接偏移。

### 9.2 降级与错误区分

- **预算耗尽**：返回带 `inspection=limited` 和稳定 `budget.*` 诊断的报告；保留已充分证明的容器或格式，不补猜未完成的必要证据。
- **实现尚不支持的结构特性**（例如 ZIP64、加密、分卷、某种压缩方法）：附 `unsupported_feature` 类诊断；如果因此无法完成身份确认，返回 `limited`，不要称作损坏。
- **检测到不一致/截断**：返回结构诊断和可得的身份层级；普通数据损坏不应变成 Rust panic，也不应混同真实 I/O 失败。
- **实际 I/O 失败**：返回 `DetectError`，CLI 用退出码 4；不得为了生成 `unknown` 而吞掉硬错误。
- 若已确认格式但只有**可选的后续检查**因预算不足，允许 `format+limited`；身份必需检查未完成则降级至 `container_only`/`unknown`，见 §6.2。

### 9.3 必须覆盖的敌意输入与时间边界

- `seek` 越界、整数溢出、声明尺寸巨大、零长度、短读、`Interrupted`、截断、重复 ZIP 关键条目、中央目录伪偏移、CFB 扇区链循环、恶意 XML。
- 不得在预算中断后再启动无界的“兜底扫描”。ZIP/CFB 解析器若在预算检查前分配巨大内存，属于依赖准入问题。
- 软耗时检查不能变成硬超时保证；外部 Reader 可阻塞。P2 才考虑独立进程硬取消。P0 在外部测试护栏下测内存/时间，**护栏不算产品实现功能**。
- 路径打开后复用句柄并检查文件类型，拒绝目录、管道和设备；对跟随符号链接、竞态及跨平台文件对象限制保持如实说明。

## 10. 依赖、体积、性能与 P0 决策关卡

- **先跑现成方案，再决定是否写新实现**。优先评估成熟 Rust 签名/容器/XML 解析能力，**但不把“某 crate 能打开文件”视作可以安全有界探测**；首版无需实现完整 ZIP/CFB/PDF/音视频解析器。
- 不为识别少量容器元数据引入完整 Office 渲染器、图像/视频解码框架、FFmpeg、GUI 或额外 runtime。
- 开工时查证候选依赖的实际 API、版本、许可、维护、功能特性和间接依赖；锁定版本及 feature。§10.3 的候选名称/历史查阅版本**不是最终依赖选定清单**。
- 允许三种复用途径：**直接依赖**（可受预算控制）、**局部代码复用/移植**（满足许可证、保留归属和修改记录）、**仅研究规则/格式规范后独立实现**。任何一种都必须有可追溯出处与正反测试；不得把不同许可证的规则数据和源代码混作一个许可结论。
- 默认关闭无关的写入、联网、加密破解、图形和格式转换 feature；不得为了缩小体积而牺牲必要的结构核验。
- 保持单个 CLI 可执行文件，库可供下游链接；不要求 Toss 再安装 CLI、外部规则文件或数据库。

### 10.1 依赖准入检查表（P0 结束前必须形成记录）

| 决策维度 | 必须回答的问题 | 不满足时的动作 |
|---|---|---|
| I/O 可控性 | 解析器能否只通过受预算 Reader 读取、seek？ | 包装适配器；不能约束时换库或缩范围 |
| 分配行为 | 是否会先根据未可信长度分配目录、字符串、XML/索引？ | 逐层设上限、替换解析方式或拒绝该实现路径 |
| 必要证据 | 能否取得 OOXML 关系和内容类型、CFB 真实流及最小 FIB 字段？ | 不用“文件名存在”代替；当前格式不宣布支持 |
| 恶意输入 | 截断/环链/重复成员/尺寸溢出是否可能 panic、循环或耗尽内存？ | 外部受限测试；不满足则阻断进入 P1-A |
| 开销 | release 二进制、依赖闭包、冷启动和库内耗时是否可接受？ | 记录权衡，必要时拆 feature；不先写虚假指标 |
| 法律与供应链 | 代码、规则数据库、样本各自许可证及再分发条件、归属、锁文件、平台是否可追溯？ | 未核实前不得拷贝/内置/发布相关资产 |
| 规则质量 | 签名是否有歧义、容器内部证据是否足够、来源是否可信、是否有伪造/截断反例？ | 只保留候选或容器证据，不升级为具体格式 |

P0 决策必须写成 `accept` / `replace` / `defer`，附依据、具体测试/代码位置、已知风险和对应影响格式。**允许缩减格式范围以保住准确性和预算**。

### 10.2 必须记录的度量

- `cargo build --release` 的真实产物尺寸（平台/目标三元组、Rust 工具链、CRT、优化选项、feature、依赖版本/数量）。
- 同样本：普通签名、DOCX、DOC、巨大目录、可寻址大文件的 `source_bytes_read`、seek 次数、解压产出、耗时和可测内存。
- 将**CLI 进程启动+运行时间**和**库内识别调用耗时**分别记录；在没有基线前不设“必须小于 1 ms”等虚假门槛。
- 本项目许可证由维护者决定，不因 Toss 是 GPL-3.0-or-later 自动继承；留存依赖许可证和测试样本来源，不把规格本身当法律授权。

### 10.3 P0-R：现成工具与 Rust 库的先行对照（必须先于大规模自研）

下表是**待验证候选**，用途是缩短开发周期，不代表符合本项目安全预算。优先选择跨路线的少数代表，不为了研究拖延实现。

| 候选 | 已知定位（公开文档所述） | P0 要验证的重点 | 预期用途 |
|---|---|---|---|
| [`file-format`](https://docs.rs/file-format/latest/file_format/) | Rust 多格式识别；可选 ZIP/CFB/MP4/EBML 等 reader feature | `from_file` 与 `from_reader` 的读取方式、结构证据、伪 DOCX/DOC、不同后缀、预算及依赖闭包；**不直接使用可能全文件读取的路径作为默认核心** | 竞争基线；通过时复用候选检测层/容器判断 |
| [`infer`](https://docs.rs/infer/latest/infer/) | Rust 轻量 Magic Number 识别与自定义匹配 | 签名准确性、false positive、读取窗口、规则重复性 | 首层候选筛选或参考规则 |
| [`pure-magic`](https://docs.rs/pure-magic/latest/pure_magic/) + [`magic-db`](https://docs.rs/magic-db/latest/magic_db/) | 纯 Rust Magic 规则解析器与预编译规则库 | 规则大小/启动成本、无界规则、适配 `Read+Seek` 能力、第三方规则版权/许可链、可观测性 | 规则引擎参考/可选对照，而非自动纳入核心 |
| [`zip`](https://docs.rs/zip/latest/zip/) + [`quick-xml`](https://docs.rs/quick-xml/latest/quick_xml/) | ZIP 读取及流式 XML 解析 | 有界目录读取、成员选择、XML 安全边界、合法 ZIP64/大目录的降级行为 | OOXML 结构层的优先候选 |
| [`cfb`](https://docs.rs/cfb/latest/cfb/) | CFB 复合文件访问 | 真正的流与 storage、截断/环链、大分配、FIB 验证能否作为附加层实现 | DOC 结构层的优先候选 |
| [`ooxml-opc`](https://docs.rs/ooxml-opc/latest/ooxml_opc/) | Office Open Packaging Conventions 层 | 关系、内容类型、命名空间、路径规范化、内存预算与依赖成本 | 可替代部分自编 OPC 逻辑 |
| [`DROID`](https://www.nationalarchives.gov.uk/information-management/manage-information/preserving-digital-records/droid/) / [`Siegfried`](https://github.com/richardlehane/siegfried) / 系统 `file` | 独立识别系统；DROID 与 Siegfried 可利用 PRONOM 签名 | 用相同样本给出另一条实现路线的结果；记录版本、规则集、选项及不一致 | **测试对照，不作为产品运行时依赖** |

**P0-R 对照协议：**

1. 建立固定样本 manifest：真/伪 DOCX、真/伪 DOC、普通 ZIP/CFB、PNG/PDF、缺失/错误后缀、截断/重复成员/超大目录/预算攻击样本；每份有真值依据与 SHA256。不要仅靠任一工具输出赋真值。
2. 同一批样本、固定工具版本和规则集版本执行对照；分别记录具体格式/仅容器/未知/报错、原始输出及候选冲突。至少对照一条**通用签名路线**与一条**容器/结构路线**，再与自有最小识别器比较。
3. 对 `file-format`、`infer` 等库级候选，使用 `detect_reader` 风格预算计数适配器尝试约束读取；记录实际读取、seek、内存/时间、CLI 大小与间接依赖。无法介入内部读取时，只作外部基线，不宣称可以内置。
4. 做错误后缀与伪造容器的负测：能识别“普通 ZIP”并不等于能判断 DOCX；能读取 CFB 并不等于能判 DOC。**准确分流的误确认率优先于支持数量。**
5. 形成逐候选决策：`wrap`（直接依赖/封装）、`reuse_part`（局部复用）、`reference_only`（对照/借鉴）、`replace` 或 `defer`，附理由。不能只凭 crate 主页宣告“可复用”。

**最小 P0-R 产物**：`research/candidates.md`、`research/benchmark-manifest.*`、`research/baseline-results.*`、`research/reuse-decisions.md`。这些是建议文件名，实际项目可调整，但内容不得省略。**对照可以先用单平台完成，不要求先移植 DROID/Siegfried 到 Rust。**

### 10.4 文件签名数据库与格式描述资源（来源目录，不是默认依赖）

| 来源 | 能提供的内容 | 本项目建议用途 | 必须留意 |
|---|---|---|---|
| **[PRONOM](https://pronom.nationalarchives.gov.uk/about)** / [DROID 二进制与容器签名](https://pronom.nationalarchives.gov.uk/signature-list) | 格式标识 PUID、扩展名/版本、BOF/EOF 等签名、容器签名；可下载独立 XML 与历史版本 | **第一优先级**：查证规则、整理候选和与 DROID/Siegfried 交叉验证 | 有偏移/范围/通配与容器规则；XML 不是“直接可用 Rust 检测逻辑”。需核对具体数据集许可/署名和版本 |
| **[`file` / `libmagic` 规则库](https://github.com/file/file)**、[magic(5) 规则语法](https://github.com/file/file/blob/master/doc/magic.man) | 文件头、多级继续规则、偏移判断、类型推断、MIME 描述 | 第二来源；补充 PRONOM 缺项，并与系统 `file` 对照 | 规则语法可能有间接偏移、搜索/继续逻辑；规则库与程序许可证须分别审查，不把 `magic.mgc` 当无审查的打包资源 |
| **[freedesktop shared-mime-info](https://specifications.freedesktop.org/shared-mime-info/latest/)** | MIME 类型、Magic、glob、继承关系与名称提示等 | 辅助 MIME/别名映射、核对简单 Magic | **glob 与文件名权重不得参与内容身份确认**；数据库/规则包许可另行查证，不能仅凭规范页面认定可内置 |
| **[Kaitai Struct 格式库](https://github.com/kaitai-io/kaitai_struct_formats)** | `.ksy` 二进制结构描述、字段、偏移和约束 | 帮助编写 DOC/媒体等最小结构验证；可用作研究参考 | **每个 `.ksy` 分别授权**，生成的代码也需核查许可/运行时/成本；不是统一许可的 Magic 数据库 |
| **[SEARCH / GCK 文件签名表](https://filesig.search.org/about)** | 人工维护的常见十六进制签名、扩展名参考 | 人工交叉核验短签名，查找罕见反例 | 不完整；本页本身不能证明批量抓取/复制/再分发权限，默认只做参考 |
| **[Siegfried](https://github.com/richardlehane/siegfried)** | PRONOM / freedesktop 等多源规则的实现与独立对照能力 | 研究优先级、规则编译和冲突处理；对照运行结果 | 它是 Go 项目；其规则源的许可**不因 Siegfried 代码采用 Apache-2.0 就自动相同**；默认使用无缓冲上限签名时可能扫描全文件，须限制/观察 |
| **[TrID](https://mark0.net/soft-trid-e.html)**（选读） | 社区格式识别定义及工具 | 仅在其他来源不足时查某个格式 | 格式定义及批量分发许可未核实之前，不能导入发布产物 |

截至本规格查阅时，PRONOM 官网提供按格式访问的 JSON/XML、最新二进制签名 `binary-signature.xml`、容器签名 `container-signature.xml`、历史版本和 GitHub 数据入口；其历史列表可见 V125，但**项目必须固定实际下载的版本/commit/hash，不能把 `latest` 作为可复现构建依赖**。PRONOM 研究仓库说明其数据使用 Open Government Licence 2.0，仍须确认本次实际取用的资源及分发场景的适用条款。参见 §15。[R1][R2][R3]

**不同数据源的语义不要混为一谈**：格式 ID / PUID / MIME / 扩展名是不同字段；一个 PUID 可能指向某版本，多个 MIME/扩展名可能共享容器。外部来源没有本项目格式 ID 的唯一映射权。

### 10.5 离线规则导入：严格采用“白名单 → 审核 → 生成 → 测试”

**首版建议**：**构建期**从固定版本的一个或少数规则源离线提取**已验证的最小规则子集**，审核后生成项目自有静态数据或少量 Rust 常量；最终可执行文件不必携带外部 XML/数据库，也不运行时下载。P0 先手写/整理极少量已核对规则亦可；不要求立即做通用 PRONOM/libmagic 解释器。

导入流水线必须有以下关卡：

1. **Source**：记录来源 URL、发布者、检索日期、固定 release/commit、原文件 SHA256、许可链接和归属要求；未核实权利时仅研究，不放进发行产物。
2. **Parse**：隔离的构建期导入工具解析来源（PRONOM XML、magic 文本、MIME XML 等）；对未知语法**拒绝或显式标记 unsupported**，不可“丢掉复杂限定条件后把剩下几个字节当原规则”。
3. **Normalize**：转换成有界中间表达，写明 `BOF`/`EOF`、绝对/相对偏移、字节掩码、范围、层级/依赖、优先级、候选格式/容器、可选 MIME/PUID、来源 ID。禁止把 filename/glob 当身份确认证据。
4. **Review**：人工或独立审查规则是否太短、是否跨格式共享、与既有规则冲突、是否需要容器/内部结构二次确认；为每条标注证据等级 `candidate_signature` / `bounded_structure`，短/弱签名不得直接升级为具体格式。
5. **Generate**：仅对**许可证允许且已复核**的白名单条目生成确定性排序的内置注册表；记录输出哈希、生成器版本、来源映射与许可通知。构建不应联网。生成文件应可复现，不受主机系统数据库影响。
6. **Validate**：为每个新增规则配正向、共享签名负例、短签名/截断、错误扩展名、预算/恶意输入；与 DROID / Siegfried / `file` 等外部基线比较差异，但**不以多数票替代格式规范或可追溯真值**。
7. **Promote**：只有通过 §11.3 同等格式门槛的规则才能进入 `supported`。其他规则可保留在研究索引、候选层或暂不打包；不给“上千格式已支持”的产品宣称。

**示意中间规则（非真实 PRONOM 记录，非最终公开接口）：**

```yaml
rule_id: sig.png.001
source:
  dataset: local-reviewed
  version: p0-fixture-1
  source_ref: png-signature
match:
  anchor: bof
  offset: 0
  bytes_hex: "89504E470D0A1A0A"
output:
  candidate_format_id: png
  evidence_level: candidate_signature
requires:
  - png.minimal_header_checks
```

不能把 `bytes_hex` 列表本身当做完整认证，也不能只保留 `bytes_hex` 而丢失原始规则所依赖的版本/掩码/约束；生成器不支持某类条件时，**整条或该分支降级为未支持**。

### 10.6 规则成本、冲突与优先级：不能“谁先命中就算谁”

- 规则执行先按低成本、可限制的 BOF 检查筛选候选，再按需做有界 EOF/目录/流结构探测；签名匹配**只产生候选**，不自动决定 `format_id`。
- 对 ZIP、CFB、RIFF、EBML、ISO BMFF、ID3/MP3 等共享容器或家族签名，必须走 §4/§5 或 P1-B 对应验证路线。短签名与嵌入式签名位置不得跨越容器层级误判外层格式。
- 同时命中多个候选时，比较可证明的结构证据和特定/通用规则关系；没有完整消歧证据就保留 `candidates`、`container_only` 或 `unknown`，而非靠规则文件顺序、数据库 popularity、文件扩展名或看似“高分”强选。
- 支持 EOF 相对规则时先获取受控长度；使用有界 `Seek` 并计算整型溢出；设置最大偏移/窗口；不进行无界全文搜索或无限跳转。
- **规则库并非真值本身**。第三方规则之间冲突、错误或过度宽松时记录 `source_rule_disagreement` 等内部问题，优先返回低证据层级并设负向测试；不把第三方名称直接绑定为本项目的稳定 ID。

### 10.7 复用部分源码、规则与样本的法律/供应链门槛

**必须分别审查四种资产**：第三方程序源码、第三方规则数据库/转换后数据、格式规范/文档、样本文件（以及 Kaitai 的独立 `.ksy`）。仅因资料公开可下载、仓库开源或格式数据可访问，**不代表可直接复制并随可执行文件再分发**。

| 复用方式 | 必须留下的证据 |
|---|---|
| Cargo 直接依赖 | crate 版本/commit、feature 与依赖闭包、对应许可证文本、公告/NOTICE 要求、静态链接和平台影响 |
| 复制或改写少量源码 | 原项目/文件路径、commit、作者和版权声明、许可条款、改动摘要、保留声明方式；不兼容/不明确时不复制 |
| 导入/派生文件头或规则 | 原规则集版本、规则 ID/位置、数据许可、署名/来源和修改记录、派生规则清单、再分发许可确认 |
| 复用测试文件 | 来源、真实性、授权、SHA256、是否可提交公共仓库；未知授权的文件只在本地测试，不进发行物 |

注意 `pure-magic` 文档显示双许可 GPL-3.0 / BSD-2-Clause，`magic-db` 声称内置基于 libmagic 的规则；**不得据此跳过对具体数据来源及依赖许可的核验**。Kaitai 格式仓库明确每份 `.ksy` 独立许可。[R7][R8][R9]

P0 决策值建议对**库、导入规则和复制代码**分别给：`accept / replace / defer`，并增加 `reference_only`（可以查阅不复制）。许可证存在疑问时默认 `defer`。以上属于工程审核清单，**不构成法律意见**。

### 10.8 复用决策表与可复现研究产物

建议在 P0 生成三张互相关联的清单：

- `research/third_party_matrix.md`：项目、仓库/URL、语言、版本、许可证/特性、`Read+Seek` 可控性、分配/预算风险、样本误识别、可复用模块、决策。
- `research/signature_sources.md`：数据源、链接、规则格式、版本、下载哈希、许可/署名、字段类型、适用格式、实际导入条数、被拒条数及原因。
- `research/rules-manifest.*`：每条进入编译产物的规则 ID、外部规则引用、已知共享候选、成本边界、正反用例 ID、最终审核者/状态。

**完成定义**不是“从网上找到了几千个文件头”，而是：能回答**这条规则从哪里来、为什么能区分该格式、匹配失败如何解释、怎样防伪、读取多少数据、能否合法发布、如何复测**。

## 11. 测试矩阵、指标口径及阶段验收

### 11.1 每一阶段共享的功能和负向矩阵

1. **后缀独立性**：已支持的普通格式使用正确、缺失、未知、错误但已登记、大小写混合后缀，内容身份与证据一致；JPEG 的 jpg/jpeg 是同义关系。
2. **ZIP/OOXML**：真 DOCX、普通 ZIP、仅伪造 `word/document.xml` 的 ZIP、带部分 OOXML 迹象但缺关系的 ZIP、DOCM、XLSX、PPTX、模板/未知 Office 变体不得混淆；P1-B 未支持的变体要降级。ZIP64/加密/分卷按支持矩阵处理。
3. **CFB/DOC**：真 DOC、普通 CFB、Excel/PowerPoint CFB、伪 `WordDocument` 流、伪空流、FIB 截断、加密 DOC、加密 OOXML、旧 Word。无法覆盖的情况诚实报告。
4. **媒体**（在相应格式纳入时）：WAV/AVI/其他 RIFF、带 ID3 但没有 MP3 帧证据、EBML 已知/未知 DocType、ISO BMFF 已知/未知品牌。
5. **伪造与截断**：空输入、随机数据、签名不完整、只有签名没有必要结构、损坏头/目录、重复关键成员、路径规范化冲突。
6. **对抗预算**：ZIP 炸弹形态 XML、巨量成员、超长名称、伪尺寸/偏移、CFB 环链、恶意 XML；预算触发必须带诊断，不崩溃/无限循环。
7. **文件与平台**：中文/空格/前导连字符路径、Unix 非 UTF-8（适用平台）、目录、设备、不可读、运行期间文件消失/变化等。真实 I/O 错误不可解释为正常 `unknown`。
8. **接口一致性**：同一内容的 path/bytes/reader 结论一致；错误扩展名不影响内容；库无 stdout/stderr 副作用；原始输入保持不变。
9. **并发/隔离**：多次、并发独立调用不串数据；不产生解压文件/缓存文件，不自动联网。
10. **规则数据库专项**：外部规则存在短/重叠签名、EOF 规则、非固定偏移、通配符/优先级、容器规则与多个格式共享签名时，必须拒绝过度简化；导入器不能把不支持的子语法静默变成低条件匹配。规则乱序、不同主机 MIME 设置、伪造后缀不能影响内容结论。
11. **报告状态**：§6.2 中所有允许组合各有正向用例；禁止组合被测试拒绝。`format+limited` 与“身份必需步骤不足而降级”需各有测试；CLI 退出码与 JSON 对应。

### 11.2 P0 硬门槛（路线验证通过的必要条件）

| 编号 | 通过条件 | 证据 |
|---|---|---|
| P0-G0 | 至少完成现成库/工具及签名数据库的有版本记录的对照；为候选代码、规则及许可分别决定 `accept/replace/defer/reference_only`，不把未审查资产纳入发行 | `research/candidates.md`、规则来源清单、原始对照记录与决策表 |
| P0-G1 | 真 DOCX 与普通 ZIP、伪 OOXML ZIP 不混淆；真 DOC 与普通 CFB、伪 Word 流不混淆 | 固定正反样本、结构证据与逐例报告 |
| P0-G2 | **固定的 P0 支持范围测试集中，错误地确认具体格式的数量为 0** | 逐例误识别统计；这不是所有现实输入的零误判保证 |
| P0-G3 | 触发预算、截断、特性不支持、结构冲突时正确降级并输出可解释诊断 | 原始 JSON、预算触发日志和退出码 |
| P0-G4 | 至少 path/bytes/reader 共用核心；错误后缀不改变内容判断；无写入副作用 | 回归测试结果 |
| P0-G5 | 至少一次**真实 release 构建**与样本运行，不仅是 `cargo check`；测量读取、耗时、可测内存、二进制大小 | 构建命令、环境、产物大小与原始结果 |
| P0-G6 | 有依赖/规则数据/局部源码的独立复用决策和当前未支持格式/平台清单 | §10.1、§10.7–10.8 决策记录、覆盖表 |

**指标分别统计，不得混算**：

- **误识别（false identification）**：报告 `identification=format`，但经独立真值核对与真实格式冲突。P0-G2 是针对固定测试集的阻断门槛。
- **漏识别/覆盖不足**：预期属于当前支持范围，却给 `container_only` / `unknown` / `limited`；必须解释原因，**不能算格式识别 PASS**，但也不等于错误确认。
- **正确降级**：样本确有不足证据或明确超出当前覆盖时，返回预期的 `container_only` / `unknown` / `limited` / `ambiguous`。按预期矩阵判 PASS。
- **崩溃/超时/失控分配**：独立失败类别，永远不能被合并进“未知格式”。

P0 未达门槛时，不强行进入 P1-A；应先修复或减少承诺的格式范围。通过 P0 仅代表最小识别路线和测试集成立，不代表通用准确率已被统计证明。

### 11.3 P1-A / P1-B 验收

**P1-A（最小可发布）**：

- 嵌入规则均可追溯来源、许可与固定版本，导入/生成不联网，可重现生成哈希；不存在未经审核的“大规则库直接打包”。

- JSON `schema_version=1`、字段合法组合、稳定诊断 code、结构化 I/O 错误与 CLI 退出码完整测试。
- 所有已宣布支持的格式拥有正/反样本；原始字节不变，资源预算可配置且默认值经 P0 校准；CLI 和库复用核心。
- 有格式能力表、`Cargo.lock`、依赖许可证与版本、用户说明、已知限制、回归命令、真实 release 度量。
- 在至少一个目标平台上完整验证；未验证平台明确列出。P1-A 不因 P1-B 格式尚未支持而失败。

**P1-B（逐格式增加）**：

- 每新增格式重新完成同等正/反/预算/后缀/平台相关测试，且不能破坏 P1-A 已锁定语义。
- DOCM/XLSX/XLSM/PPTX/PPTM 等 OOXML、RAR/7z、音视频只在实测通过后标为支持；媒体、旧 Word、加密格式的不足分别披露。
- 按真实需求推进 Windows/Linux/macOS 验证；某平台未测即标记未验证，不隐含“完全跨平台”。
- 扩展新的文件头来源时，先跑 §10.5 流水线与许可审核，防止新增规则破坏原有误识别/预算门槛。

### 11.4 Toss 现有样本的可选使用

本机已有公开来源和合成样本可用于离线验证，但不代表独立工具可自动获取或依赖绝对路径：

- DOC：`.sandbox-test/doc-corpus-20261030/`，53 份；来源、分组和哈希见 `MANIFEST.json`。
- DOCX：`.sandbox-test/docx-corpus-20261027/`，150 份公开来源样本；另有现成合成边界夹具。
- 压缩包、图片、媒体：`tests/corpus/`，按实际存在样本选择并核对来源。

以上路径均**相对 Toss 工程**。独立项目只能读取维护者明确提供或授权的文件；不得复制整库、提交私人文件、将本机绝对路径作为测试依赖。新增样本须记录来源、版本/提交、许可证及 SHA256，不自动把所有语料提交到仓库。

DOC 回归中的“正常读取”不是格式识别的真值；需以可追溯来源、容器证据及独立工具/人工复核判真。53 份 DOC 中有加密和更早 Word 类型，不能全部标为 Word97+。

### 11.5 验收报告必须可复验

每个样本保存：来源 ID/SHA256、预期结果及依据、实际报告、运行命令、退出码、预算配置、来源读取统计、耗时和可测内存。`supported`/`container_only`/`not_supported`/`not_tested` 必须区分。

报告单列**实现者自测、独立复验、维护者人工验收**，且给出复跑脚本/命令、平台、工具链及未测试范围。测试进程的 OS 内存/时间护栏与产品自身能力必须分开表述；不得宣布“所有文件安全”“不会误判”“完全跨平台”。

## 12. Toss 未来接入约定（仅供参考）

P0/P1-A/P1-B 阶段均不实现 Toss 接入，不改变现有默认行为。未来由 Toss 自己完成适配：

1. 调用独立 Rust 库，按具体格式映射图片、媒体、压缩包、Word、PDF 处理器。
2. DOC/DOCX 的解析器选择应依据真实识别结果，而非再次查看后缀。
3. 已达到本工具确认门槛的 DOCX 可优先走 Word；只确认 ZIP 的文件仍不能默认当 Word。
4. 已见 OOXML 候选却因预算/歧义无法确认时，Toss 需专门决定“提示用户”还是“容器操作”；不能仅因 fallback=ZIP 就静默自动解压。
5. 识别错误、内容截断、I/O 失败不应伪装成后缀兜底成功。是否保留扩展名提示是 Toss 的产品决策，必须区分“提示”与“内容已确认”。
6. 识别报告不代表处理能力：识别 XLSX、DOCM、受保护 DOC，不等于 Toss 支持打开、转换或解密。
7. 库与处理器尽量使用同一份已打开输入；若接口必须重新打开，应承认检查与使用之间的变化风险，不把识别当安全校验。
8. 保留单 EXE、拖拽默认动作、终端阅读和未来 JSON 导出方向；独立工具不直接接管这些功能。

## 13. 给独立开发窗口的首轮交付清单

**首轮只验 P0，不要求一口气完成 P1-A/P1-B。先有对照，再实现；调研必须产出可以复测的结果。**

- [ ] 独立仓库、最小 Rust 库与 CLI、锁文件；无 Toss 私有类型或修改。
- [ ] 先运行 `file-format` / `infer` 等代表性库及 DROID/Siegfried/`file` 至少一项独立对照工具，固定版本/配置，保存同样本结果及误确认清单。
- [ ] 审核 PRONOM/DROID 等文件头数据库，保存来源 URL/版本/哈希/许可、可用规则种类、不能直接导入的复杂规则和精选候选白名单。
- [ ] 对复用第三方**代码、规则数据、测试样本**分别核验再分发条件；决策 `wrap / reuse_part / reference_only / replace / defer` 并附理由。
- [ ] PNG/PDF 等最小签名识别，及 ZIP vs DOCX、CFB vs DOC 的**真实可执行原型**和负向样本。
- [ ] 明确“普通 ZIP”与“疑似 OOXML 未确认”的报告区别；截断/预算/恶意结构得到合理降级。
- [ ] 可约束的来源读取、目录/名称、XML/解压预算，提供真实触发案例与已知内存/时间风险。
- [ ] 内容识别与后缀诊断分离，path/bytes/reader 结果一致；至少有一种可机读 JSON 输出。
- [ ] 依赖/feature/许可证/版本/间接依赖清单，`accept/replace/defer` 取舍理由。
- [ ] 真实 release 构建大小、读取量、耗时、可测内存，附工具链/环境/命令与原始日志索引。
- [ ] P0-G0 至 P0-G6 验收表逐项说明 PASS/FAIL/NOT_TESTED；列出误识别、漏识别、正确降级与崩溃的不同数量。
- [ ] 下一步只根据**真实失败案例**决定补修 P0、进入 P1-A，或推迟某类格式；不提前开发 GUI、重命名器或 Toss 集成。

> **最终原则：先找并测试成熟实现与可信文件签名数据源；能复用的合法复用，不能受控的只作参考；再证明“内容识别比后缀更能正确分流，而且不会为了识别就无界解析文件”。通过 P0，再把 API/JSON 做成稳定产品，最后按证据扩充格式。**

## 14. v1.2 修订记录与待决事项

### 继承 v1.1、在本版继续有效的规格

1. 原 P1 拆为 **P1-A 最小可发布版**与 **P1-B 格式/平台扩展**；P2 继续需求驱动。
2. 明确普通 ZIP 与疑似 OOXML 证据不足时的不同结论，禁止用“回退 ZIP”掩盖未完成的 Office 判断。
3. 为 `identification` × `inspection` 建立合法组合、报告字段非空约束、扩展名优先级及退出码对应。
4. 明确 `format+limited` 的适用条件；禁止 `format+ambiguous`，区别身份必需检查与非必需检查。
5. 定义预算触发稳定 code、源读取计数口径和解析器内存风险边界；不把进程护栏冒充库特性。
6. 依赖准入必须形成 `accept/replace/defer` 决策；P0 固定样本误确认数为零，且单列漏识别和正确降级。

### v1.2 新增的规格变化

1. 将“**先实测现成方案，再决定封装、局部复用、仅参考或自研**”写入 P0 前置研究流程；增加库/工具候选矩阵、统一样本对照和版本记录。
2. 增加 PRONOM/DROID、libmagic、freedesktop shared-mime-info、Kaitai、SEARCH/GCK、Siegfried 等**文件头/结构资源**的来源目录、适用范围及限制。
3. 新增“固定来源 → 受限解析 → 保留全部限定条件 → 白名单审核 → 静态生成 → 正/负样本测试”的规则导入流水线；默认不在运行时联网下载数据库。
4. 新增**规则执行预算**、短签名/歧义/多候选处理、候选与具体格式确认的证据隔离。
5. 第三方**代码、规则数据、格式说明、测试样本**分开做许可与来源核验，明确内置/移植不等于阅读参考；新增 P0-G0 调研与决策门槛。
6. 文档所列项目版本、许可线索和 URL 是研究入口，不代表依赖已选定或性能、安全、合法再分发已实际验收。

### 仍由 P0 真实验证决定，不能预先宣称完成

- 具体 crate、版本、feature、许可证兼容性和静态链接行为；是否真的比自有小型规则实现更划算。
- PRONOM/libmagic 等数据源具体规则是否可满足本项目预算/证据要求，实际允许采用与分发的条目数量及许可措施；是否需要制作可复现的规则生成器。
- ZIP64、OOXML Strict/Transitional、加密包/旧 Word 的实际支持深度。
- 检查时的最小 FIB 版本/字段条件和证据规则；需要查证格式规范并固化负向测试。
- 预算数值是否适合大目录/超大可寻址文件；P0 后可有记录地调整默认值。
- 稳定格式注册表、诊断码精确拼写、CLI/库公开类型的最终命名及跨平台实测状态。


## 15. 公开资料与调研来源（2026-10-31 查阅；非项目已实测证明）

> 本节列出 v1.2 **新增的外部调研入口**。原始 v1.1 的产品需求、样本说明、技术约束仍以 §0–14 为准。公开网页或许可证说明可能更新，实施时须重新核对并固定具体 release/commit 与检索日期。所有第三方资产的实际复制、分发、嵌入权利均须重新检查。

| 编号 | 来源与具体入口 | 在本规格中的用途 |
|---|---|---|
| R1 | [英国国家档案馆 PRONOM — About / Accessing data](https://pronom.nationalarchives.gov.uk/about) | 格式元数据、PUID、JSON/XML 与二进制/容器签名获取入口 |
| R2 | [PRONOM — 历史 DROID 签名文件](https://pronom.nationalarchives.gov.uk/signature-list) | 可固定签名集版本，独立保存下载哈希，比较变更 |
| R3 | [PRONOM_Research — Licensing 说明](https://github.com/digital-preservation/PRONOM_Research) | 提及 PRONOM 数据为 OGL 2.0；仍要验证本次获取的具体资源条款 |
| R4 | [`file` / `libmagic` 源码库](https://github.com/file/file)；[magic 规则格式说明](https://github.com/file/file/blob/master/doc/magic.man) | Magic 文件规则、继续匹配、偏移/间接读取约束研究 |
| R5 | [freedesktop shared MIME-info Specification](https://specifications.freedesktop.org/shared-mime-info/latest/) | MIME 与 magic/glob 规则语义；明确 glob 不用于内容身份认证 |
| R6 | [Kaitai Struct 格式仓库](https://github.com/kaitai-io/kaitai_struct_formats) | `.ksy` 结构描述与各文件单独许可的入口 |
| R7 | [Siegfried](https://github.com/richardlehane/siegfried) | PRONOM/freedesktop 等签名独立对照；文档明确默认签名可能扫描全文件 |
| R8 | [`pure-magic` Rust 文档](https://docs.rs/pure-magic/latest/pure_magic/) | Rust 规则执行路线及文档所列 GPL-3.0/BSD-2-Clause 双许可线索 |
| R9 | [`magic-db` Rust 文档](https://docs.rs/magic-db/latest/magic_db/) | 将修改后的 libmagic 规则内置到 Rust 二进制的现成方向；许可链待核验 |
| R10 | [`file-format` Rust 文档](https://docs.rs/file-format/latest/file_format/) | 多格式探测与可选 reader feature 基线 |
| R11 | [`infer` Rust 文档](https://docs.rs/infer/latest/infer/) | 轻量签名识别基线 |
| R12 | [`zip`](https://docs.rs/zip/latest/zip/)、[`cfb`](https://docs.rs/cfb/latest/cfb/)、[`quick-xml`](https://docs.rs/quick-xml/latest/quick_xml/)、[`ooxml-opc`](https://docs.rs/ooxml-opc/latest/ooxml_opc/) | 各容器/OPC/结构层候选库，需针对预算实测 |
| R13 | [SEARCH / GCK File Signatures](https://filesig.search.org/about) | 人工查阅各种签名，不视为可自由抓取分发 |
| R14 | [DROID 官方介绍与下载](https://www.nationalarchives.gov.uk/information-management/manage-information/preserving-digital-records/droid/) | 另一实现路线的 CLI 识别对照、PRONOM 规则背景 |
| R15 | [Microsoft MS-DOC 文档](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-doc/acb08ace-f268-4879-a998-de592103a70e) | DOC FIB 最小结构验证的官方规范入口 |

**已核实的是这些来源的公开定位与可见说明；未核实的是它们在项目目标平台/预算下的实际表现、每个候选库的许可证闭包、全部规则条目的版权及可分发性。** 前者是调研结果，后者必须在 P0 做工程验证。
