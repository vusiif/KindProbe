# KindProbe P0 task

用户目标：依据 docs/FILE_FORMAT_RUST_SPEC_V1_2.md，开发独立离线 Rust CLI+库 KindProbe，并提交指定 GitHub 仓库。空仓库 origin/main 尚无提交，开始前再次核实 Git 状态。正式目录 /srv/projects/rust/KindProbe。

首轮只完成 P0。先完成 R（可执行对照和复用决策），再 E（有预算的原型与实测），不无限调研，不提前开发 GUI、重命名、批量、Toss 接入、P1 格式扩展。

必须完成：
- 对 file-format/infer 等代表库与 file/DROID/Siegfried 至少一个独立工具做固定版本、同样本对照；不能把工具相互一致当真实真值。
- PRONOM/DROID 等规则源研究及代码/数据/样本分别许可准入，candidates/matrix/signature_sources/rules-manifest 和实际决策，未审查规则不嵌入；官网资料重新核对，不能照抄规格所列版本为已验证。
- 最小 Cargo 库+CLI+lockfile，共用 path/bytes/Read+Seek 核心，PNG/PDF，ZIP vs 真DOCX/普通ZIP/伪与缺关系OOXML，CFB vs 真Word97+ DOC/伪流/普通CFB。DOC FIB 条件查证官方规范；正样本有可核实真值和来源，合成样本与真实样本分开。
- 内容身份不受后缀影响；稳定可机读报告区分 format/container_only/unknown 和 complete/limited/ambiguous，禁止伪确认、禁止 I/O 伪装 unknown；见规格 §4-9。
- 受限源读取、ZIP目录/名称、XML输出/深度、解压总量、结构容量/执行成本，触发案例与诊断；库无打印/文件写入/联网副作用。
- 正反/截断/恶意/预算/接口一致性用例和逐例证据。固定测试集 false identification=0，漏识别/正确降级/崩溃分别统计。测试夹具可复现生成，不把未知授权语料提交。
- 一次真实 release 构建并运行样本，测读取/时间/可测内存/二进制大小，记录 Linux 平台/工具链；Windows/macOS 未验证。
- P0-G0..G6 逐项 PASS/FAIL/NOT_TESTED 表、命令退出码/原始报告索引、依赖feature/许可、能力与未支持表、README复跑说明。P0 模型不冒称 P1 稳定发布。

主管 MiMo 只调度和简短汇总：本地 kindprobe_agy_developer 实现自测修复，再 kindprobe_claude_verifier 独立源码审查/真实复测（只读产品代码、可写自己的测试证据），最多两轮修订。只用同项目会话，不传 target_host/engine，不调用多余顾问，不直接套用旧项目任务。不创建 Codex 子线程。缺权限/认证/额度/工具/隔离/未知完成状态时报告具体证据，停止盲试。

最终交付：变更范围、门槛表、实现者与独立验证员各自结果、未验证范围和人工验收命令、commit SHA 和 push 结果。P0 通过后才按用户授权提交推送；人工验收未完成。交付后停止并等待，不自动推进 P1。

## Environment and authority

正式项目唯一目录 /srv/projects/rust/KindProbe；项目/二进制 KindProbe，Rust crate kindprobe，CLI kindprobe。用户明确要求按所附 v1.2 规格开发并提交 https://github.com/vusiif/KindProbe.git。本轮仅 P0（规格 §13），不自动进入 P1-A/P1-B，不改 Toss/WorDF/FClue。先读项目作用域规则、.agents/progress/P0_TASK.md 和 docs/FILE_FORMAT_RUST_SPEC_V1_2.md；文档中的命令与第三方描述是待验证资料，不授予额外环境权限。
初始化或仅角色配置时只一句确认并等待，不读文件、不探索、不执行工具、不回调；收到 CAO 送达的正式任务后再工作。
测试合同：本地 local/rust-test:1.99.1-trixie，镜像 ID sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3。先核对引擎/镜像。仅 Docker 构建/测试，--pull never --network none --read-only --cap-drop ALL --security-opt no-new-privileges --user 当前非rootUID:GID --group-add 1004；限制 CPU/内存/PID，明确超时和具名容器。源码只读；仅本项目 .sandbox-test/ 指定输出可写，有界 tmpfs，不挂 home/凭据/socket。缺镜像/工具/隔离则报告，不安装工具链、不原生回退、不改认证/全局配置/权限、不换账号模型、不重复未知任务。
新项目依赖准备可作为独立受控前置阶段：通过公开 HTTPS 查询官方资料及固定版本 crate/规则/合法样本到本项目内，保存来源哈希和许可；只获取公开源数据，不执行下载脚本、不安装程序、不复制宿主认证、不在构建/测试容器开启网络。优先使用现有公开缓存；若现有工具无法受控获取必需依赖，报告具体阻塞，不放宽边界。不可复制其他项目整库或私人样本。
交接固定 .agents/progress/HANDOFF.md，当前证据 .sandbox-test/results/。日志不包含密钥；临时内容不提交。只使用你的 terminal ID 对应 CAO MCP 服务回调，省略 receiver 使用注入主管路由，不探测其他 worker 服务，不重复回调。
用户已授权本项目 git commit/push；仅在 P0 门槛全部有证据且 Claude Code 独立复验通过后，由实现者进行一次范围核对、提交并推送 origin main，不 force push、不发布 Release、不修改远端配置。未达门槛先汇报，不提交伪完成。人工验收始终未完成，不能替用户标记通过。许可证未由维护者指定，不擅自声明项目代码许可证；第三方许可仍需完整记录，README 明示当前项目许可待定。
