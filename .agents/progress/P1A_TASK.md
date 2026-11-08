# Current dispatch note

2026-11-03: Windows/model/original-session/Docker gates passed. User authorized P1-A. The prepared/not-dispatched and port-blocker statements below are HISTORICAL; this file is now the concrete assignment. Existing profile P0-only clauses describe the prior phase, superseded by this explicit P1-A user assignment; authentication/model/isolation and permission boundaries remain unchanged.

# KindProbe P1-A task — prepared, NOT dispatched

## User intent and baseline

User authorized proceeding after Round 2 independent verification passed. Next phase is spec v1.2 P1-A (minimum releasable CLI + Rust library), not P1-B/P2. Last independently checked remote commit: 892a40b715987d6fb804eb6bec5da45575b5dfb2. Check current Git state and preserve all existing edits before implementation.
Canonical project: /srv/projects/rust/KindProbe; remote https://github.com/vusiif/KindProbe.git.
This local task is prepared while the Windows startup gate is blocked; no supervisor/worker has been launched or messaged for P1-A.

## Goal and scope

Using docs/FILE_FORMAT_RUST_SPEC_V1_2.md §§6–9 and §11.3, stabilize the existing supported P0 formats into a documented v1 API/JSON/CLI contract on Linux x86_64. Do not add formats, GUI, batch scanning, renaming, conversion, Toss integration or claim Windows/macOS verification. Existing local corpora of Toss/WorDF/FClue remain out of scope.

MiMo coordinates; reuse current project agy implementation and Claude Code independent-verification sessions if actually alive. Codex is not a routine worker. Restore original supervisor native session ses_e6858a8a6ffeJlMl3k7VxSJWpY / terminal 0eb85957 only after verifying current identity/state; do not fork or create a substitute to bypass monitoring/network failure. Historical IDs require live-state checks.

## Deliverables and acceptance

1. First make a concise P1-A gap checklist against the actual Round 2 code and spec, distinguishing implemented, tested, missing and deliberately unsupported. Do not repeat completed P0 research or rely on the old whole-review 'no blocking defects' conclusion; it was disproven by Codex experiments.
2. Freeze/document the intended public Rust API, type/field semantics and compatibility boundary. Keep path/bytes/Read+Seek on one core. Confirm filename independence, extension precedence, non-UTF8 behavior, ordinary-file checks on the opened handle, typed I/O/argument errors, no library stdout/stderr/write/network side effects. Fix remaining in-scope contract defects rather than treating them as new formats.
3. JSON schema_version=1: documented/machine-checkable report/error shape, nullable fields and legal/illegal identification×inspection×verification combinations. Stable evidence/warning/error codes with trigger tests. Never assert complete structure/evidence that was not actually inspected; avoid paths/content leaks in report details.
4. CLI: single-file usage, text/JSON output, -- separator, help/version, exact exit-code and JSON envelope behavior for missing/unknown/limited/ambiguous/invalid arguments/real I/O/non-regular inputs. Tests must check the full envelope and relevant fields, not only substring/exit status.
5. Resource budgets: enumerate and enforce the spec's applicable source read, directory/entry/name/XML/decompression/depth/structure-capacity limits and explain configurable defaults. Missing hard limits or parser preallocation risks must be addressed or precisely scoped. Do not claim process-level time/memory guards are library guarantees. If a new bound would change compatibility materially, document the choice and tests; no arbitrary feature expansion.
6. Rules/dependencies: preserve approved version/feature/license/source decisions; verify embedded rule provenance and offline deterministic generation/hash where applicable. Do not import an unreviewed large rule database. Keep Cargo.lock. No automatic toolchain/provider/image upgrade.
7. Regression: preserve Codex acceptance counterexamples and their intent; all 18 checks, existing 52-fixture matrix (with corrected positive fixtures), and permanent in-repo tests must pass with false identification=0 for the fixed suite. Do not weaken expected results, delete failing tests or turn forged inputs into 'true' fixtures. Add only meaningful tests for the P1-A gaps.
8. Reproducibility: an ordinary checkout must contain the required test generators/manifests/expectations and documented regression commands; no dependency on another project's absolute paths or ignored historical .sandbox-test files for essential product tests. Public/legal synthetic fixtures may be committed with provenance; caches and transient evidence stay ignored.
9. Documentation: supported/container-only/not-supported/not-tested capability table, exact structure checks and limitations, user/API/JSON examples, dependency versions/licenses, build and regression instructions. Preserve honest Linux-only verification status. Update stale HANDOFF claims rather than retaining contradictory current summaries.
10. Real Linux release build/run with commands, compiler/target/features, source-read stats, binary size/hash and time. Measure target process memory if feasible; otherwise state not measured, never substitute docker client RSS. Do not claim bitwise reproducibility unless build paths/environment are controlled and actually tested.

