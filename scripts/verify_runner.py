#!/usr/bin/env python3
"""
KindProbe P0 Independent Verification Runner
Can be executed independently by verifier or CI to validate all P0 gates (P0-G0 to P0-G6).

Execution Contract:
- Local Docker image: local/rust-test:1.99.1-trixie
- Expected Image ID: sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3
- Sandbox flags: --pull never --network none --read-only --cap-drop ALL --security-opt no-new-privileges --user 1000:1000 --group-add 1004
"""

import json
import os
import subprocess
import sys
import time

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
EXPECTED_IMAGE = "local/rust-test:1.99.1-trixie"
EXPECTED_IMAGE_ID = "sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3"

def run_command(cmd, description):
    print(f"[RUN] {description}: {' '.join(cmd)}")
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        print(f"[FAIL] {description} exited with code {p.returncode}")
        print(f"Stdout:\n{p.stdout}")
        print(f"Stderr:\n{p.stderr}")
    else:
        print(f"[PASS] {description}")
    return p

def main():
    print("==================================================================")
    print("      KindProbe P0 Independent Verification Runner                ")
    print("==================================================================")

    # 1. Image Check
    p = subprocess.run(["docker", "image", "inspect", EXPECTED_IMAGE, "--format", "{{.Id}}"], capture_output=True, text=True)
    if p.returncode != 0 or EXPECTED_IMAGE_ID not in p.stdout:
        print(f"[FAIL] Image ID mismatch or image not found: {p.stdout.strip()}")
        sys.exit(1)
    print(f"[PASS] Docker image ID verified: {EXPECTED_IMAGE_ID}")

    # 2. Cargo Test in Sandbox
    test_cmd = [
        "docker", "run", "--rm", "--name", "kindprobe-verify-test",
        "--pull", "never", "--network", "none", "--read-only",
        "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
        "--user", "1000:1000", "--group-add", "1004",
        "--cpus", "2", "--memory", "2g", "--pids-limit", "100",
        "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
        "-v", f"{PROJECT_ROOT}:/workspace:ro",
        "-v", f"{PROJECT_ROOT}/.sandbox-test:/output:rw",
        EXPECTED_IMAGE,
        "cargo", "test", "--offline", "--manifest-path", "/workspace/Cargo.toml"
    ]
    p_test = run_command(test_cmd, "Run test suite in offline sandbox")
    if p_test.returncode != 0:
        sys.exit(1)

    # 3. Release Build in Sandbox
    release_cmd = [
        "docker", "run", "--rm", "--name", "kindprobe-verify-release",
        "--pull", "never", "--network", "none", "--read-only",
        "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
        "--user", "1000:1000", "--group-add", "1004",
        "--cpus", "2", "--memory", "2g", "--pids-limit", "100",
        "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
        "-v", f"{PROJECT_ROOT}:/workspace:ro",
        "-v", f"{PROJECT_ROOT}/.sandbox-test:/output:rw",
        EXPECTED_IMAGE,
        "cargo", "build", "--release", "--offline", "--manifest-path", "/workspace/Cargo.toml"
    ]
    p_rel = run_command(release_cmd, "Build release binary in offline sandbox")
    if p_rel.returncode != 0:
        sys.exit(1)

    # 4. Measure fixtures using release binary
    measure_cmd = [sys.executable, os.path.join(PROJECT_ROOT, "scripts", "run_release_measurements.py")]
    p_meas = run_command(measure_cmd, "Run release matrix measurements")
    if p_meas.returncode != 0:
        sys.exit(1)

    # Check results file
    res_path = os.path.join(PROJECT_ROOT, ".sandbox-test", "results", "p0_matrix_results.json")
    with open(res_path) as f:
        data = json.load(f)

    summary = data.get("summary", {})
    false_ids = summary.get("false_identifications", 0)
    misses = summary.get("missed_identifications", 0)
    crashes = summary.get("crashes", 0)

    print("------------------------------------------------------------------")
    print(f"Summary Verification: false_id={false_ids}, misses={misses}, crashes={crashes}")
    print("------------------------------------------------------------------")

    if false_ids > 0:
        print(f"[FAIL] P0-G2 FAILED: False identifications count is {false_ids} (must be 0)")
        sys.exit(1)

    if misses > 0 or crashes > 0:
        print(f"[FAIL] Misses ({misses}) or Crashes ({crashes}) encountered")
        sys.exit(1)

    print("[SUCCESS] All P0 Gates verified successfully!")
    sys.exit(0)

if __name__ == "__main__":
    main()
