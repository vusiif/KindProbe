# Windows migration — 2026-11-20

- Canonical repository: `C:/Users/IceLolly/Music/Rust/KindProbe`.
- Migrated locally with Git bundle plus the original uncommitted patch. No commit or push was made.
- The WSL project copy was removed after full source/evidence verification; old WSL paths below are historical.
- Full prior evidence and Git recovery files: `C:/Users/IceLolly/Music/CAO/wsl-migration-20261120/KindProbe`.
- Latest candidate and independent Windows report are available under this repository's `.sandbox-test/results/`.
- Migration does not change product acceptance: real Explorer drag-and-drop is still awaiting user confirmation; no tests or VM operations were rerun.
- The user explicitly authorized committing and pushing the preserved changes on 2026-11-20; this supersedes the historical commit/push hold below, while manual acceptance remains pending.

## Preserved handoff

# KindProbe current handoff

## Current state (2026-11-11, America/Los_Angeles)
- **PAUSED by user request (2026-11-11, America/Los_Angeles).** Supervisor 19a395e0 and verifier 14abd6b3 interrupted after delivery. Preserve source, candidate and evidence; no more VM operations, tests, fixes or commit/push until user resumes. Verifier reported PASS; actual Explorer drag-and-drop remains unverified.
- **VERIFIER PASS (2026-11-11, terminal 14abd6b3, kindprobe_agy_verifier, independent session).** Remaining automated Windows gate is satisfied; only **user manual drag-and-drop acceptance** and the user's commit decision remain. Supervisor 19a395e0 cross-checked archived evidence (did not rerun anything): `.sandbox-test/results/locale-57a9914c/guest-product-receipt.json` GUID `9d857352d9474648a8ddab4415a0d30a`, success=true, sha256 `57a9914c…bcc7` hash_verified=true, native `kernel32!GetUserDefaultUILanguage()`=1033 (0x0409, not Chinese → English expected and observed); runs: dedicated success exit 0 was_waiting=true language match, standard CLI exit 0, CLI error exit 4 `io.open_failed` localized, dedicated error exit 4 localized + pause verified, harness error branch writes a fresh-GUID failure receipt. Screenshot `kindprobe-real-ui-console.png` (81,679 B) verified visually: readable non-black "KindProbe - Dedicated Console" window with full inspection output on Tiny10 desktop. Report: `.sandbox-test/results/verifier_win_acceptance/VERIFIER_REPORT.md`/`.json`.
- Verifier run details: bounded `Check` exit 0 (tools running, auth authenticated); pre-restore evidence exported first (`guest-product-receipt-pre-restore.json` 2,804 B, `run-guest-log-pre-restore.txt` 120 B); `Restore` clean-base exit 0; all adapters Offline enforced/verified (epoch `191876ca-f2c6-4886-9784-c0f28760a193`, offlineVerified=true, networkMode=Offline); staged+repo candidate hashes PASS (2,158,322 B).
- **Screenshot budget exhausted: 0 captures remain** (≥3 used across restores, ≥2 black frames). No further screenshots without explicit user authorization.
- **Limitation to disclose at acceptance:** the dedicated-console runs in the receipt/screenshot were launched by the harness with `KINDPROBE_TEST_DEDICATED_CONSOLE=1` (simulated drag-drop semantics: pause + localized prompt verified). A real Explorer drag→exe by the user is still required and is a human step agents cannot perform.
- **Not yet authorized:** commit/push stays blocked until the user completes manual drag acceptance or explicitly decides otherwise. Preserve GPL-3.0-or-later and the four dirty files (src/main.rs, tests/test_cli_contract.rs, README.md, .agents/progress/HANDOFF.md). No source changes (no new product defect), no new builds/tests.
- Historical (paused attempt by 7fbc250a, superseded): Tools/auth/Restore/Offline had PASSED this attempt (state epoch 01ab188b-c711-487e-b762-6a89c6c284de, offlineVerified=true, networkMode=Offline, snapshot=clean-base, updatedAt 07:55:54). Log tail: CopyFrom receipt ok 08:08:45 → Run ok 08:09:20 → Screenshot ok 08:09:50 → run-guest-log export 08:13:16 → vmrun reported `virtual machine is not powered on` 08:15:27 (VM externally powered off; **guest internal state UNKNOWN/UNREACHABLE — launcher exit 0 and host-side VM-off do not prove guest processes stopped**). Evidence kept in place under `C:\Users\IceLolly\Desktop\Others\vm-test-results\KindProbe-locale-57a9914c\`: guest-product-receipt.json (2,804 B, guest-exported: candidate hash verified, English UI text, exit 0, wait state), run-guest-log.txt (120 B, prior run SUCCESS), kindprobe-real-ui-console.png (2,367 B — **black frame, not readable interaction evidence**), staging/. WSL-side: no in-flight pwsh/vmrun processes; Docker 0 running (7 old verifier containers Exited 3 weeks ago). This is NOT a PASS: readable screenshot + user drag acceptance still pending; no commit/push was made.
- Retracted hypothesis: run-guest-verification.ps1 encoding was NOT defective (byte check: UTF8 BOM=true). Only two harness issues were confirmed from source: error branch wrote no fresh failure receipt; language probe used culture objects instead of the product's GetUserDefaultUILanguage native API. Do not revive the encoding theory on resume.
- Post-stop Check (08:19:04, after the VM-off observation at 08:15:27; arrived during pause, no follow-up taken): `-Operation Check` exit 4, verbatim `vmware-clean-test: tools not ready within 60s (last state: installed)` — consistent with the externally powered-off VM, not a product defect. State file still shows epoch 01ab188b…, updatedAt 07:55:54 (stale; offlineVerified=true refers to the 07:55 Restore, not 08:19). Worker stopped per rules: no retry, no re-dispatch, no workaround; recommends the user manually confirm the Tiny10 desktop and VMware Tools service are ready before any resume.
- History: a pause (7fbc250a) superseded an earlier resume; the current explicit resume by the user (supervisor 19a395e0) supersedes that pause.
- Product implementation and independent automated verification are complete. No new product work or test rerun is needed without a concrete defect.
- CAO Windows port/HTTP startup preflight passed for the current attempt. The previous localhost forwarding block is resolved; the remaining block is Tiny10 VMware Tools/VIX readiness.
- Current identities: supervisor **19a395e0**; verifier **14abd6b3** (kindprobe_agy_verifier, delivered PASS — retained pending human acceptance). Old 7fbc250a/f0faf142 terminated with the WSL restart — never message them. Previous native histories and all evidence remain preserved.

## Scope and source baseline
- Repository: /srv/projects/rust/KindProbe; remote: https://github.com/vusiif/KindProbe.git.
- HEAD: c03cf04a0bd303ba5f3c5c07179c647a3959eadc; license: GPL-3.0-or-later.
- Existing uncommitted changes: src/main.rs, tests/test_cli_contract.rs, README.md, and this handoff. Preserve them; do not reset/stash.
- Authorized features: Windows single-file drag-to-exe / readable dedicated console, plus automatic Chinese UI in Chinese environments and English otherwise. No new formats, GUI or P1-B.
- JSON/library contracts and exits 0/2/3/4/1 remain unchanged; no pause or extra prose in JSON/pipes/ordinary terminal calls.

## Completed evidence
- Independent locale verification: cargo tests 39/39, fixture matrix 52/52, Codex acceptance 18/18; Docker runs reported exit 0 and supervisor checked evidence.
- Earlier drag-drop independent Phase A PASS; historical VM interaction matrix 10/10 PASS. These results do not prove the new Windows UI-language call was exercised in a real Chinese guest.
- Windows candidate: .sandbox-test/results/locale-57a9914c/kindprobe.exe; 2,158,322 bytes.
- SHA256: 57a9914c56fdfb92492352f16a1f34dfe09961906be4eb4c7321a876acd5bcc7.
- Manifest: .sandbox-test/results/locale-57a9914c/manifest.json and manifest.md.
- Candidate delivered to Windows: C:/Users/IceLolly/Desktop/Others/vm-test-results/KindProbe-locale-57a9914c/staging/kindprobe.exe; copied and hash-verified this recovery, not executed on host.
- Old candidates 2295c8f5 (drag-drop) and 2de62492 (initial Windows) and their host evidence are unchanged.

## Remaining gates and unique next step
- Pending: real Windows UI-language/runtime presentation and an actual readable interaction screenshot, followed by user drag-and-drop acceptance.
- Prior VM gate failed: VMware Tools/VIX remained installed, not running. Do not claim guest readiness or trust stale receipts.
- Bounded verifier re-run (2026-11-11, kindprobe_agy_verifier terminal f0faf142) reproduced the block: `Invoke-Tiny10.ps1 -Operation Restore` exit 4, verbatim `vmware-clean-test: tools not ready within 180s (last state: installed)`; log entry `logs/vm10-operations.jsonl` status=fail exitCode=4; state epoch 94ce335e-ecf9-4c7b-9be6-a2d624b8aee0, offlineVerified=false, networkMode=Unknown, note="restore in progress", snapshot=clean-base, updatedAt 2026-11-11T05:52:33-08:00. Staging hash PASS both copies (57a9914c…bcc7, 2,158,322 bytes). 0 screenshots used; guest product receipt none (channel blocked before transfer/execution). Report: .sandbox-test/results/verifier_win_acceptance/VERIFIER_REPORT.md and .json. Verifier stopped per failure rule: no retry, no re-dispatch, no workaround.
- **Next:** unblock the Tiny10 guest VMware Tools/VIX channel outside this verification loop (host/VM-level fix is a human/infra step, not agent-repairable under current constraints), then arrange only the remaining bounded VM check/evidence task. Do not restart implementation, reacquire toolchains or repeat 39/52/18/10 merely to rebuild context.
- VM actions only through vmware-clean-test Invoke-Tiny10.ps1. If needed Restore clean-base, then verify Tools/auth, all-adapter Offline and staging hashes before Run/CopyTo. No direct vmrun workaround, credential reads, VMX edits or snapshot creation/deletion.
- Screenshot budget does not reset across sessions: one historical black frame used; at most two further captures under the existing task budget. Environment failure gets a concrete blocked report, not a retry loop.
- Only after required combined validation or an explicit user decision on the remaining gate may the already-authorized commit/push proceed; no force push or Release. Do not claim the current uncommitted features are on GitHub.

## Execution contract and identities
- Reuse current role profiles; supervisor coordinates, agy implements, separate agy session verifies. Original project histories remain available; their IDs are historical, not proof of live execution.
- Existing approved images: local/rust-test:1.99.1-trixie and local/kindprobe-winbuild:1.99.1. Product build/test stays offline, source-read-only, nonroot, cap-drop ALL, no-new-privileges and bounded resources; only project .sandbox-test outputs writable.
- User approved deployed agy adapter tool-approval behavior; other interaction prompts still checked individually. This is not OS isolation of all agy tools.
- Timer/ACK/automatic recovery were not deployed by the skill update; do not promise automatic wakeup or treat terminal completion as task completion.

## History
- Full pre-compaction handoff, including old stage evidence, pause records and identities, is preserved locally at: .sandbox-test/results/handoff-history/HANDOFF-before-20261111T050925.md.
- The archive is ignored and is not an extra artifact to commit. Git also retains earlier committed history. Read history only for a specific contradiction, not on every resume.
