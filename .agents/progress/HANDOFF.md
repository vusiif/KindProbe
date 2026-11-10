## License decision (2026-11-09, America/Los_Angeles)

- Maintainer explicitly selected **GPL-3.0-or-later**. Added the complete unmodified GNU GPL v3 license text in `LICENSE`, set Cargo package SPDX metadata and updated the active README declaration. Earlier pending-license statements below describe historical checkpoints and are superseded by this decision. Third-party notices are unchanged.
- This is a license/metadata update only; P1-A implementation and verification remain complete, human acceptance is still pending, and P1-B is not authorized. Validation: TOML parses with the exact SPDX expression, LICENSE matches the local canonical GNU GPL v3 text byte-for-byte, and `git diff --check` passes. No product test rerun is needed for unchanged source.

## Current checkpoint (2026-11-08 主管 5beee9de 接续, Round1 复验 PASS → 提交推送)

- **独立复验 PASS（已核证据在盘）**：验证员 `181d8896`（kindprobe_agy_verifier 独立会话）逐项复验 5 项修复全 PASS、0 缺陷（reproduced=0/inferred=0）：①fcMin/fcMac 移除依据成立（MS-DOC 2.5.1 FibBase 32 字节、24..32 reserved MUST be ignored；夹具实测退出码 3 container_only 非假绿）；③表流真实路径+大写 /0TABLE 测试 PASS；②256KiB 恰好边界 1 字节 probe 触发测试 PASS；④归一化依据成立（ECMA-376 Part 2 §10.1.1.4/§10.1.2.3，仅 part_name/ext 归一、ContentType 原样，非全量小写）；⑤--json 预扫描 ErrorEnvelope（遇 `--` 停止）前后序实测 PASS；4 项新增触发测试非同义反复。
- **验证员亲自 Docker 复跑（全部退出码 0）**：cargo test 30/30、doctest、release 产物 912,408B / SHA256 `8665bd5efe4ce7ae3ae9fe1569fc84a966f7f2251f704ff96630df967b911c07`（与预期 100% 一致）、52 矩阵 52/52 false=0、Codex 18/18、release 度量 23/23。镜像 sha256 `cdf54f8f…` 与合同一致，参数含 --network none/只读/降权。
- **主管证据抽查（在盘，.sandbox-test/results/verifier_round1/）**：`INDEPENDENT_VERIFICATION_P1A_ROUND1.md`、cargo_test.log（各二进制 test result: ok）、doctest.log、52_fixtures_regression_results.json（total 52/false 0/misses 0/crashes 0/degradations 30）、run_acceptance.log（PASS 18/18）、release_measurements.json、产物 SHA 独立复算一致。`.sandbox-test/` 按 .gitignore 不入库。
- **修订预算**：Round1 消耗 1/2，剩余 1 轮。
- **提交推送完成**：`0b55e49`（58 文件，+1556/−133）已推送 `origin main`（`892a40b..0b55e49`），一次提交，无 force、无 Release、未改远端配置。工作区干净。
- **旧验证员**：`4e7658be` tmux session 已消失（record unknown），未复用、未新造已通过证据。

- **新主管**：OMP 主管 `5beee9de` 接替 `567bae8e`（历史保留）；实现者回调已迁至 `receiver_id=5beee9de`。
- **实现者**：`a618e7ea` / native `77e21659-f414-4cd9-b58d-f51df5cd8354` 同进程经用户新指示重启，启动参数现为适配器既有 `--dangerously-skip-permissions`（不再是 --mode auto），未改全局设置、未新建 native 对话；Docker 测试合同不变。worker `completed`、pending 0。
- **Round1 交付报告已收**（inbox message `163`，sender a618e7ea）：5 项审查项全闭环（①fcMin/fcMac 按 MS-DOC FibBase 32 字节布局判定为无效检查并移除；②256KiB 恰好边界 1 字节 probe；③CFB 表流按真实匹配路径打开；④OPC Content_Types 按 ECMA-376 Part 2 §10.1.2.3 归一化；⑤--json 预扫描输出 ErrorEnvelope），自报 5 条回归全部退出码 0（cargo test 30、doctest、52 矩阵 false=0、Codex 18/18、release 23 样本 912,408B / SHA256 `8665bd5efe4ce7ae3ae9fe1569fc84a966f7f2251f704ff96630df967b911c07`），预算 1/2。**自报不作复验通过。**
- **主管证据抽查（在盘）**：`52_fixtures_p1a_results.json`（total 52/false 0/misses 0/crashes 0/degradations 30）、`release_measurements.json`、产物 912,408B、`codex-acceptance-run-20261108T153237/` 均存在。
- **独立复验已派并已回**：本地 assign `kindprobe_agy_verifier` → worker **`181d8896`**（旧 `4e7658be` tmux session 已消失，record unknown，不复用不新造证据）；结论见上（PASS）。
- **权限处置**：实现者侧因用户新指示已配 skip-permissions，不再逐条代批。Docker WSL 集成已恢复（/usr/bin/docker 与 /var/run/docker.sock 在，Ubuntu-26.04 集成开启），测试仅走批准合同。

## Prior recovery checkpoint (2026-11-08 20:37, Asia/Shanghai)

- **P1-A 接续已开工（实现中）**。原实现者已按人类澄清以普通权限恢复：tmux session `cao-kindprobe-resume-20261101` / window `kindprobe_agy_developer-7d7b` 重建，原 native conversation `77e21659-f414-4cd9-b58d-f51df5cd8354` 经 `agy --conversation … --model 'Gemini 3.8 Flash (Medium)'` 恢复（无 skip-permissions），进程存活，工作目录 /srv/projects/rust/KindProbe，代理 8 项注入。
- **回调路由迁移**：`~/.gemini/config/mcp_config.json` 仅 `cao-mcp-server-a618e7ea` entry 增加 `CAO_CALLBACK_TERMINAL_ID=567bae8e`（备份 `mcp_config.json.bak-20261108-recovery`）；已核验新 agy 进程的 MCP 子进程同时带 `CAO_TERMINAL_ID=a618e7ea` 与该回调变量。未改其他 entry/凭据/权限/数据库。
- **任务送达**：P1A 四步接续指令经 `POST /terminals/a618e7ea/inbox/messages`（sender 567bae8e）送达，message_id 152，delivered；worker 已开始执行（两次单次批准只读 `git status/diff`，未持久 allow）。实现者回调目标：`send_message receiver_id=567bae8e`。
- **prewarm**：`a618e7ea` pane_alive=true、status=idle、pipe=true；`0eb85957` 为 pane_missing（旧主管，不复活，由 567bae8e 替代）。
- **现场**：HEAD `892a40b`；保留 `src/lib.rs`、`tests/test_cli_contract.rs`、`P1A_TASK.md`（未跟踪）改动；Docker 镜像 `local/rust-test:1.99.1-trixie` digest 一致，无运行容器。
- **监督唤醒句柄（实测状态，2026-11-08 21:30 人类纠正后措辞）**：**定时监督尚未部署**——无已验证的"22:00 到点自动唤醒主管"调度句柄，cron/at 不可用，systemd timer 未配置本任务。worker 回调（inbox 投递）**仅在 worker 主动发送时**唤醒我，不等于定时自动核查；到点核查目前依赖人类催促或我在回合内主动查看 pane。
- **权限处置纪律（人类纠正）**：以真实 pane 画面为准处理权限框；`Running command` 标签**不代表**无权限请求，读屏漏框由人类代批过一次（mkdir），后续每轮先看 pane 尾部是否有 `Requesting permission`。
- **回归规则（人类纠正）**：新增/修改实现后，相关回归与全量必要检查**必须按 Docker 合同运行**；旧「不重跑已通过测试」仅指**无改动时**不重复无关测试，**不可据此跳过变更后的回归**。
- **估时与检查点**：四步实施（含 D8 迁移 18 harness/29 夹具/52 矩阵入库）预计 **2–4 小时**（依据：缺口已批准、现场已核、但夹具迁移量大且受逐条权限审批与 503 风险拖慢）。**下一检查：2026-11-08 22:00 CST**（区间上沿）；未回调则主动核查 pane/inbox/容器；再现 503 即保存证据停止。
- 本轮未改产品源码、未跑产品测试、未提交推送。

## P1-A 实施完成 → 独立验证中（2026-11-09 06:30 前后）

