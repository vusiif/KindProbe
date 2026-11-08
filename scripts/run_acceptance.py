#!/usr/bin/env python3
"""
KindProbe P1-A In-Repo Codex 18-Item Acceptance Test Runner
Executes the full 18 acceptance checks (9 positive/baseline + 9 adversarial regression)
inside the approved Docker test container using repo-tracked fixtures and harness.
"""

from pathlib import Path
import json, subprocess, os, hashlib, time, shutil, sys

PROJECT_ROOT = Path(__file__).resolve().parent.parent
IMAGE = "local/rust-test:1.99.1-trixie"
EXPECTED_IMAGE_ID = "sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3"

def main():
    print("==================================================================")
    print("       KindProbe P1-A Codex 18-Item Acceptance Suite              ")
    print("==================================================================")

    # 1. Image Check
    p = subprocess.run(["docker", "image", "inspect", IMAGE, "--format", "{{.Id}} {{.Os}}/{{.Architecture}}"], capture_output=True, text=True)
    if p.returncode != 0 or not p.stdout.startswith(EXPECTED_IMAGE_ID):
        print(f"[FAIL] Image mismatch: {p.stdout.strip()}")
        sys.exit(1)
    print(f"[PASS] Docker image ID verified: {EXPECTED_IMAGE_ID}")

    # Output directory in .sandbox-test/results
    out_dir = PROJECT_ROOT / ".sandbox-test/results" / ("codex-acceptance-run-" + time.strftime("%Y%m%dT%H%M%S", time.gmtime()))
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "fixtures").mkdir(exist_ok=True)

    base_docker = [
        "docker", "run", "--rm",
        "--pull", "never", "--network", "none", "--read-only",
        "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
        "--user", f"{os.getuid()}:{os.getgid()}", "--group-add", "1004",
        "--cpus", "2", "--memory", "2g", "--pids-limit", "128",
        "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
        "--tmpfs", "/cargo:rw,nosuid,size=64m,mode=1777",
        "-v", f"{PROJECT_ROOT}:/workspace:ro",
        "-v", f"{out_dir}:/accept:rw",
        "-v", f"{PROJECT_ROOT}/.sandbox-test/cargo-home/registry:/cargo/registry:ro",
        "-e", "CARGO_HOME=/cargo",
        "-e", "CARGO_TARGET_DIR=/accept/build",
        "-e", "CARGO_INCREMENTAL=0",
        "-e", "TMPDIR=/tmp",
        "-w", "/workspace",
        IMAGE
    ]

    def run_stage(stage, args, timeout=300, check=True):
        c_name = f"kindprobe-accept-{stage}"
        cmd = ["docker", "run", "--rm", "--name", c_name,
               "--pull", "never", "--network", "none", "--read-only",
               "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
               "--user", f"{os.getuid()}:{os.getgid()}", "--group-add", "1004",
               "--cpus", "2", "--memory", "2g", "--pids-limit", "128",
               "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
               "--tmpfs", "/cargo:rw,nosuid,size=64m,mode=1777",
               "-v", f"{PROJECT_ROOT}:/workspace:ro",
               "-v", f"{out_dir}:/accept:rw",
               "-v", f"{PROJECT_ROOT}/.sandbox-test/cargo-home/registry:/cargo/registry:ro",
               "-e", "CARGO_HOME=/cargo",
               "-e", "CARGO_TARGET_DIR=/accept/build",
               "-e", "CARGO_INCREMENTAL=0",
               "-e", "TMPDIR=/tmp",
               "-w", "/workspace",
               IMAGE] + args
        started = time.monotonic()
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        elapsed = round(time.monotonic() - started, 3)
        (out_dir / f"{stage}.stdout.log").write_text(p.stdout)
        (out_dir / f"{stage}.stderr.log").write_text(p.stderr)
        if check and p.returncode != 0:
            print(f"[FAIL] {stage} failed with code {p.returncode}")
            print(p.stderr)
            sys.exit(1)
        print(f"[PASS] Stage '{stage}' ({elapsed}s, exit {p.returncode})")
        return p

    # Build release
    run_stage("release-build", ["cargo", "build", "--release", "--offline", "--locked", "--manifest-path", "/workspace/Cargo.toml"])

    # Compile probe
    deps_dir = out_dir / "build/release/deps"
    def find_rlib(name):
        files = list(deps_dir.glob(f"lib{name}-*.rlib"))
        if len(files) != 1:
            raise RuntimeError(f"Ambiguous or missing rlib for {name}: {files}")
        return f"/accept/build/release/deps/{files[0].name}"

    kindprobe_rlib = find_rlib("kindprobe")
    cfb_rlib = find_rlib("cfb")
    serde_json_rlib = find_rlib("serde_json")

    run_stage("probe-compile", [
        "rustc", "--edition=2021",
        "/workspace/tests/codex_acceptance/probe.rs",
        "-L", "dependency=/accept/build/release/deps",
        "--extern", f"kindprobe={kindprobe_rlib}",
        "--extern", f"cfb={cfb_rlib}",
        "--extern", f"serde_json={serde_json_rlib}",
        "-o", "/accept/probe"
    ])

    # Run probe (7 API & CFB cases)
    probe_res = run_stage("probe-run", ["/accept/probe"])
    api_cfb_rows = [json.loads(line) for line in probe_res.stdout.splitlines() if line.strip()]
    (out_dir / "api-cfb-cases.json").write_text(json.dumps(api_cfb_rows, indent=2))

    # Run CLI cases (11 cases: 7 docx XML cases + 4 CLI envelope/flag cases)
    cases_script = (PROJECT_ROOT / "tests/codex_acceptance/cases.py").read_text()
    cases_script = cases_script.replace("root=Path('/accept')", f"root=Path({repr(str(out_dir))})")
    cases_script = cases_script.replace("proc=subprocess.run([str(exe),'--json',str(p)],capture_output=True,text=True,timeout=5)", "proc=invoke(['--json', '/accept/fixtures/' + p.name])")
    cases_script = cases_script.replace("proc=subprocess.run([str(exe)]+args,capture_output=True,text=True,timeout=5)", "proc=invoke(args)")
    cases_script = cases_script.replace("p=subprocess.run([str(exe)]+args,capture_output=True,text=True,timeout=5)", "p=invoke(args)")

    idx = [0]
    def invoke(args):
        idx[0] += 1
        return run_stage(f"cli-{idx[0]}", ["/accept/build/release/kindprobe"] + args, check=False)

    exec(compile(cases_script, "cases.py", "exec"), {"invoke": invoke})

    cli_rows = json.loads((out_dir / "cli-cases.json").read_text())
    all_cases = api_cfb_rows + cli_rows

    total = len(all_cases)
    passed = sum(1 for r in all_cases if r.get("pass"))
    failed = total - passed

    summary = {
        "total": total,
        "passed": passed,
        "failed": failed,
        "failed_ids": [r.get("id") for r in all_cases if not r.get("pass")],
        "conclusion": "PASS" if failed == 0 else "FAIL"
    }
    (out_dir / "acceptance_summary.json").write_text(json.dumps(summary, indent=2))
    print("------------------------------------------------------------------")
    print(f"Codex 18-Item Acceptance Result: {summary['conclusion']} ({passed}/{total} passed)")
    print("------------------------------------------------------------------")
    if failed > 0:
        print(f"Failed cases: {summary['failed_ids']}")
        sys.exit(1)

if __name__ == "__main__":
    main()
