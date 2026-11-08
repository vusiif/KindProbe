#!/usr/bin/env python3
"""
KindProbe P1-A In-Repo 52-Fixture Regression Suite
Executes the full 52-fixture matrix (23 dev + 26 verifier + 3 round1)
inside the approved Docker test container using repo-tracked fixtures.
"""

import json
import os
import subprocess
import sys
import time

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DOCKER_IMAGE = "local/rust-test:1.99.1-trixie"

def docker_exec(container_name, binary_args):
    cmd = [
        "docker", "run", "--rm",
        "--name", container_name,
        "--pull", "never",
        "--network", "none",
        "--read-only",
        "--cap-drop", "ALL",
        "--security-opt", "no-new-privileges",
        "--user", f"{os.getuid()}:{os.getgid()}",
        "--group-add", "1004",
        "--cpus", "2",
        "--memory", "2g",
        "--pids-limit", "100",
        "--tmpfs", "/tmp:rw,nosuid,size=256m,mode=1777",
        "-e", "TMPDIR=/tmp",
        "-v", f"{PROJECT_ROOT}:/workspace:ro",
        "-v", f"{PROJECT_ROOT}/.sandbox-test:/output:rw",
        DOCKER_IMAGE,
        "/output/target/release/kindprobe",
    ] + binary_args
    return subprocess.run(cmd, capture_output=True, text=True)