- **实施者 `a618e7ea` 已回调交付四步实施**：句柄级 fstat（src/lib.rs）、字段级信封断言（tests/test_cli_contract.rs，新增覆盖 0/2/3/4）、D8 迁移（fixtures×3 + manifest + tests/codex_acceptance + scripts 两脚本）、README 重构、release 度量口径修正。自报：cargo test 27/27、doctest 0、52 夹具 False ID=0、Codex 18/18、release 909,616B SHA256 `cc4e9e87…`、RSS 未测、修订预算 0/2 未消耗。
- **主管现场核对**：git status 与自报清单一致（5 修改 + 新增 fixtures/tests/scripts 目录），证据文件 `.sandbox-test/results/codex-acceptance-run-20261108T142139/`、`release_measurements.json` 等在盘；无运行容器；worker status=completed。
- **独立验证已派**：本地 assign `kindprobe_agy_verifier` → terminal **`4e7658be`**（独立会话，只读产品源码，证据写 `.sandbox-test/results/verifier_p1a/`），要求亲自复跑 cargo test/doctest/52 矩阵/Codex 18 项并逐项核对 D3 路径回显、D8 普通 checkout、README 诚实性。**等待其回调；两轮修订预算 0/2 待用。**
- **独立验证结论（已核证据落盘）**：**PASS，0 缺陷，0 不符点，修订预算 0/2 未消耗**。验证员亲自复测：cargo test 27/27、doctest 0、52 矩阵 False ID=0、Codex 18/18、release 909,616B/`cc4e9e87…`/RSS None；12 份日志/JSON 在 `.sandbox-test/results/verifier_p1a/`（主管 ls 核实存在）。D3 路径回显结论成立（DetectionReport 无路径字段，路径仅在 CLI 信封回显用户输入）。
- **阶段末整体审查已派**：9890 `agy_whole_reviewer` → remote terminal **`86b4e695`** / session **`cao-f37716cc`**（health ok，lolicon 属 devprojects 组可读写报告路径）。只读产品源码，仅写 `.agents/progress/FINAL_REVIEW.md`，不构建/测试/联网。等待其回调。
- **整体审查已回调（PASS 带 5 项发现，报告已写 FINAL_REVIEW.md，基线 892a40b+工作区）**：①[中] cfb_doc.rs:260-271 fcMin/fcMac 读 [32..40] 疑应为 [24..32] 且夹具生成器同错位疑似假绿 ②[中/低] DecompressLimitReader 恰好 256KiB EOF 误报 xml_output_exceeded ③[低] CFB 表流 case-insensitive 匹配后硬编码路径打开 ④[低] OPC Content_Types 大小写敏感 HashMap ⑤[契约] --json 下参数错误(exit 2)无 ErrorEnvelope。主管初核：5 处源码位置均属实存在（①的偏移对错需按规格核查，P0 历史有「fcMin/fcMac 字段解释未被采信、该区域 reserved」记录，已要求先核规格再改，不盲改）。
- **修复 Round 1/2 已派**（message_id 160，预算 1/2 即将消耗）：5 项逐条处理+边界触发测试+全量 Docker 合同回归；worker 已开始读 cfb_doc.rs。第 1 项若核查判定现行为合理须给依据。
- **有界化指令（message 161，仍在 Round 1 内）**：人类纠正两点——①worker 第①项已无界搜索 43 次、源码零新增改动，且主管两次以 `Searching web`/`Running command` 误报「无待批框」（实际各有权限框，人类代批 2 次：`grep invalid_fib_offsets`、`grep adv_fake…`）。指令：停止无界搜索，保存官方依据+未确认处；**第①项 fcMin/fcMac 与第④项 OPC 大小写列为争议项，单列不阻塞**（不机械平移、不把审查当确证、不一律转小写）；先修已明确的 ②③⑤；30 分钟内阶段性回报。**权限处置以 pane 当前尾部 `Requesting permission`/`Allow creation` 为准，每回合实际核查。**
- **有界检查点：2026-11-08 07:55 PST（30 分钟）核 worker 阶段回报**；定时监督未部署，此为我主动核查点，非自动唤醒承诺。阶段回报后：争议项交裁决，已修项进验证员复验 → 达标后 commit/push。



## Prior recovery checkpoint (2026-11-08 earlier, Asia/Shanghai)

- User requested trying the updated cao-workflow. Read SKILL.md, omp-runtime.md, startup-preflight.md and supervision-timeout.md. Current supervisor policy uses OMP; old OpenCode history is preserved, never represented as an OMP restore.
- Windows preflight passed (9888-9892, Clash 41096, HTTP). Main node currently lists only the unrelated Toss supervisor; no KindProbe session/worker or pending original execution was found. KindProbe prewarm is ready with no terminals. No session was deleted or created.
- Started the already installed, stopped Docker Desktop without changing settings. WSL Docker now responds, no running containers were listed, and approved image local/rust-test:1.99.1-trixie matches sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3. No product tests were run.
- Current blocker: actual native OMP is omp/18.18.1, while /srv/cao-team/omp-network/omp_gate.py explicitly permits only omp/18.18.0. The gate correctly stopped before model authentication; no model availability or generation success is claimed. No runtime/version pin modification or bypass was performed.
- Existing KindProbe source and installed OMP profiles remain, but ~/.config/cao-launcher/projects.json is absent, so generic helper network-check rejects KindProbe as unregistered. Direct existing-profile OMP --check reached the version blocker. Do not regenerate/overwrite the team to hide this registration issue.
- HEAD remains 892a40b; preserve partial src/lib.rs and tests/test_cli_contract.rs changes plus local task/handoff. No new product changes, commits or pushes in this recovery.
- Next: verify OMP 18.18.1 compatibility and update the deployed version contract under corresponding runtime authorization; then pass native auth/model gates, use the existing profile and API startup without initial_message, hand off P1-A explicitly, and resume the original agy conversation in normal permission mode. Preserve prior no-skip-permissions boundary. Timer/takeover implementation still unverified; do not claim automatic monitoring.

## Current recovery checkpoint (2026-11-07, Asia/Shanghai)

- User requested continuation under the updated cao-workflow. Windows port/proxy/HTTP preflight passed; both CAO nodes list no sessions, prewarm lists no terminals, and the icelolly CAO tmux server is absent. No original agy/cargo/rustc process was found; no task was redispatched.
- Original OpenCode native session `ses_e6858a8a6ffeJlMl3k7VxSJWpY` still exists at `/srv/projects/rust/KindProbe`. Preserve it; do not create a replacement native conversation. The unrelated live OpenCode process was left untouched.
- HEAD remains `892a40b`. Preserve existing uncommitted `src/lib.rs` (handle-based metadata check), `tests/test_cli_contract.rs` (model imports), HANDOFF and untracked P1A_TASK. No tests or review of these changes were performed in this recovery.
- Startup is blocked: native OpenCode is now v2.2.6; guard.py:118 assumes a mapping, while native `debug config` returns a two-item list. Gate reports AttributeError before model authentication; provider availability is NOT verified. No bypass or runtime modification was performed.
- Docker is unavailable: `/usr/bin/docker` links to absent `/mnt/wsl/docker-desktop/cli-tools/usr/bin/docker`; container state therefore cannot be verified. No native test fallback is authorized.
- New supervision-timeout skill was read. Timer/wakeup and temporary MiMo executor capabilities have not been deployed or verified. Do not claim automatic supervision/takeover is active.
- Next: obtain authorization for the concrete OpenCode 2.2.6 gate compatibility repair and restore/check existing Docker Desktop integration, then pass gates, resume original native sessions, reconcile existing partial edits and continue P1-A. Independent verification uses a separate local agy conversation; no Claude fallback. No P1-B or license selection.

## Session cleanup check (2026-11-05)

- Verified cleanup complete on 2026-11-05: stale CAO worker record `d9a85ac5` is absent from the terminal list and returns 404. Both retained KindProbe panes and terminal records remain present; all seven checked inboxes have no pending messages, no test containers are running, and node 9890 has no sessions. No additional terminal or session deletion was needed. Toss and WorDF sessions were preserved.

- Retired stale CAO worker record `d9a85ac5` (icelolly/9889, session `cao-kindprobe-resume-20261101`, old pane `kindprobe_agy_developer-f59e`). Its pane/native agy client is absent; no result, product change, pending message or running test container was found. Existing historical handoff records this failed attempt. Current replacement `a618e7ea` and supervisor `0eb85957` remain; their capacity blocker and all evidence are preserved. Only the stale CAO terminal record is reclaimed; native conversation history and product files remain.

# KindProbe P1-A Handoff Document

> ## 🟢 当前状态置顶 (2026-11-08 23:35, Asia/Shanghai) — **P1-A 修复 Round 1 完成，全量回归与 Codex 18 项全部 PASS**
>
> **阶段：spec v1.2 P1-A（最小可发布 CLI + Rust 库）—— 稳定 API/JSON v1/CLI/预算/错误与诊断契约**
> **实现者：`kindprobe_agy_developer` (`a618e7ea`)**
> **主管：`567bae8e`**
>
> ### A. 修复 Round 1 达成情况 (审查 5 项全量闭环)
> 1. **项 1（fcMin/fcMac 偏移规格核查与移除）**：
>    - 对照微软官方规范 [MS-DOC] 2.5.1 `FibBase` 结构，FibBase 严格为 32 字节（`0x00..0x20`）。偏移 24..32 处为 `reserved3`（2 字节）、`reserved4`（2 字节）与 `reserved5`（4 字节），规范明确定义 MUST be 0 and MUST be ignored；偏移 32 处为 `csw` 与 `fibRgW`。
>    - `fcMin`/`fcMac` 并不存在于 Word 97+ 的 `FibBase` 结构中（Word 97+ 采用 `FibRgLw97.cbMac` 与 `Plc` 结构）。旧审查推断要求将读取移至 24..32 是错误的，会读取保留字段产生误报。
>    - 判定该无效检查属于历史代码多余假设，已正式从 `cfb_doc.rs` 移除。
> 2. **项 2（DecompressLimitReader 恰好 256KiB 边界修复）**：
>    - 修复前当输入流恰好解压满 256KiB 并正常 EOF 时，后续探测读取因 `current_read >= max` 直接误报 `budget.xml_output_exceeded`。
>    - 修复：在 `current_read >= max` 时先进行 1 字节底层读取探测：若底层为 EOF (`Ok(0)`) 则返回正常 EOF (`Ok(0)`) 不报超限；若底层确实仍有后续字节，则准确报错 `budget.xml_output_exceeded`。
>    - 新增单测 `test_decompress_limit_reader_exact_and_overflow_boundaries` 验证恰好 256KiB 成功与 256KiB+1 报错。
> 3. **项 3（CFB 表流打开大小写敏感性修复）**：
>    - `cfb_doc.rs` 原先通过 `entry.name().eq_ignore_ascii_case` 匹配条目后，在 `open_stream` 时硬编码 `format!("/{}", required_table_name)`。
>    - 现改为记录 `entry.path().to_path_buf()`，使用真实匹配到的路径打开表流。
>    - 新增单测 `test_cfb_case_insensitive_table_stream_open` 覆盖 `/0TABLE` 大写路径。
> 4. **项 4（[Content_Types].xml OPC 规范大小写不敏感匹配）**：
>    - 依据 ECMA-376 Part 2 (OPC) §10.1.2.3，Extension 匹配不区分大小写，PartName 遵循 URI 比较规则。
>    - 现将 `parse_content_types` 中的 `Override PartName`、`Default Extension` 以及待查询键在存取时均归一化为 ASCII 小写。
>    - 新增单测 `test_ooxml_case_insensitive_content_types` 覆盖大写扩展名与大写部件名。
> 5. **项 5（CLI 参数错误在 --json 模式下输出完整 ErrorEnvelope）**：
>    - 对命令行参数做预扫描提取 `--json`，无论 `--json` 出现在非法参数前还是后，均保证以 JSON 格式输出 `{"schema_version":1,"error":{"code":"error.invalid_arguments","message":"..."}}`，退出码保持 `2`。
>    - 文本模式保持向 stderr 输出错误并退出码 2。
>    - 测试覆盖各类参数错误在 text/json 模式及不同参数顺序下的完整断言。
>
> ### B. 真实自测与全量回归实据（Docker 沙箱合同全部退出码 0）
> - **Cargo 离线全量测试**: **30 项测试全部 PASS**（原 27 项新增 3 项触发测试，0 failed, 0 ignored）。
> - **Cargo Doctest**: **PASS**（退出码 0）。
> - **52 夹具矩阵回归 (`scripts/run_52_fixtures_regression.py`)**: **52/52 全部 PASS**（False ID=0, Misses=0, Crashes=0, Correct Degradations=30，退出码 0）。
> - **Codex 18 项验收套件 (`scripts/run_acceptance.py`)**: **18/18 全部 PASS**（退出码 0）。
> - **Release 构建与度量 (`scripts/run_release_measurements.py`)**: **23 组样本全部 PASS**（退出码 0）。产物大小 912,408 字节，SHA256 `8665bd5efe4ce7ae3ae9fe1569fc84a966f7f2251f704ff96630df967b911c07`。
>
> ### C. 修订预算
> - **修订预算**：消耗 1 轮（1/2），剩余 1 轮。
>
> ---
>