## Validation and review flow

Implement/self-test -> Claude Code independent source review and actual targeted retests -> necessary fixes (default maximum two repair/review rounds for this new phase). P0 history remains separate; do not erase it. This is a new authorized phase, not an automatic extra P0 repair round.
On a stable completed P1-A milestone, arrange one stage-end whole-project source review using lolicon/9890 agy_whole_reviewer (verify health and read/report access first; target_host=http://127.0.0.1:9890, omit engine/use_worktree). Product source read-only; only specified review report writable, no builds/tests/network/delegation. Reuse existing same-project remote session via a verified follow-up interface when available; never duplicate assign to simulate follow-up. Confirm/triage reported in-scope blockers; suggestions are not work authorization.
Distinguish implementation self-report, independent real test, whole-source review, Codex verification and human acceptance. No claim of human acceptance by agents.

## Existing environment and authority

Every actual environment/client recovery must pass the updated cao-workflow Windows excluded-port/listener/Clash/Windows HTTP gates, native provider authenticated /models=200 with selected MiMo model, original session identity, proxy/NO_PROXY and output-monitor preflight. Model-list success does not establish completion quota. No account/model fallbacks, duplicate workers, global network changes or blind retries.
Build/test image: local/rust-test:1.99.1-trixie, ID sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3. Recheck locally before use; --pull never --network none --read-only --cap-drop ALL --security-opt no-new-privileges, non-root UID:GID + group1004, CPU/memory/PID/time bounds, named containers. Source read-only; only task-scoped project .sandbox-test output writable, bounded tmpfs, TMPDIR=/tmp. No home/auth/docker socket mount, toolchain install or native product-test fallback. Host Python can prepare synthetic inputs/control Docker; product/compiled probes run only inside the approved container.
Use the existing public dependency cache minimally; authenticated/native tool routing belongs to each original client, never copied across accounts. Missing tools/image/isolation/auth/quota/unknown completion -> checkpoint and report, no blind resubmit.
User's existing KindProbe commit/push authorization persists for verified in-scope changes; do not force-push or create Releases. Project license is not selected: do not assign one without the user. Code/documentation can advance with this unresolved publishing decision honestly recorded; do not claim an actual public-release gate passed while required licensing remains undecided.

## Handoff and stop

Use existing .agents/progress/HANDOFF.md with a short current P1-A status at top and links to evidence. Keep current profile/session/terminal IDs and the essential blockers/next step; do not create per-turn narrative logs. Save this task in the project's progress directory when environment gates allow.
Deliver gap checklist, changes, real verification and review evidence, unsupported/unverified scope, commit/push result and user acceptance steps. Finish P1-A and pause for human review. No automatic P1-B/new features.

## Present blocker (2026-11-03)

Windows gate ready=false: IPv4 and IPv6 reserved range 9849–9948 overlaps CAO 9888–9892. No CAO listener in that range was observed; Clash 41096 is listening via verge-mihomo. Dynamic TCP range observed 1024–15000. WSL relay and Docker Desktop processes exist; this does not prove agents are running. No WSL recovery, task dispatch, system network modification or restart was performed in this turn.
