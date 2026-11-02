#!/usr/bin/env python3
"""
Run release binary measurements inside Docker sandbox according to specification §10.2 and §11.2 P0-G5.
Records platform, toolchain, target, dependency count, binary size, source_bytes_read, seek_count,
decompressed_bytes, engine execution time, CLI process execution time, peak memory (RSS), and exit code.
"""

import json
import os
import subprocess
import sys
import time

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MANIFEST_PATH = os.path.join(PROJECT_ROOT, "research", "benchmark-manifest.json")
RESULTS_DIR = os.path.join(PROJECT_ROOT, ".sandbox-test", "results")
RELEASE_BIN = os.path.join(PROJECT_ROOT, ".sandbox-test", "target", "release", "kindprobe")

def run_measurements():
    os.makedirs(RESULTS_DIR, exist_ok=True)
    with open(MANIFEST_PATH, "r") as f:
        manifest = json.load(f)

    # 1. Environment and build metadata
    bin_stat = os.stat(RELEASE_BIN)
    bin_size = bin_stat.st_size

    with open(RELEASE_BIN, "rb") as f:
        import hashlib
        bin_sha256 = hashlib.sha256(f.read()).hexdigest()

    meta = {
        "platform": "Linux 6.6.137+ x86_64",
        "container_image": "local/rust-test:1.99.1-trixie",
        "container_image_id": "sha256:cdf54f8f43b6987e28d89c2d9fe0548bfdfbcd3db227b5ac20ca45e7f8330fe3",
        "rust_toolchain": "rustc 1.99.1 (58cf8f9c3 2026-10-13)",
        "target_triple": "x86_64-unknown-linux-gnu",
        "build_profile": "release",
        "binary_size_bytes": bin_size,
        "binary_sha256": bin_sha256,
        "features": ["zip/deflate"],
        "dependencies_count": 43, # from Cargo.lock
    }

    results = []
    false_identification_count = 0
    missed_count = 0
    correct_degradation_count = 0
    crash_count = 0

    for item in manifest:
        name = item["name"]
        expected_format = item["expected_format"]
        expected_container = item["expected_container"]
        fixture_path = f"/workspace/fixtures/{name}"

        container_name = f"kindprobe-rel-{name.replace('.', '_')}"

        # We run /usr/bin/time -v on docker command to measure peak RSS and execution time
        cmd = [
            "/usr/bin/time", "-v",
            "docker", "run", "--rm", "--name", container_name,
            "--pull", "never", "--network", "none", "--read-only",
            "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
            "--user", "1000:1000", "--group-add", "1004",
            "--cpus", "2", "--memory", "2g", "--pids-limit", "100",
            "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
            "-v", f"{PROJECT_ROOT}:/workspace:ro",
            "-v", f"{PROJECT_ROOT}/.sandbox-test:/output:rw",
            "local/rust-test:1.99.1-trixie",
            "/output/target/release/kindprobe", "--json", fixture_path
        ]

        t0 = time.perf_counter()
        p = subprocess.run(cmd, capture_output=True, text=True)
        t_cli_total_us = int((time.perf_counter() - t0) * 1_000_000)

        stdout = p.stdout.strip()
        stderr = p.stderr.strip()

        # Parse /usr/bin/time output from stderr
        peak_rss_kb = None
        for line in stderr.splitlines():
            if "Maximum resident set size" in line:
                try:
                    peak_rss_kb = int(line.split(":")[-1].strip())
                except Exception:
                    pass

        try:
            report = json.loads(stdout)
            format_id = report.get("format_id")
            container_id = report.get("container_id")
            identification = report.get("identification")
            inspection = report.get("inspection")
            stats = report.get("stats", {})

            # Evaluation check
            is_false_id = False
            is_miss = False
            is_correct_deg = False

            if expected_format is not None:
                if format_id == expected_format:
                    pass
                elif format_id is not None:
                    is_false_id = True
                    false_identification_count += 1
                else:
                    is_miss = True
                    missed_count += 1
            else:
                # Expected format is None (unknown or container only)
                if format_id is not None:
                    is_false_id = True
                    false_identification_count += 1
                else:
                    is_correct_deg = True
                    correct_degradation_count += 1

            record = {
                "fixture_name": name,
                "expected_format": expected_format,
                "expected_container": expected_container,
                "command": " ".join(cmd),
                "exit_code": p.returncode,
                "stdout_json": report,
                "peak_resident_set_size_kb": peak_rss_kb,
                "source_bytes_read": stats.get("source_bytes_read"),
                "seek_count": stats.get("seek_count"),
                "entries_inspected": stats.get("entries_inspected"),
                "decompressed_bytes": stats.get("decompressed_bytes"),
                "engine_duration_micros": stats.get("duration_micros"),
                "cli_total_duration_micros": t_cli_total_us,
                "is_false_identification": is_false_id,
                "is_miss": is_miss,
                "is_correct_degradation": is_correct_deg,
            }
            results.append(record)

        except Exception as e:
            crash_count += 1
            results.append({
                "fixture_name": name,
                "expected_format": expected_format,
                "command": " ".join(cmd),
                "exit_code": p.returncode,
                "stdout_raw": stdout,
                "stderr_raw": stderr,
                "error": str(e),
                "crash": True
            })

    output_data = {
        "metadata": meta,
        "summary": {
            "total_fixtures": len(manifest),
            "false_identifications": false_identification_count,
            "missed_identifications": missed_count,
            "correct_degradations": correct_degradation_count,
            "crashes": crash_count,
        },
        "records": results
    }

    out_file = os.path.join(RESULTS_DIR, "p0_matrix_results.json")
    with open(out_file, "w") as f:
        json.dump(output_data, f, indent=2)

    measurements_file = os.path.join(RESULTS_DIR, "release_measurements.json")
    with open(measurements_file, "w") as f:
        json.dump(output_data, f, indent=2)

    print(f"Release measurements completed for {len(results)} items.")
    print(f"Summary: false_id={false_identification_count}, missed={missed_count}, correct_deg={correct_degradation_count}, crashes={crash_count}")
    print(f"Saved to {out_file} and {measurements_file}")

if __name__ == "__main__":
    run_measurements()