>
> **阶段：spec v1.2 P1-A（最小可发布 CLI + Rust 库）—— 稳定 API/JSON v1/CLI/预算/错误与诊断契约**
> **实现者：`kindprobe_agy_developer` (`a618e7ea`)**
> **主管：`567bae8e`**
>
> ### A. 实施成果摘要 (Deliverables 1–10 达成情况)
> 1. **步骤一（契约修复）**：
>    - `src/lib.rs:28-40`：先 `File::open(path)`，再在句柄上调用 `file.metadata()?.is_file()`，消除打开前的路径竞态（TOCTOU）。
>    - D3 路径回显核查：合规。`DetectError::NotRegularFile` 作为库内部类型化错误对象，由 CLI 顶层捕获并在 stderr/错误信封回显调用方输入的路径（符合 CLI 诊断契约），而核心 `DetectionReport` 绝不泄漏宿主绝对路径。
>    - `tests/test_cli_contract.rs`：补全退出码 0、2、3、4 的完整反序列化断言与字段级测试，彻底替代原子串 `contains`。
> 2. **步骤二（D8 关键缺口彻底解决）**：
>    - 迁移 29 个缺失夹具（26 个 verifier + 3 个 round1）至 `fixtures/verifier/` 与 `fixtures/verifier_round1/`，并校验生成 `fixtures/verifier_manifest.json`（SHA256 与大小 100% 吻合）。
>    - 迁移 Codex 18 项验收 harness（`tests/codex_acceptance/cases.py`、`probe.rs` 及 `fixtures/codex_acceptance/`）。
>    - 在仓库根目录新增 `scripts/run_acceptance.py` 与 `scripts/run_52_fixtures_regression.py`，使普通 clean checkout 即可一键直接复跑全部 18 项 Codex 验收与全部 52 夹具矩阵，完全脱离对被忽略的 `.sandbox-test` 历史绝对路径的依赖。
> 3. **步骤三（D9 文档与许可声明）**：
>    - 全面更新 `README.md`：详细给出能力表（支持/仅容器/不支持/未验证）、精确结构核验机制与限制、Rust API 快速上手范例、CLI 用法与退出码规范、JSON v1 契约规范、第三方依赖许可清单。
>    - 平台验证状态诚实标明：仅 Linux x86_64 已验证，Windows/macOS 标明未验证。
>    - 项目许可证严守规则：标明「待维护者另行指定，本项目不擅自声明」。
> 4. **步骤四（D10 真实 Release 度量）**：
>    - 在受控沙箱容器 `local/rust-test:1.99.1-trixie` 内执行 `cargo build --release`。产物大小 909,616 字节，SHA256 `cc4e9e87765ad20ba98564748ce2ced02f68a77f0624365b5cb2ab6648f8ad42`。
>    - `scripts/run_release_measurements.py` 严格规范内存口径：因容器镜像无单独进程级 introspector，且禁止拿 docker client RSS 充数，真实进程内存如实记录为未测 (`not measured`)，并在文档与报告中明确说明不轻言逐字节完全可重现。
>
> ### B. 真实自测实据
> - **Cargo 离线测试套件**: 27 项测试全部 PASS（用例数由原 26 项新增至 27 项，增加 `test_cli_exit_codes_and_envelopes_full_coverage`，退出码 0）。
> - **52 夹具回归矩阵 (`scripts/run_52_fixtures_regression.py`)**: 52 组测试全部 PASS（False ID=0, Misses=0, Crashes=0, Correct Degradations=30，退出码 0）。
> - **Codex 18 项验收套件 (`scripts/run_acceptance.py`)**: 18/18 全部 PASS（7 API/CFB 探针 + 7 docx XML 对抗 + 4 CLI 信封，退出码 0）。
> - **Release 矩阵度量 (`scripts/run_release_measurements.py`)**: 23 组样本度量全部 PASS（False ID=0, Misses=0, Crashes=0，退出码 0）。
>
> ### C. 修订预算与后续
> - **修订预算**：实施阶段自测一次性全绿，**2 轮修订预算均未消耗**（0/2 消耗）。
> - **后续交接**：已准备完毕，等待独立验证员（`kindprobe_agy_verifier`）进行源码审查与独立复测。
>
> ---
>

