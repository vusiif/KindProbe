#!/usr/bin/env python3
"""
Run multi-candidate baseline benchmarks on the fixed test manifest.
Evaluates:
- system `file` (v5.46)
- Siegfried (v1.11.9, PRONOM V125 + container-signature-20260119)
- infer (v0.22.0)
- file-format (v0.29.0)
"""

import json
import os
import subprocess
import sys

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MANIFEST_PATH = os.path.join(PROJECT_ROOT, "research", "benchmark-manifest.json")
SF_PATH = os.path.join(PROJECT_ROOT, ".sandbox-test", "research-tools", "siegfried", "sf")
SF_HOME = os.path.join(PROJECT_ROOT, ".sandbox-test", "research-tools", "siegfried", "home", "siegfried")
BASELINE_BIN = os.path.join(PROJECT_ROOT, ".sandbox-test", "target", "debug", "baseline-comparator")

def run_benchmarks():
    with open(MANIFEST_PATH, "r") as f:
        manifest = json.load(f)
        
    results = []
    
    for item in manifest:
        fpath = os.path.join(PROJECT_ROOT, item["path"])
        item_res = {
            "name": item["name"],
            "expected_format": item["expected_format"],
            "expected_container": item["expected_container"],
            "description": item["description"],
            "sha256": item["sha256"],
            "size": item["size"],
            "candidates": {}
        }
        
        # 1. file 5.46
        try:
            p = subprocess.run(["file", "-b", fpath], capture_output=True, text=True, timeout=5)
            out = p.stdout.strip()
            item_res["candidates"]["file_5_46"] = {
                "raw": out,
                "exit_code": p.returncode
            }
        except Exception as e:
            item_res["candidates"]["file_5_46"] = {"error": str(e)}
            
        # 2. Siegfried 1.11.9
        try:
            p = subprocess.run([SF_PATH, "-home", SF_HOME, "-json", fpath], capture_output=True, text=True, timeout=5)
            sf_json = json.loads(p.stdout)
            matches = sf_json.get("files", [{}])[0].get("matches", [])
            item_res["candidates"]["siegfried_1_11_9"] = {
                "matches": matches,
                "exit_code": p.returncode
            }
        except Exception as e:
            item_res["candidates"]["siegfried_1_11_9"] = {"error": str(e)}
            
        # 3. Rust baseline comparator (infer + file-format) via Docker
        # Using docker to maintain read-only sandbox execution contract
        cmd = [
            "docker", "run", "--rm", "--name", f"kindprobe-bench-{item['name'].replace('.', '_')}",
            "--pull", "never", "--network", "none", "--read-only",
            "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
            "--user", "1000:1000", "--group-add", "1004",
            "--cpus", "2", "--memory", "2g", "--pids-limit", "100",
            "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
            "-v", f"{PROJECT_ROOT}/fixtures:/workspace/fixtures:ro",
            "-v", f"{PROJECT_ROOT}/.sandbox-test:/output:rw",
            "local/rust-test:1.99.1-trixie",
            "/output/target/debug/baseline-comparator",
            f"/workspace/fixtures/{item['name']}"
        ]
        try:
            p = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            if p.returncode == 0:
                cmp_data = json.loads(p.stdout.strip())
                item_res["candidates"]["infer_0_22_0"] = {
                    "extension": cmp_data.get("infer_result"),
                    "mime": cmp_data.get("infer_mime"),
                }
                item_res["candidates"]["file_format_0_29_0"] = {
                    "name": cmp_data.get("file_format_result"),
                    "extension": cmp_data.get("file_format_ext"),
                    "media_type": cmp_data.get("file_format_media")
                }
            else:
                item_res["candidates"]["rust_comparator_error"] = {
                    "exit_code": p.returncode,
                    "stderr": p.stderr.strip()
                }
        except Exception as e:
            item_res["candidates"]["rust_comparator_error"] = {"error": str(e)}
            
        results.append(item_res)
        
    out_path = os.path.join(PROJECT_ROOT, "research", "baseline-results.json")
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Benchmark completed for {len(results)} items. Saved to {out_path}")

if __name__ == "__main__":
    run_benchmarks()