def main():
    print("==================================================================")
    print("          KindProbe P1-A 52-Fixture Regression Suite              ")
    print("==================================================================")

    # 1. 23 developer fixtures (in fixtures/)
    dev_fixtures = [
        {"name": "valid_png.png", "path": "/workspace/fixtures/valid_png.png", "exp_fmt": "png", "exp_code": 0},
        {"name": "valid_pdf.pdf", "path": "/workspace/fixtures/valid_pdf.pdf", "exp_fmt": "pdf", "exp_code": 0},
        {"name": "plain_zip.zip", "path": "/workspace/fixtures/plain_zip.zip", "exp_fmt": "zip", "exp_code": 0},
        {"name": "true_docx.docx", "path": "/workspace/fixtures/true_docx.docx", "exp_fmt": "docx", "exp_code": 0},
        {"name": "true_docx_rel_path.docx", "path": "/workspace/fixtures/true_docx_rel_path.docx", "exp_fmt": "docx", "exp_code": 0},
        {"name": "true_docx_strict.docx", "path": "/workspace/fixtures/true_docx_strict.docx", "exp_fmt": "docx", "exp_code": 0},
        {"name": "true_doc.doc", "path": "/workspace/fixtures/true_doc.doc", "exp_fmt": "doc", "exp_code": 0},
        {"name": "plain_cfb.cfb", "path": "/workspace/fixtures/plain_cfb.cfb", "exp_fmt": None, "exp_code": 3},
        {"name": "fake_docx_zip.zip", "path": "/workspace/fixtures/fake_docx_zip.zip", "exp_fmt": None, "exp_code": 3},
        {"name": "fake_doc_cfb.doc", "path": "/workspace/fixtures/fake_doc_cfb.doc", "exp_fmt": None, "exp_code": 3},
        {"name": "partial_ooxml_missing_rels.zip", "path": "/workspace/fixtures/partial_ooxml_missing_rels.zip", "exp_fmt": None, "exp_code": 3},
        {"name": "partial_ooxml_missing_main_part.zip", "path": "/workspace/fixtures/partial_ooxml_missing_main_part.zip", "exp_fmt": None, "exp_code": 3},
        {"name": "budget_xml_deep.docx", "path": "/workspace/fixtures/budget_xml_deep.docx", "exp_fmt": "docx", "exp_code": 3},
        {"name": "cfb_cyclic_chain.doc", "path": "/workspace/fixtures/cfb_cyclic_chain.doc", "exp_fmt": None, "exp_code": 3},
        {"name": "mismatched_png.pdf", "path": "/workspace/fixtures/mismatched_png.pdf", "exp_fmt": "png", "exp_code": 0},
        {"name": "mismatched_docx.zip", "path": "/workspace/fixtures/mismatched_docx.zip", "exp_fmt": "docx", "exp_code": 0},
        {"name": "mismatched_doc.docx", "path": "/workspace/fixtures/mismatched_doc.docx", "exp_fmt": "doc", "exp_code": 0},
        {"name": "truncated_png.png", "path": "/workspace/fixtures/truncated_png.png", "exp_fmt": None, "exp_code": 3},
        {"name": "truncated_pdf.pdf", "path": "/workspace/fixtures/truncated_pdf.pdf", "exp_fmt": None, "exp_code": 3},
        {"name": "truncated_zip.zip", "path": "/workspace/fixtures/truncated_zip.zip", "exp_fmt": None, "exp_code": 3},
        {"name": "truncated_cfb.doc", "path": "/workspace/fixtures/truncated_cfb.doc", "exp_fmt": None, "exp_code": 3},
        {"name": "empty_file.bin", "path": "/workspace/fixtures/empty_file.bin", "exp_fmt": None, "exp_code": 3},
        {"name": "random_garbage.bin", "path": "/workspace/fixtures/random_garbage.bin", "exp_fmt": None, "exp_code": 3},
    ]

    all_items = []
    for item in dev_fixtures:
        item["suite"] = "developer_fixtures"
        all_items.append(item)

    # 2. 29 verifier adversarial fixtures (26 verifier + 3 round1 in fixtures/verifier/ and fixtures/verifier_round1/)
    vfy_manifest_path = os.path.join(PROJECT_ROOT, "fixtures", "verifier_manifest.json")
    with open(vfy_manifest_path) as f:
        vfy_manifest = json.load(f)

    for item in vfy_manifest:
        all_items.append({
            "suite": "verifier_fixtures",
            "name": item["name"],
            "path": f"/workspace/{item['path']}",
            "exp_fmt": item["expected_format"],
            "exp_code": item["expected_exit_code"],
        })

    print(f"Total fixtures to verify: {len(all_items)}")
    assert len(all_items) == 52, f"Expected 52 fixtures, got {len(all_items)}"

    false_ids = 0
    misses = 0
    correct_degradations = 0
    crashes = 0
    records = []

    for idx, item in enumerate(all_items):
        name = item["name"]
        suite = item["suite"]
        cont_path = item["path"]
        exp_fmt = item["exp_fmt"]
        exp_code = item["exp_code"]

        c_name = f"kp-p1a-52reg-{idx}-{name.replace('.', '_')[:10]}"
        res = docker_exec(c_name, ["--json", cont_path])

        json_obj = None
        try:
            json_obj = json.loads(res.stdout.strip())
        except Exception as e:
            crashes += 1

        is_false_id = False
        is_miss = False
        if json_obj:
            act_ident = json_obj.get("identification")
            act_fmt = json_obj.get("format_id")
            if exp_fmt is None:
                if act_ident == "format":
                    is_false_id = True
                    false_ids += 1
                else:
                    correct_degradations += 1
            else:
                if act_ident == "format" and act_fmt == exp_fmt:
                    pass
                else:
                    is_miss = True
                    misses += 1

        records.append({
            "name": name,
            "suite": suite,
            "path": cont_path,
            "expected_format": exp_fmt,
            "expected_exit_code": exp_code,
            "actual_exit_code": res.returncode,
            "is_false_id": is_false_id,
            "is_miss": is_miss,
            "exit_code_match": res.returncode == exp_code,
            "stdout": res.stdout.strip()[:200]
        })

    summary = {
        "total": len(all_items),
        "false_identifications": false_ids,
        "missed_identifications": misses,
        "crashes": crashes,
        "correct_degradations": correct_degradations,
        "records": records,
    }

    out_file = os.path.join(PROJECT_ROOT, ".sandbox-test", "results", "52_fixtures_p1a_results.json")
    os.makedirs(os.path.dirname(out_file), exist_ok=True)
    with open(out_file, "w") as f:
        json.dump(summary, f, indent=2)

    print("------------------------------------------------------------------")
    print(f"52-Fixture Matrix Result: false_ids={false_ids}, misses={misses}, crashes={crashes}, correct_degradations={correct_degradations}")
    print("------------------------------------------------------------------")

    if false_ids == 0 and misses == 0 and crashes == 0:
        print("[SUCCESS] All 52 fixtures passed without regression!")
    else:
        print("[FAIL] Regression detected in 52-fixture suite!")
        sys.exit(1)

if __name__ == "__main__":
    main()