>
> **阶段：spec v1.2 P1-A（最小可发布 CLI + Rust 库）—— 用户显式新授权，取代旧 profile 的「仅 P0 / 不得进 P1」范围条款。**
> **主管模型：MiMo-V2.6-Flash Xiaomi Token Plan（正常，临时 DeepSeek 覆盖已于 2026-11-05 结束）。**
> **认证 / 模型 / 账号 / 测试隔离 / 权限边界不变；不切账号模型、不延长超时掩盖异常、不原生测试回退、不循环重派、不扩范围。**
>
> ### A. 基线与范围
> | 项 | 值 |
> |---|---|
> | 基线 commit | `892a40b715987d6fb804eb6bec5da45575b5dfb2`（主管直读 `.git/logs/HEAD` 核对，3 提交线性） |
> | 远端 | `origin/main` 同值；P0 无 force / 无 Release |
> | 任务书 | `.agents/progress/P1A_TASK.md`（顶部 `Current dispatch note` 取代文末历史 blocker 与 "prepared, NOT dispatched"） |
> | 未提交改动 | `HANDOFF.md`（主管更新）与 `P1A_TASK.md`（未跟踪）；**无任何产品源码改动** |
> | 缺口表 | **已收到并经主管独立核实确认**（2026-11-05，经 `a618e7ea` 专属路由 `cao-mcp-server-a618e7ea` 送达，message `142`→回传；非文件，见对话记录） |
> | 修订预算 | **2 轮全部未消耗**（尚无产品实现） |
>
> ### A2. 主管模型（2026-11-04 临时覆盖 → 2026-11-05 已恢复）
> | 项 | 值 |
> |---|---|
> | 主管 CAO terminal | `0eb85957`（**保留，未新建替代主管**） |
> | 主管 profile 名 | `kindprobe_mimo_supervisor`（**保留**，未改） |
> | **当前实际模型** | **MiMo-V2.6-Flash Xiaomi Token Plan**（**已恢复**） |
> | 核验方式 | 同一 OpenCode native session `ses_e6858a8a6ffeJlMl3k7VxSJWpY` 经**原生** `/models` 核对：模型列表 **HTTP 200** 且原模型存在；**原生 `/models` 已选择回 MiMo**；TUI 底部确认 `MiMo-V2.6-Flash Xiaomi Token Plan` |
> | 历史（2026-11-04） | MiMo 故障 → 用户授权**临时**主管模型覆盖为 **DeepSeek V4.1 Flash（`deepseek/deepseek-flash`）**；**已于 2026-11-05 结束** |
> | 注意 | profile 的 **API `model` 字段仍是启动时标签**，**不能据此否定实际模型切换** |
>
> **不变项**：`P1A_TASK` 范围 / 角色 / 权限 / 认证 / 测试隔离全部不变；主管**仍只协调，不实现产品**；不扩格式、不进 P1B、不擅自选 LICENSE。
>
> **范围**：仅把**既有已验证格式**的 API / JSON v1 / CLI / 预算 / 错误与诊断 / 用户文档稳定为产品契约（规格 §§6–9、§11.3）。
> **不做**：新增格式、GUI、批量扫描、重命名、格式转换、Toss 集成、P1-B。
> **不重做**：P0 研究与已通过证据。
> **必须持续有效**：Codex 9 项失败回归 + 52 夹具正反矩阵，**不得降低测试预期**。
> **不采信**：旧 `FINAL_REVIEW.md`「无阻断」结论（已被 Codex 实测推翻）。
>
> ### B. 阻塞历史（2026-11-03，已修复，仅作存档）
> - **首次派单失败**：新建本地实现者 `kindprobe_agy_developer`（terminal `889a7325`）初始化失败：
>   `Antigravity CLI initialization timed out after 180.0 seconds`
>   → 系统已回收该终端，**任务内容从未送达**，零进度损失、无重复会话、无未知状态。
> - 当时已排除 profile/配置问题（`find_profiles` 中该 profile 健康，score 3.917 / coverage 3 / 检索首位）。
> - 当时处置：保存进度、带实据汇报、**停止盲试**、未切账号/模型/隔离/权限。
>
> **修复（用户侧 cao-workflow 更新，2026-11-03）**：
> - 新增规则：`source ~/.config/wsl-proxy/proxy.sh` 后 `proxy-on` 与 `proxy-check`；**控制终端代理不代表 worker 已继承**。
> - 经 Codex 核实，CAO `create_window` 复用现有 tmux `session.new_window`：本次在 `proxy-on`/`proxy-check` 通过后，**仅为 KindProbe 会话 `cao-kindprobe-resume-20261101`** 设置大小写 `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` 与 `NO_PROXY`/`no_proxy`（含 `localhost`/`127.0.0.1`/`::1`/当前网关），使新窗口继承。
> - **未改 provider 代码、未改全局服务、未改其他会话、未重启 Agent。**
> - 门禁：Windows 门禁、MiMo 原 native 会话模型认证门禁**均通过**；原主管 pane/pipe 正常、pending 0；Git 仍只有交接与 `P1A_TASK.md` 改动。
>
> **2026-11-04 第二次派单结果（已消亡，仅作存档）**：
> - 实现者 `kindprobe_agy_developer`（terminal `d9a85ac5`）**任务曾正式送达**，但此后 **pane 与 agy 进程均已消失**；主管 inbox **无来自它的任何结果回调**，pending 0。
> - **不因旧数据库行 unknown 将其视为活动，也不盲等。**
> - **残留证据核对（主管直读）**：`.agents/progress/` 仅 `FINAL_REVIEW.md` / `HANDOFF.md` / `P0_TASK.md` / `P1A_TASK.md` —— **无缺口表**；`docs/` 仅规格文件；Git 仍 `892a40b7`，**无产品源码改动**。
> - **结论：无可接续内容，不重做已有部分（因为没有）；修订预算未消耗。**
> - 判定为**旧会话消亡**，**非重复活动派单**。
>
> **2026-11-04 第三次派单结果 — 🔴 容量阻塞（当前阻塞，已停止派单）**：
> - 本地实现者 `kindprobe_agy_developer`，**terminal `a618e7ea`**，实际 pane **`kindprobe_agy_developer-7d7b`**。
> - API 层：`initial_delivery=delivered`、`status=completed` —— ⚠️ **`status=completed` 不等于产品交付**（见下）。
> - **实际 pane 停在服务错误（原样记录）**：
>   ```
>   Our servers are experiencing high traffic...
>   UNAVAILABLE (code 503): No capacity available for model gemini-3.8-flash-medium
>   ErrorID 95332b8c-0f8b-4fed-b6ab-7eff1db98f3f-102-2010
>   ```
> - **它已探索部分源码并执行若干检查，但没有返回缺口表。**
> - **Git 仍基线 `892a40b7`，只有 `HANDOFF.md` / `P1A_TASK.md` 文档改动，无产品源码改动。**
> - **未完成事项**：① 代理环境自检回报 ② **缺口表（第 1 步交付物）** ③ 主管范围确认 ④ 全部实施与自测 ⑤ 独立复验 ⑥ 整体审查 ⑦ 提交推送。
> - **修订预算：2 轮全部未消耗。**
>
> **本轮处置（按用户指令）**：
> - ✅ 保存进度并写入本 HANDOFF；**停止派单**；**不盲等回调**
> - ✅ **保留 `a618e7ea` 供同任务后续接续**（**不删除**）
> - ❌ **不自动重试生成**、**不 assign 替代 worker**、**不换模型/账号**、**不扩范围**、**不延长超时**
> - 等待用户决定；Codex 将如实报告容量阻塞。
>
> **⚠️ 授权边界限制（用户 2026-11-04 明确限定）**：
> - 本次用户**只授权修改「主管」模型**（MiMo → DeepSeek Flash）。
> - **未授权修改实现者模型**。`a618e7ea` 的 `gemini-3.8-flash-medium` **503 容量错误是独立阻塞**，与主管模型切换无关。
> - 因此：**不因这次切换重派、不重试生成、不自动换实现者账号/模型、不 assign 替代 worker**。
> - **保留原任务与全部证据**，等待**容量恢复**或**用户另行指示**。
> - **不重跑已通过的产品测试**（P0 的 52 夹具 / Codex 18 项 / 26 项 cargo test 结果继续有效）。
>
> **2026-11-05 实现者恢复与接续授权（用户指令）**：
> - **原 agy worker `a618e7ea` 重启后 pane/进程已确认消失**。
> - 经原生 `agy --conversation` 支持，已从对应 KindProbe / 容量错误日志确定 **native conversation `77e21659-f414-4cd9-b58d-f51df5cd8354`**，并**恢复同一 CAO window / terminal 及其专属 MCP 身份**，**不新 assign**。
> - **首次恢复带 `skip-permissions` 开关被自动审批拒绝** → **已改用普通权限成功恢复，未启用该开关**；**不得借「重新 assign + 默认高权限 provider」方式绕过本次拒绝**。
> - **当前 worker 真实界面**：空输入提示 + `?` 快捷键 footer，**显示历史已完成的自检/缺口表摘要**；CAO 曾一度仍 `unknown`。
> - ⚠️ **历史曾调用错误服务 `cao-mcp-server-fc210c53/send_message` —— 不得采信其自称「报送成功」**；**主管 inbox 无 `a618` 结果**。
>   → **必须要求它只使用自己专属的 `a618e7ea` 服务回调**，并由主管**核对实际结果**。
> - **代理正常、Docker 镜像正常。**
>
> **本次接续授权（仅一次）**：待 worker 空闲可正确识别后，**对昨天 503 容量故障在原会话尝试一次接续** ——
> 1. **优先读取已有缺口表内容并正确回调**（**不重跑已通过测试、不从头研究**）
> 2. 主管**核定范围**后再实施 P1A
> 3. **若再次 503 → 保存证据停止**，不自动反复生成、不换模型、不换账号
>
> **2026-11-05 状态刷新核实结果（Codex 定向核对，未改数据库/runtime）**：
> - `a618e7ea` **原生会话普通权限恢复成功**。
> - 通过**真实空输入提示处的 `?` 快捷键帮助界面**（**无模型调用、非审批**）刷新输出，再按明确 **`esc` Close 关闭** → **API 由 `unknown` 恢复为 `idle`**；**pane/pipe 正常，无待审批提示**。
> - **未改数据库或 runtime 代码、未 resize、未发送产品指令。**
> - **未启用 `skip-permissions`**（首次恢复时该开关被自动审批拒绝），**不得以「重新 assign + 默认高权限 provider」绕过该拒绝**。
> - 原 worker native 会话 **`77e21659-f414-4cd9-b58d-f51df5cd8354` 已保留**，后续恢复可复用。
>
> **接续已发出（仅一次）**：主管已通过 **`a618e7ea` 专属 MCP 路由**（`send_message`，`receiver_id=a618e7ea`，message_id `142`）下达指令 —— 要求其**读回已有缺口表并正确报送**，**不重跑 26 项已通过测试、不重做研究、不改产品源码**；并明确其**只准用 `a618e7ea` 自己的 MCP 身份回调**（历史误用 `cao-mcp-server-fc210c53` 不算送达）。
> **等待其回调；若再次 503 或出现实际权限确认 → 带具体证据暂停，不默认高权限重启/新 assign 绕过。**
>
> **2026-11-05 接续响应与首次权限处置（已核实）**：
> - MiMo **已真实回复**；原 worker `a618e7ea` 在**原 native 会话**中**已响应 message `142`**；**本轮未重现 503**。
> - 普通权限模式下，它**首个请求**执行只读命令：`env | grep -iE proxy | sort`
> - **Codex 核对**：实际 agy 进程中 proxy 命名变量**只有既定 8 项，且无认证项** → 针对**已观察到的该条只读命令**选择 **`1. Yes run once`**；
>   **未选 always/persist、未改 settings、未启用 skip-permissions。**
>
> **后续权限审批策略（用户 2026-11-05 明确）**：
> - 有**实际权限请求**时，用已支持的 **`answer_user_prompt` 等正式接口**处理。
> - **逐项核对**是否属于**用户既有授权的当前项目范围**。
> - **仅可对明确低风险 / 已授权的具体操作单次批准**（`Yes run once`）。
> - ❌ **不可一律放行 / 持久 allow（always/persist）**
> - ❌ **不可用高权限重启 / 新 assign 绕过**
> - ⚠️ **不明确 / 越权 / 有破坏性 → 报告用户，不自行批准**
>
> **进行中**：继续在**原 worker** 收回缺口表，走**正确 `a618e7ea` 专属路由**回调；收到后主管**核定范围**，再按当前 P1A 合同推进。
>
> **2026-11-05 缺口表已收到 → 范围已批准 → 实施中**：
> - 实现者 `a618e7ea` 经**正确专属路由**回传 **P1-A 完整四分类缺口表（D1–D10 + 有意不支持项）**，含环境自检实据（代理 8 项大小写双套、HEAD `892a40b7…`、零产品源码改动、2 轮预算未消耗）。
> - **主管独立核实两条驱动性事实主张，均属实**（不采信自报）：
>   1. `src/lib.rs:28-40` **确为先 `std::fs::metadata(path)` 后 `File::open(path)`**（路径式检查在打开前）
>   2. `tests/test_cli_contract.rs:56,57,67,105` **确为 `stdout.contains(...)` 子串断言**，非字段级信封校验
> - **主管补充两点（已下达，必须纳入）**：
>   1. **D8 量化**：`.gitignore` 第 2 行 `.sandbox-test/` **整目录忽略** → 普通 checkout **只有 23 个夹具**；52 矩阵中 **29 个**（26 verifier + 3 round1）与**完整 18 项 harness**（`run_acceptance.py`/`cases.py`/`probe.rs`+fixtures）**全部不可得**。D8 必须一并纳入。
>   2. **D3 路径回显核查点**：`src/lib.rs:34` `DetectError::NotRegularFile(path.to_string_lossy().to_string())` **携带完整宿主路径**，与「绝不泄露宿主绝对路径」潜在冲突 → **先按规格 §7/§9.3 核查再定，不臆断**。
> - **实施范围已批准（四步）**：① 句柄级检查 + CLI 字段级信封断言 ② D8 迁移（18 项 harness + 29 夹具 + 52 矩阵）③ README/文档/许可诚实声明 ④ 沙箱 release 重新度量。
> - **红线重申**：不重跑已通过测试、不改反例预期、不删失败测试、不把伪输入改真、不扩格式、不进 P1B、不擅自选 LICENSE、权限逐项审批、再现 503 即停。
>
> ### 🔴 当前阻塞（2026-11-05，第二次 503 容量故障 —— 已 checkpoint，停止重试）
> - **已完成并保留的进度（不得再说缺口表未产出）**：
>   - ✅ **P1-A 缺口表已通过** —— 经 `a618e7ea` 正确专属路由 `cao-mcp-server-a618e7ea` 回传，**经主管独立核实确认**（直读证实 `src/lib.rs:28-40` 路径式检查、`test_cli_contract.rs` 子串断言两条事实主张）
>   - ✅ **四步实施范围已批准**（见上）
>   - ✅ 环境自检实据完整（代理 8 项大小写双套、HEAD `892a40b7…`）
> - **本次失败实据（原样记录）**：
>   - worker `a618e7ea` **收到四步实施任务后已读 `src/lib.rs` / `model.rs`**
>   - 实际 pane 再次报：
>     ```
>     UNAVAILABLE (code 503): No capacity available for model gemini-3.8-flash-medium
>     ErrorID 95332b8c-0f8b-4fed-b6ab-7eff1db98f3f-176-2010
>     ```
>   - ⚠️ **区别于昨日的 `…-102-2010`**（**新 ErrorID 后缀 `-176-2010`**），是**新的容量失败**，非同一条记录。
> - **Git 状态**：仍 `892a40b7…`，仅 `HANDOFF.md` / `P1A_TASK.md` 文档改动；**无产品源码改动、无新提交、无运行中的测试容器**。
> - **修订预算**：**2 轮全部未实际动用**（尚无产品实现，不得记为已消耗）。
>
> **本次处置（按用户指令）**：
> - ✅ 更新本 HANDOFF 顶部并 **checkpoint**
> - ✅ **停止重试、不盲等实施回报**
> - ❌ **不自动生成重试**、**不新 assign 替代 worker**
> - ❌ **不换实现者模型/账号**、**不启用持久授权**、**不用高权限重启绕过**
> - ❌ **不动用未实际发生的产品修订预算**
> - **保留**：原 worker **native conversation `77e21659-f414-4cd9-b58d-f51df5cd8354`** + 正确 `a618e7ea` 专属回调路由
> - **等待容量恢复或用户另行指示**
>
> **2026-11-05 用户授权继续 → 有界接续实施中**
> - 用户已**明确授权继续 P1-A**，并要求先读取已更新的 `/mnt/c/Users/IceLolly/.codex/skills/cao-workflow/SKILL.md`（已读）。
> - **复用**现有主管 + `a618e7ea` / native `77e21659-f414-4cd9-b58d-f51df5cd8354`；两 pane 存活、pending 0、无运行测试容器。
> - **本次授权范围**：在**原实现者普通权限会话**对最新 503 做**一次有界接续**，按**已批准四步**实施；**不重新研究、不重开缺口表**。
> - **失败即停**：再遇 503 / 认证 / 额度失败 → **立即保存证据停止**，不循环重试、不切模型账号、不新 assign 替代实现者。
>
> ### 🔁 独立验证分工变更（2026-11-05，取代旧安排）
> | 角色 | 当前指定 | 状态 |
> |---|---|---|
> | **日常独立验证** | 主账号独立 agy High 会话 **`kindprobe_agy_verifier`**（本地，与实现者分属不同会话） | ✅ **启用**（`find_profiles` 存在，score 4.906 检索首位） |
> | Claude Code 验证员 / `claude_consultant` | — | ❌ **已按 2026-11-05 人类要求退出本工作流**：不自动派发、**不复用旧 Claude worker**、**不作故障回退**（配置保留供人类自用） |
> | 阶段整体审查 | 第二账号 agy High `agy_whole_reviewer` @ `target_host=http://127.0.0.1:9890` | 阶段完成后一次 |
>
> - 独立性来自**独立会话 + 重新取证**，**不代表不同模型的交叉验证**。
> - **验证员只读产品源码**，可写自己的测试/证据；实现者自评**不得**替代独立会话复验。
> - ⚠️ **若验证启动入口带先前被拒绝的 `skip-permissions` → 先报告并采用可核实的普通权限启动方案，禁止直接绕过。**
>
> **P1-A 收尾边界（不变）**：产品测试继续沿用**已批准受限 Docker 合同**，**禁止原生回退**；实现后由 `kindprobe_agy_verifier` 审查复测 → 阶段整体审查 → 达标后按既有授权 **commit/push origin main**（**不 force、不 Release**）→ **不选项目 LICENSE** → **不进入 P1-B** → 更新交接并**停等验收**。
>
> ### C. 历史会话（均不存活，不得派单）
> - P1-A 第二次实现者 `d9a85ac5`：曾送达任务，pane/agy 进程消失，无回调，无产出。
> - P1-A 首次实现者 `889a7325`：初始化超时失败，已回收，任务未送达。
> - P0 实现者 `41f07fa1`、P0 验证员 `40ff2406`：真实 pane 已消失，prewarm `pane_missing`。
> - P0 整体审查 `0e866f20` / session `cao-39f1107c` @ `http://127.0.0.1:9890`（lolicon）：P0 阶段收尾审查用，历史存档。
> - 主管 `0eb85957` / `ses_e6858a8a6ffeJlMl3k7VxSJWpY`：已恢复，无 fork、无替代主管。
> - 更早历史：`6cf1541f` / `27f72082` / `849bcb62` / `7e43a873` / `ee25d750`。
>
> **✅ 保留会话（存活，供同任务后续接续，不得删除、不得派替代 worker）**
> - **`a618e7ea`** — P1-A 第三次实现者，pane `kindprobe_agy_developer-7d7b`，当前停在上述 503 容量错误；**同任务接续时优先复用它**。
>
> ### D. P1-A 待办与验证流程（本阶段独立预算，尚未消耗）
> 1. 精简**缺口表**（四分类：已实现 / 已测试 / 缺失 / 有意不支持）→ 主管核范围
> 2. 实施缺项 + 补测试 → 实现自测
> 3. **Claude Code 独立源码审查 + 真实定向重测**
> 4. 至多 **2 轮修订**（P1-A 独立预算，与 P0 已用尽的 2 轮无关）
> 5. 稳定里程碑 → 9890 `agy_whole_reviewer` 阶段收尾全项目源码审查
> 6. 达标后 commit / push origin main（不 force、不创建 Release）
> 7. **暂停等人工验收，不自动进入 P1B**
>
> **已知关键缺口（主管预判，待缺口表确认）**：Codex 18 项验收的 `run_acceptance.py` / `probe.rs` / `cases.py` / `fixtures/` 全在**被 .gitignore 的** `.sandbox-test/results/codex-acceptance-20261102/`，普通 checkout 不可得；`tests/test_codex_acceptance_regression.rs` 仅覆盖 9 项失败，另 9 项通过项不在仓库内 → 违反任务书 Deliverable 8。
>
> ### E. 许可证
> **用户尚未指定项目许可证。不得擅自选择 LICENSE。** 若其阻碍正式发布门槛，单列为**待决定**，不冒称全部发布条件通过；不阻碍已授权的代码加固与文档工作。

---

> ## ⛔ P0 历史状态 (2026-11-02 深夜更新) — Codex 验收打回 → Round 2 修复并复验通过，**待人工验收**
>
> **阶段结论：Codex 亲自验收 FAIL 打回；Revision Round 2（终轮）已完成修复与独立复验，实测达标。人工验收尚未完成。未进入 P1。**
>
> ### A. ⚠️ 撤回声明（必须先读）
> 1. **撤回**此前「P0 已验收 / 整体审查无阻断」的结论。Codex 对基线 `6c4dace5…` 亲自实测 **FAIL**（18 项中 9 项失败），**该实测已否定旧结论**。
> 2. `.agents/progress/FINAL_REVIEW.md` 的「**无阻断性缺陷**」判断**已被实测推翻，不再作为正确性证明宣传**；其 §1.1 关于 `fcMin/fcMac` 的字段解释**未被采信**（现官方 FibBase 该区域为 reserved），本轮未照搬。
> 3. **度量勘误**：Round-1 报告中「release 100% 可重现构建」的说法**不成立**。同源代码在不同 `CARGO_TARGET_DIR` 下产物不同（路径嵌入符号表/恐慌元数据）。
>
> ### B. Codex 验收打回（原始失败证据，永久保留不得覆盖）
> - 目录：`.sandbox-test/results/codex-acceptance-20261102/`
> - `REPORT.md`、`execution.json`（`conclusion: FAIL`，`total 18 / passed 9 / failed 9`，`source_unchanged: true`）
> - `api-cfb-cases.json`、`cli-cases.json`、`run_acceptance.py`
> - 失败 9 项：`invalid_root_deep`、`wrong_namespace_prefixed`、`no_namespace_empty_root`、`invalid_multiple_roots`、`cfb_forged_fib_missing_table`、`cfb_embedded_word_in_workbook`、`cfb_unsupported_fib_version`、`api_missing_extension`、`png_signature_only_evidence`
>
> ### C. Round 2 修复与独立复验（终轮）
> | 项 | 实据 |
> |---|---|
> | 实现者修复 | 9 项打回缺陷 + 收口补 2 个历史正例（`adv_true_doc.doc`、`adv_true_doc_renamed.zip` 补 `0Table` 直属流） |
> | 新增永久回归 | `tests/test_codex_acceptance_regression.rs`（9 项） |
> | **Codex 18 项** | 独立重跑 `codex-acceptance-rerun-20261102T145643/execution.json` → **`conclusion: PASS`，`passed 18 / failed 0 / failed_ids: []`** |
> | **52 夹具** | `verifier_round2/52_fixtures_regression_results.json` → **`false_id 0 / misses 0 / crashes 0`**，`correct_degradations 30` |
> | `cargo test` | **26/26**（17 既有 + 9 新回归），退出码 0；doctest 退出码 0 |
> | 官方基准 | `scripts/verify_runner.py` 退出码 0（`false_id=0, misses=0, crashes=0`） |
> | 独立复验报告 | `.sandbox-test/results/verifier_round2/INDEPENDENT_VERIFICATION_R2.md` 与 `INDEPENDENT_VERIFICATION_R2_FINAL.md` |
>
> **Release 度量定案**（主管与验证员一致）：
> - 规范路径 `/output/target/release/kindprobe`（宿主 `.sandbox-test/target/release/kindprobe`）= **909,048 字节 / SHA256 `74b4f7d4793431c8bb60927832ab0304c19d0bc50361e4cc033c0a6eae4a7c8e`**，与 `release_measurements.json` 比特级一致。
> - `908,704 字节 / 5e5e2b50a8d46aa23c1f6f6fa813e751821358e7bd7c72c02b7c4756dc50f0f9` 系 `run_acceptance.py` 设 `CARGO_TARGET_DIR=/accept/build` 的临时产物，**不是**规范 release 度量。
>
> ### D. 修订预算（已用尽）
> - **Revision Round 1**：本文档 §7（初始审查 5 项缺陷修复）。
> - **Revision Round 2（终轮）**：Codex 打回 9 项 + 收口 2 正例。
> - **预算已用 2 轮，剩余 0 轮。** 若后续再发现 P0 缺陷，**不得自动扩轮**，须保存摘要并报告阻塞、交人类决定。
>
> ### E. 会话与 Terminal ID（保留不清理）
> | 角色 | Terminal / Session | 位置 |
> |---|---|---|
> | 主管 (MiMo) | `0eb85957` / 原生对话 `ses_e6858a8a6ffeJlMl3k7VxSJWpY` | 本地 |
> | 实现者 `kindprobe_agy_developer` | `41f07fa1` | 本地 |
> | 独立验证员 `kindprobe_claude_verifier` | `40ff2406` | 本地 |
> | 全项目审查 `agy_whole_reviewer` | terminal `0e866f20` / session `cao-39f1107c` | 远端 `http://127.0.0.1:9890` (lolicon) |
>
> 历史记录 `6cf1541f` / `27f72082` / `849bcb62` / `7e43a873` / `ee25d750` 仅为旧会话，**不存活、不得向其派单**。
>
> ### F. 证据分层（必须分别引用，不得混为一谈）
> 1. **实现者自测** → 本文档 §7（Round 1）与 Round 2 收口汇报。⚠️ 实现者曾两次自报不实（「52 组无回归」实为 misses=2；release 路径与 SHA 张冠李戴），**均以独立实测为准**。
> 2. **独立实测** → `verifier_round1/INDEPENDENT_VERIFICATION_R1.md`（Round 1）、`verifier_round2/INDEPENDENT_VERIFICATION_R2*.md`（Round 2，**权威**）。
> 3. **Codex 亲自验收** → `codex-acceptance-20261102/REPORT.md`（**FAIL 历史**）及各 `codex-acceptance-rerun-*/execution.json`（修复后 PASS）。
> 4. **整体审查** → `.agents/progress/FINAL_REVIEW.md` —— **其无阻断结论已被实测否定，仅作历史存档**。
> 5. **人工验收** → **未完成 (Pending Human Acceptance)**。AI 代理不得代为标记通过。
>
> ### G. 遗留与边界
> - `FINAL_REVIEW.md` §1 的 8 项 `inferred` 缺陷与 §3 的 4 条 P1 建议：**未实施**，属新增范围，留待人类选择。
> - 项目代码许可证**仍待维护者指定**，`README.md` 已明示待定；第三方许可记录完整。
> - 未验证：Windows / macOS；P1-B 格式（DOCM/XLSM/PPTM、XLSX/PPTX、7z/RAR、媒体）。
>
> ### H. 后续
> 1. 维护者**再次人工验收**（Codex 打回后的复验结果需由人最终确认）。
> 2. **本轮到此暂停，不自动推进 P1、不开发新功能、不自动扩轮。**

**Date:** 2026-11-02T23:15:00+08:00  
**Executor:** kindprobe_agy_developer（Round 2 修复） / kindprobe_claude_verifier（Round 2 独立复验）  
**Git Remote:** `https://github.com/vusiif/KindProbe.git` (branch `main`) — Round 2 修复**待提交推送**（见置顶 C）

---

## 1. P0 Hard Gate Status (P0-G0 .. P0-G6)

| Gate ID | Condition | Status | Evidence Index |
|---|---|---|---|
| **P0-G0** | Candidate comparison (`file`, `Siegfried`, `infer`, `file-format`), signature source & licensing audit, explicit `accept/replace/defer/reference_only` decisions | **PASS** | `research/candidates.md`<br>`research/baseline-results.json`<br>`research/reuse-decisions.md`<br>`research/third_party_matrix.md`<br>`research/signature_sources.md`<br>`research/rules-manifest.json` |
| **P0-G1** | True DOCX vs plain ZIP vs fake OOXML ZIP separation; true DOC vs plain CFB vs fake Word stream separation | **PASS** | `fixtures/fake_docx_zip.zip` -> `container_only` (`ooxml.content_types_missing`)<br>`fixtures/fake_doc_cfb.doc` -> `container_only` (`cfb.invalid_word_fib`)<br>`fixtures/true_docx.docx` -> `format: docx`<br>`fixtures/true_doc.doc` -> `format: doc`<br>Report: `.sandbox-test/results/p0_matrix_results.json` |
| **P0-G2** | In the fixed P0 test suite, false identifications must equal 0 | **PASS** | **False identifications = 0** across 23 developer fixtures and 25 independent verifier fixtures (tested via release binary in sandbox) |
| **P0-G3** | Proper degradation and stable diagnostics on budget triggers, truncation, unsupported features, structural conflicts | **PASS** | `budget_xml_deep.docx` -> `inspection: limited`, code `budget.xml_depth_exceeded`<br>Truncated/empty inputs -> `input.truncated`, `input.empty`<br>`cfb_cyclic_chain.doc` -> `cfb.stream_read_error`<br>ZIP directory window exceeded -> `budget.zip_directory_window_exceeded`<br>Decompressed total exceeded -> `budget.total_decompressed_exceeded`<br>CFB entries exceeded -> `budget.entries_exceeded`<br>ZIP duplicate/conflict members -> `zip.conflicting_entries`, `inspection: ambiguous` |
| **P0-G4** | Shared core across `detect_path` / `detect_bytes` / `detect_reader`; extension diagnosis independence; zero I/O side effects | **PASS** | `tests/test_api_consistency.rs` (all 3 APIs yield identical reports)<br>`tests/test_extension.rs` (mismatched extension does not change format)<br>Real read/seek I/O failures propagate as `Err(DetectError::Io)` with CLI exit code 4 |
| **P0-G5** | Real release build & measurements (Linux, toolchain, size, source read, duration, memory) | **PASS** | Target: `x86_64-unknown-linux-gnu`<br>Binary Size: 895,928 bytes (~875 KiB), SHA256 `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`<br>Detailed log: `.sandbox-test/results/release_measurements.json` |
| **P0-G6** | Third-party dependency, rule data, and code reuse decisions; unsupported matrix | **PASS** | `research/reuse-decisions.md`<br>`README.md` Capability & Unsupported Matrix |

---

## 2. Evidence Directory Index (`.sandbox-test/results/`)

- `.sandbox-test/results/p0_matrix_results.json`: Full machine-readable record of all 23 fixture runs using the release binary inside Docker sandbox (exit codes, peak RSS, engine time, CLI time, source bytes read, seek counts).
- `.sandbox-test/results/release_measurements.json`: Environment metadata, compiler version, build profile, and aggregated test summary.
- `research/baseline-results.json`: 4-candidate comparison results (`file 5.46`, `Siegfried 1.11.9`, `infer 0.22.0`, `file-format 0.29.0`) over the 23 fixtures.
- `.sandbox-test/results/verifier/`: **首轮（Round 1 之前）**独立验证员对抗测试结果、manifest 与测试 harness（25 组夹具）。
- `.sandbox-test/results/verifier_round1/`: **Round 1 独立复验**产物（独立实测层，权威）：
  - `INDEPENDENT_VERIFICATION_R1.md` —— 独立复验报告，结论 PASS，含 P0-G0..G6 表与 E1–E10 命令退出码。
  - `verifier_round1_results.json` —— 52 组夹具 + 5 组 CLI 契约的机器可读实测（summary 全零）。
  - `run_verifier_round1.py`、`test_verifier_round1_cases.rs` / `_bin`、`fixtures/`（3 组 Round 1 回归夹具）。
  - ⚠️ 该目录被 `.gitignore` 排除，Glob 查不到**不等于**丢失。
- `.agents/progress/FINAL_REVIEW.md` —— 9890 全项目阶段收尾审查报告（无阻断性缺陷，8 项 `inferred` 发现 + 4 条 P1 建议）。

---

## 3. Metrics Summary

- **Developer Test Fixtures:** 23 (100% PASS, False ID = 0, Miss = 0, Degraded = 12, Crashes = 0)
- **Verifier Adversarial Fixtures:** 25 (100% PASS, False ID = 0, Miss = 0, Degraded = 14, Crashes = 0)
- **Cargo Unit & Integration Tests:** 18 tests (100% PASS, 0 failed, 0 warnings)
- **Release Binary Size:** 895,928 bytes (~875 KiB)
- **Release Binary SHA256:** `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`
- **Engine Execution Time per File:** 20 µs to 500 µs (average < 0.2 ms)
- **Docker Client Peak Resident Set Size:** ~27.7 MB

---

## 4. Unverified Scope & Boundary Constraints

1. **Test Corpus Authorization & Boundary (User Instruction 2026-10-31):**
   - 规格 §11.4 中提到的 Toss 本机语料（`.sandbox-test/doc-corpus-20261030/`、`docx-corpus-20261027/`、`tests/corpus/` 等）本轮**明确未获用户授权，禁止读取、复制或引用**。
   - 原任务书中“如使用 §11.4 提到的本机 Toss 语料…”一句**作废**，本项目严禁访问上述路径。
   - 同时严禁读取或复制 Toss、WorDF、FClue 的任何私有或本机样本文件。
   - 测试样本严格限定为两类：
     - (a) 本项目内用可复现脚本（`scripts/generate_fixtures.py`）自行合成/生成的无侵权夹具（当前 23 组样本全部属于此类）；
     - (b) 公开 HTTPS 来源、且来源 URL、版本/提交、许可、SHA256 均已核实并记录的样本。
   - 未知授权语料一律不得进入测试集或仓库。
2. **Formats (Format Scope):**
   - Office Macro variants (DOCM, XLSM, PPTM) -> Deferred to P1-B
   - Other Office families (XLSX, PPTX, XLS, PPT) -> Deferred to P1-B (currently recognized as container `zip` or `cfb`)
   - Archives (7z, RAR) -> Deferred to P1-B
   - Media (MP4, MP3, WAV, WebM) -> Deferred to P1-B
3. **Platforms:**
   - Linux x86_64 (`Debian trixie / Linux 6.6`): **Verified**
   - Windows / macOS: **Unverified (未验证)**
4. **Licensing:**
   - Project license is unassigned by maintainer; explicitly marked as pending in `README.md`.
5. **Human Acceptance:**
   - Human acceptance is pending; cannot be marked as passed by AI agents.

---

## 5. Independent Verification Command

To independently verify all P0 gates from the host:
```bash
python3 /srv/projects/rust/KindProbe/scripts/verify_runner.py
```
This script will:
1. Verify the local Docker test image (`local/rust-test:1.99.1-trixie`, ID `sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3`).
2. Run `cargo test --offline` inside the read-only sandbox.
3. Build the release binary inside the read-only sandbox.
4. Execute release benchmark measurements on all 23 fixtures.
5. Confirm `false_identifications == 0`, `misses == 0`, and `crashes == 0`.

---

## 6. Independent Verification Report (独立复验报告 - 初始审查)

**Date:** 2026-10-31T23:35:00+08:00  
**Verifier:** kindprobe_claude_verifier (Claude Code / Gemini Flash High Proxy)  
**Verifier CAO Terminal ID:** `ee25d750`  
**Target Host / Remote:** `https://github.com/vusiif/KindProbe.git` (branch `main`)  
**Git Commit / Push Status:** Clean read-only review, NO commit/push executed by verifier.

### 6.1 Verification Commands and Exit Codes

| Step | Executed Command | Exit Code | Observed Outcome |
|---|---|---|---|
| **Docker Image Verification** | `docker image inspect local/rust-test:1.99.1-trixie --format '{{.Id}}'` | `0` | Digest verified: `sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3` |
| **Cargo Test (Sandbox)** | `docker run ... cargo test --offline --manifest-path /workspace/Cargo.toml` | `0` | 11 unit & integration tests passed; 0 failed |
| **Release Build (Sandbox)** | `docker run ... cargo build --release --offline --manifest-path /workspace/Cargo.toml` | `0` | Binary built: 871,832 bytes; SHA256 `39806728292e5c117c91d1cf751f27109c2eff767e26a096cf24d8355c4e117b` |
| **Independent Adversarial Suite** | `python3 -I .sandbox-test/results/verifier/run_verifier_tests.py` | `0` | 25 self-synthesized adversarial fixtures: False ID = 0, Miss = 0, Degraded = 14, Crash = 0 |
| **3-API Consistency Verification** | `/output/results/verifier/verify_api_consistency` | `0` | All 25 adversarial fixtures: `detect_path` == `detect_bytes` == `detect_reader` |
| **CLI Error Envelope Contract** | `kindprobe --json <dir>` / `kindprobe --json <non_existent>` | `4` | Strict single-line JSON error envelopes `io.not_regular_file` and `io.open_failed` |

### 6.2 Hard Gate Status (P0-G0 .. P0-G6)

| Gate ID | Condition | Status | Verifier Findings & Audit Summary |
|---|---|---|---|
| **P0-G0** | Candidate comparison (`file`, `Siegfried`, `infer`, `file-format`), signature source & licensing audit, explicit `accept/replace/defer/reference_only` decisions | **PASS** | `research/candidates.md`, `research/baseline-results.json`, `research/reuse-decisions.md`, `research/third_party_matrix.md`, `research/signature_sources.md`, `research/rules-manifest.json` are complete with fixed versions, raw candidate outputs, SHA256 hashes, and explicit decisions. |
| **P0-G1** | True DOCX vs plain ZIP vs fake OOXML ZIP separation; true DOC vs plain CFB vs fake Word stream separation | **PASS** | Validated across both original 23 fixtures and 25 verifier adversarial fixtures. Fake DOCX (missing rels, missing content-types, wrong content-type, directory only, path traversal, wrong XML root) 100% degraded to `container_only`. Fake DOC (zero FIB, pre-97 FIB, invalid fc offsets) 100% degraded to `container_only`. True DOCX and True DOC 100% confirmed. |
| **P0-G2** | In the fixed P0 test suite, false identifications must equal 0 | **PASS** | **False identifications = 0** across all 23 developer fixtures and all 25 verifier adversarial fixtures. |
| **P0-G3** | Proper degradation and stable diagnostics on budget triggers, truncation, unsupported features, structural conflicts | **PASS** | Truncation triggers `input.truncated`; empty triggers `input.empty`; XML depth > 64 triggers `budget.xml_depth_exceeded` with `inspection=limited`; CFB cyclic FAT triggers `cfb.stream_read_error`. *(Note: see findings below regarding unused budget options).* |
| **P0-G4** | Shared core across `detect_path` / `detect_bytes` / `detect_reader`; extension diagnosis independence; zero I/O side effects | **PASS** | 3 APIs yield identical identification, container, verification, and inspection across all 25 adversarial fixtures. Mismatched extensions (.pdf on DOCX, .docx on PNG, .zip on DOC, no extension) do not alter content identity. Library has zero stdout/stderr prints, zero file writes, zero network operations. *(Note: see defect below regarding I/O error return).* |
| **P0-G5** | Real release build & measurements (Linux, toolchain, size, source read, duration, memory) | **PASS** | Release binary size 871,832 bytes (~851 KiB), SHA256 `39806728292e5c117c91d1cf751f27109c2eff767e26a096cf24d8355c4e117b`. Engine execution time: 20 µs to 500 µs. Memory RSS ~27.7 MB. |
| **P0-G6** | Third-party dependency, rule data, and code reuse decisions; unsupported matrix | **PASS** | Dependency licenses verified (MIT / Apache-2.0). Capability and unsupported matrix documented in `README.md`. Platform constraints documented (Linux verified, Windows/macOS unverified). Project license pending maintainer determination. |

### 6.3 Independent Statistics Summary

- **Total Verifier Adversarial Fixtures:** 25
- **False Identifications:** 0
- **Missed Identifications:** 0
- **Correct Degradations:** 14
- **Crashes / Uncaught Panics:** 0 (on valid paths)
- **JSON Single-Object + Newline Contract Violations:** 0
- **Raw Input Byte Immutability:** 100% verified (SHA256 identical before and after inspection)

### 6.4 Differences from Developer Self-Reported Metrics (Historical Audit Note)

1. **Binary Size Discrepancy**: Developer HANDOFF section 3 originally stated `872,328 bytes (~852 KiB)`, whereas `release_measurements.json` and actual verifier build measured `871,832 bytes` (496 bytes difference). SHA256 checksum was identical (`39806728292e5c117c91d1cf751f27109c2eff767e26a096cf24d8355c4e117b`). This discrepancy has been reconciled in Revision Round 1 by measuring the actual sandbox build output (`895,928` bytes, SHA256 `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`), which matches exactly across filesystem, JSON, and sha256sum.
2. **Defect Discovery**: The developer initial self-tests only covered UTF-8 paths, normal I/O, and positive/negative synthetic samples. Independent verifier fuzzing uncovered defects that have been addressed in Revision Round 1.

### 6.5 Initial Audit Defect & Quality Findings (Ranked by Severity)

#### [High Severity] CLI Panic on Non-UTF-8 Command-Line Arguments
- **Location:** `src/main.rs:45`
- **Observed Behavior:** When a file path containing non-UTF-8 byte sequences is passed to the CLI, `env::args().collect()` panicked with `called Result::unwrap() on an Err value` before `detect_path` was invoked.
- **Specification Conflict:** Spec §8 and §6.3 require Unix non-UTF-8 paths to be supported without panic.
- **Resolution in Round 1:** Resolved via `env::args_os()` and `to_string_lossy().starts_with('-')`.

#### [Medium Severity] Swallowing I/O Read/Seek Failures into Unknown DetectionReport
- **Location:** `src/detectors/mod.rs:20-39, 64-81`
- **Observed Behavior:** When `reader.read(&mut header)` or `reader.seek(SeekFrom::Start(0))` encountered a genuine `std::io::Error`, `detect_stream` caught the error and returned a `DetectionReport` with `identification: Unknown, inspection: Limited, warnings: ["io.read_error" / "io.seek_error"]`. In CLI mode, this caused exit code 3.
- **Specification Conflict:** Spec §7, §8, §9.2 require genuine I/O errors to return `Err(DetectError::Io)` with CLI exit code 4.
- **Resolution in Round 1:** Resolved by propagating `std::io::Error` as `Err(DetectError::Io)`.

#### [Low / Quality Severity] Unenforced Budget and Statistic Fields
- **Location:** `src/detectors/zip_ooxml.rs` & `src/detectors/cfb_doc.rs`
- **Observed Behavior:**
  1. `options.max_zip_directory_window` was defined but never checked.
  2. `options.max_total_decompressed_bytes` was defined but never checked.
  3. In `detect_cfb_doc`, `comp.walk()` did not check `options.max_entries`, and `stats.entries_inspected` was never updated.
  4. Duplicate ZIP member names or conflicting relationships were not flagged as `zip.conflicting_entries` with `inspection: ambiguous`.
  5. In `src/extension.rs:33-36`, non-UTF-8 extension check returned `ExtensionStatus::Unrecognized` before checking Priority 2 (`identification != Format` -> `Undetermined`).
- **Resolution in Round 1:** All 5 items fully enforced and covered by regression tests.

### 6.6 Evidence Directory Index (`.sandbox-test/results/verifier/`)

- `.sandbox-test/results/verifier/verifier_manifest.json`: Manifest of 25 independent adversarial fixtures with SHA256 and expected outputs.
- `.sandbox-test/results/verifier/verifier_results.json`: Full machine-readable execution results of all 25 adversarial fixtures and edge cases.
- `.sandbox-test/results/verifier/verify_api_consistency.rs` & compiled binary: Verified `detect_path` == `detect_bytes` == `detect_reader` across all 25 fixtures.
- `.sandbox-test/results/verifier/test_io_error_handling.rs` & compiled binary: Verified reproduction of I/O error swallowing behavior.
- `.sandbox-test/results/verifier/test_non_utf8_path.rs` & compiled binary: Verified library-level non-UTF-8 path handling.
- `.sandbox-test/results/verifier/fixtures/`: 25 synthesized test files.

---

## 7. Revision Round 1 Remediation Log (2026-11-01)

The following defect items identified during initial verification review have been remediated, verified in sandbox, and covered with automated regression tests:

### 7.1 Remediated Defects

1. **[High] CLI Panic on Non-UTF-8 Command-Line Arguments (`src/main.rs`)**:
   - Replaced all string assumptions with `std::env::args_os()` and `OsString` / `PathBuf`.
   - Used lossless `arg.to_string_lossy().starts_with('-')` for option checking, preventing panics on non-UTF-8 options and file paths on Unix filesystems.
   - Added automated regression test `test_cli_non_utf8_arguments_and_paths` in `tests/test_cli_contract.rs`. Verified verifier's `test_non_utf8_path`.

2. **[Medium] Swallowing I/O Read/Seek Failures into Unknown DetectionReport (`src/detectors/mod.rs`, `src/lib.rs`)**:
   - Changed `detect_stream`, `detect_zip_ooxml`, and `detect_cfb_doc` signatures to return `Result<DetectionReport, DetectError>`.
   - Genuine `std::io::Error` instances on `read` or `seek` (e.g. broken pipe, disk I/O error) now propagate as `Err(DetectError::Io(e))` rather than returning `Identification::Unknown` with exit code 3.
   - CLI prints single-line JSON error envelope with `"code": "io.open_failed"` and exits with code 4 according to specification §8 / §9.2.
   - Preserved distinction: budget exhaustion still returns `Ok(report)` with `inspection: limited` and stable `budget.*` codes; structural data truncation/corruption returns `Ok(report)` with `container_only` or `unknown`.
   - Added automated regression test `test_real_io_errors_return_err` in `tests/test_budget_adversarial.rs`. Verified verifier's `test_io_error_handling` (both simulated read and seek errors return `Err(DetectError::Io)`).

3. **[Medium/Low] Unenforced Budget and Statistic Fields (`src/budget.rs`, `src/detectors/zip_ooxml.rs`, `src/detectors/cfb_doc.rs`)**:
   - `max_zip_directory_window`: Pre-checks EOCD `cd_size` and enforces dynamic byte window during central directory parsing via `BudgetReader::start_window()`. Exceeding limits returns `inspection: limited` with warning `budget.zip_directory_window_exceeded`. Tested in `test_budget_zip_directory_window`.
   - `max_total_decompressed_bytes`: Tracked cumulative decompressed bytes across all XML parts (`_rels/.rels`, `[Content_Types].xml`, `word/document.xml`). When total decompressed bytes reach budget, stops and returns `inspection: limited` with warning `budget.total_decompressed_exceeded`. Tested in `test_budget_total_decompressed_bytes`.
   - `max_entries` in CFB: Inspected entry count is tracked in `comp.walk()`. Exceeding `options.max_entries` halts iteration and returns `inspection: limited` with warning `budget.entries_exceeded`. Tested in `test_cfb_entries_budget_and_stats`.
   - `stats.entries_inspected` in CFB: Now correctly updated with the actual number of compound document entries visited.

4. **[Medium] ZIP Duplicate Member Names and Conflicting Relationships (`src/detectors/zip_ooxml.rs`)**:
   - Pre-parses central directory headers from `cd_offset` to detect duplicate raw entry names or normalized OPC collisions before `ZipArchive` deduplicates them into `IndexMap`. Duplicate entries yield `identification: container_only`, `inspection: ambiguous`, and warning `zip.conflicting_entries`.
   - `_rels/.rels` relationship parser detects multiple conflicting `officeDocument` targets, returning `identification: container_only`, `inspection: ambiguous`, and warning `zip.conflicting_entries`.
   - Tested in `test_zip_conflicting_duplicate_entries` and verified against `adv_duplicate_key_members.zip`.

5. **[Low] Extension Comparison Priority Order (`src/extension.rs`)**:
   - Reordered checks in `compare_extension()` according to specification §6.3:
     1. `extension.is_none()` or empty -> `ExtensionStatus::Missing`.
     2. `report.identification != Identification::Format` or `accepted_extensions.is_empty()` -> `ExtensionStatus::Undetermined` (evaluated *before* UTF-8 conversion).
     3. Non-UTF-8 extension on confirmed format -> `ExtensionStatus::Unrecognized`.
     4. UTF-8 match in `accepted_extensions` -> `ExtensionStatus::Match`.
     5. UTF-8 match in registry with different format -> `ExtensionStatus::Mismatch`.
     6. Otherwise -> `ExtensionStatus::Unrecognized`.
   - Tested in `test_compare_extension_pure_function` with non-UTF-8 extensions across Format, ContainerOnly, and Unknown reports.

6. **Binary Measurement Discrepancy Correction**:
   - Corrected release binary measurement records:
     - Sandbox Release Binary: 895,928 bytes (~875 KiB)
     - SHA256: `d1cc0fdd073cb9a52fa73a33fde6dd612d420308d29181204fd57aae6c1f6b68`
     - Verified consistency across filesystem (`stat`), sha256sum, and `.sandbox-test/results/release_measurements.json`.

---

## 8. 收尾交付状态 (Final Delivery Status)

**结论：P0 达标，已提交并推送；人工验收未完成；本轮暂停，不进入 P1。**

### 8.1 交付清单
- **变更范围**：57 个文件 / 6,287 行新增 —— `src/`(9)、`tests/`(6)、`fixtures/`(23)、`research/`(7)、`scripts/`(4)、`docs/`(1)、`.agents/progress/`(3)、根配置(4)。
- **明确排除**：`.sandbox-test/`（构建缓存与复验日志）、`target/`、临时/备份文件、Toss/WorDF/FClue 任何内容、规格 §11.4 本机语料、密钥与凭据。
- **Commit**：`fc3d77da4951219b92afea5913fb10d1333e5a25` → `origin/main` 推送成功。

### 8.2 门槛与三方结果
| 层级 | 执行者 | 结论 | 证据 |
|---|---|---|---|
| 实现者自测 | `kindprobe_agy_developer` | Round 1 五项修复自测通过 | 本文档 §7 |
| 独立实测 | `kindprobe_claude_verifier` | **PASS**，P0-G0..G6 全过 | `.sandbox-test/results/verifier_round1/INDEPENDENT_VERIFICATION_R1.md` |
| 整体审查 | `agy_whole_reviewer` (9890) | **无阻断性缺陷**，P0-G0..G6 全过 | `.agents/progress/FINAL_REVIEW.md` |
| 人工验收 | 维护者 | **未完成** | — |

### 8.3 未验证范围
1. Windows / macOS **未实机验证**（仅 Linux x86_64 `Debian trixie / Linux 6.6` 已验证）。
2. P1-B 格式递延：DOCM/XLSM/PPTM、XLSX/PPTX/XLS/PPT、7z/RAR、MP4/MP3/WAV/WebM。
3. `seek_count` 预算硬上限尚未引入。
4. `FINAL_REVIEW.md` 的 8 项 `inferred` 缺陷未修复（见置顶 D）。
5. 项目代码许可证待维护者指定。

### 8.4 人工验收命令
```bash
# 独立复验（宿主执行，走 Docker 合同）
python3 /srv/projects/rust/KindProbe/.sandbox-test/results/verifier_round1/run_verifier_round1.py
# 或
python3 /srv/projects/rust/KindProbe/scripts/verify_runner.py
```

### 8.5 下一步
等待维护者人工验收与许可指定，并由人类决定是否排期修复 `FINAL_REVIEW.md` §1 的 8 项缺陷 / 启动 P1。**代理不得代为标记验收通过，不自动推进 P1。**

