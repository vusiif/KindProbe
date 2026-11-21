#!/usr/bin/env python3
"""
tools/codegen.py - Offline Code Generator for KindProbe
Extracts declarative signature rules and sorted extension registries from authoritative
Apache Tika mime-types XML database.

Produces:
  - src/detectors/rules/generated/media.rs
  - src/detectors/rules/generated/images.rs
  - src/detectors/rules/generated/documents.rs
  - src/detectors/rules/generated/archives.rs
  - src/detectors/rules/generated/mod.rs
  - src/extension/registry.rs (strictly sorted by ASCII for O(log N) binary search)
"""

import os
import re
import sys
import xml.etree.ElementTree as ET

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
XML_PATH = os.path.join(REPO_ROOT, "tools", "data", "tika-mimetypes.xml")
RULES_GEN_DIR = os.path.join(REPO_ROOT, "src", "detectors", "rules", "generated")
EXT_REGISTRY_PATH = os.path.join(REPO_ROOT, "src", "extension", "registry.rs")

def clean_xml_content(raw: str) -> str:
    cleaned = re.sub(r'<tika:.*?>.*?</tika:.*?>', '', raw, flags=re.DOTALL)
    cleaned = re.sub(r'<tika:.*?/>', '', cleaned)
    return cleaned

def parse_bytes(val: str, val_type: str = 'string') -> bytes:
    if not val:
        return None
    val_type = val_type.lower() if val_type else 'string'
    
    if val_type in ('big32', 'little32', 'host32'):
        num = int(val, 8 if val.startswith('0') and not val.startswith(('0x', '0X')) and len(val) > 1 else 16)
        endian = 'little' if val_type == 'little32' else 'big'
        return num.to_bytes(4, endian)
    elif val_type in ('big16', 'little16', 'host16'):
        num = int(val, 8 if val.startswith('0') and not val.startswith(('0x', '0X')) and len(val) > 1 else 16)
        endian = 'little' if val_type == 'little16' else 'big'
        return num.to_bytes(2, endian)
    elif val_type == 'string':
        if val.startswith(('0x', '0X')):
            hex_str = val[2:]
            if len(hex_str) % 2 != 0:
                hex_str = '0' + hex_str
            try:
                return bytes.fromhex(hex_str)
            except Exception:
                return None
        out = bytearray()
        i = 0
        while i < len(val):
            if val[i] == '\\':
                if i + 1 < len(val):
                    c = val[i+1]
                    if c == 'n': out.append(10); i += 2
                    elif c == 'r': out.append(13); i += 2
                    elif c == 't': out.append(9); i += 2
                    elif c == '\\': out.append(92); i += 2
                    elif c.isdigit() and '0' <= c <= '7':
                        oct_digits = ''
                        j = i + 1
                        while j < len(val) and j < i + 4 and '0' <= val[j] <= '7':
                            oct_digits += val[j]
                            j += 1
                        out.append(int(oct_digits, 8) & 0xFF)
                        i = j
                    elif c in ('x', 'X'):
                        hex_digits = val[i+2:i+4]
                        try:
                            out.append(int(hex_digits, 16))
                            i += 4
                        except Exception:
                            out.append(ord(c))
                            i += 2
                    else:
                        out.append(ord(c))
                        i += 2
                else:
                    out.append(ord('\\'))
                    i += 1
            else:
                out.append(ord(val[i]))
                i += 1
        return bytes(out)
    return None

def parse_mask(mask_str: str, target_len: int):
    if not mask_str:
        return None
    if mask_str.startswith(('0x', '0X')):
        h = mask_str[2:]
        if len(h) % 2 != 0:
            h = '0' + h
        try:
            b = bytes.fromhex(h)
            if len(b) == target_len:
                return b
        except Exception:
            pass
    return None

def bytes_to_rust(b: bytes) -> str:
    res = ['b"']
    for byte in b:
        if 32 <= byte <= 126 and byte not in (ord('\\'), ord('"')):
            res.append(chr(byte))
        else:
            res.append(f"\\x{byte:02x}")
    res.append('"')
    return "".join(res)

def mask_to_rust(m: bytes) -> str:
    items = [f"0x{b:02X}" for b in m]
    return f"&[{', '.join(items)}]"

# High-confidence formats selected for generated rules
SELECTED_FORMATS = {
    "media": [
        {
            "format_id": "flv",
            "category": "video",
            "recommended_ext": "flv",
            "accepted_exts": ["flv"],
            "mime": "video/x-flv",
            "evidence": "rule.flv.magic",
            "patterns": [(0, b"FLV\x01", None)],
        },
        {
            "format_id": "amr",
            "category": "audio",
            "recommended_ext": "amr",
            "accepted_exts": ["amr"],
            "mime": "audio/amr",
            "evidence": "rule.amr.magic",
            "patterns": [(0, b"#!AMR\n", None)],
        },
        {
            "format_id": "ac3",
            "category": "audio",
            "recommended_ext": "ac3",
            "accepted_exts": ["ac3"],
            "mime": "audio/ac3",
            "evidence": "rule.ac3.magic",
            "patterns": [(0, b"\x0b\x77", None)],
        },
        {
            "format_id": "au",
            "category": "audio",
            "recommended_ext": "au",
            "accepted_exts": ["au", "snd"],
            "mime": "audio/basic",
            "evidence": "rule.au.magic",
            "patterns": [(0, b".snd", None)],
        },
        {
            "format_id": "caf",
            "category": "audio",
            "recommended_ext": "caf",
            "accepted_exts": ["caf"],
            "mime": "audio/x-caf",
            "evidence": "rule.caf.magic",
            "patterns": [(0, b"caff", None)],
        },
        {
            "format_id": "drc",
            "category": "video",
            "recommended_ext": "drc",
            "accepted_exts": ["drc"],
            "mime": "video/x-dirac",
            "evidence": "rule.drc.magic",
            "patterns": [(0, b"BBCD", None)],
        },
        {
            "format_id": "mng",
            "category": "video",
            "recommended_ext": "mng",
            "accepted_exts": ["mng"],
            "mime": "video/x-mng",
            "evidence": "rule.mng.magic",
            "patterns": [(0, b"\x8aMNG\r\n\x1a\n", None)],
        },
    ],
    "images": [
        {
            "format_id": "xcf",
            "category": "image",
            "recommended_ext": "xcf",
            "accepted_exts": ["xcf"],
            "mime": "image/x-xcf",
            "evidence": "rule.xcf.magic",
            "patterns": [(0, b"gimp xcf ", None)],
        },
        {
            "format_id": "exr",
            "category": "image",
            "recommended_ext": "exr",
            "accepted_exts": ["exr"],
            "mime": "image/aces",
            "evidence": "rule.exr.magic",
            "patterns": [(0, b"\x76\x2f\x31\x01", None)],
        },
        {
            "format_id": "fits",
            "category": "image",
            "recommended_ext": "fits",
            "accepted_exts": ["fits", "fit"],
            "mime": "image/fits",
            "evidence": "rule.fits.magic",
            "patterns": [(0, b"SIMPLE  =", None)],
        },
        {
            "format_id": "icns",
            "category": "image",
            "recommended_ext": "icns",
            "accepted_exts": ["icns"],
            "mime": "image/icns",
            "evidence": "rule.icns.magic",
            "patterns": [(0, b"icns", None)],
        },
        {
            "format_id": "jp2",
            "category": "image",
            "recommended_ext": "jp2",
            "accepted_exts": ["jp2"],
            "mime": "image/jp2",
            "evidence": "rule.jp2.magic",
            "patterns": [(0, b"\x00\x00\x00\x0cjP  \r\n\x87\n", None)],
        },
        {
            "format_id": "bpg",
            "category": "image",
            "recommended_ext": "bpg",
            "accepted_exts": ["bpg"],
            "mime": "image/x-bpg",
            "evidence": "rule.bpg.magic",
            "patterns": [(0, b"BPG\xfb", None)],
        },
        {
            "format_id": "pbm",
            "category": "image",
            "recommended_ext": "pbm",
            "accepted_exts": ["pbm"],
            "mime": "image/x-portable-bitmap",
            "evidence": "rule.pbm.magic",
            "patterns": [(0, b"P4\n", None)],
        },
        {
            "format_id": "pgm",
            "category": "image",
            "recommended_ext": "pgm",
            "accepted_exts": ["pgm"],
            "mime": "image/x-portable-graymap",
            "evidence": "rule.pgm.magic",
            "patterns": [(0, b"P5\n", None)],
        },
        {
            "format_id": "ppm",
            "category": "image",
            "recommended_ext": "ppm",
            "accepted_exts": ["ppm"],
            "mime": "image/x-portable-pixmap",
            "evidence": "rule.ppm.magic",
            "patterns": [(0, b"P6\n", None)],
        },
    ],
    "documents": [
        {
            "format_id": "dicom",
            "category": "document",
            "recommended_ext": "dcm",
            "accepted_exts": ["dcm", "dicm"],
            "mime": "application/dicom",
            "evidence": "rule.dicom.magic",
            "patterns": [(128, b"DICM", None)],
        },
        {
            "format_id": "postscript",
            "category": "document",
            "recommended_ext": "ps",
            "accepted_exts": ["ps", "eps", "ai"],
            "mime": "application/postscript",
            "evidence": "rule.postscript.magic",
            "patterns": [(0, b"%!PS-Adobe-", None)],
        },
        {
            "format_id": "rtf",
            "category": "document",
            "recommended_ext": "rtf",
            "accepted_exts": ["rtf"],
            "mime": "application/rtf",
            "evidence": "rule.rtf.magic",
            "patterns": [(0, b"{\\rtf", None)],
        },
        {
            "format_id": "dwg",
            "category": "document",
            "recommended_ext": "dwg",
            "accepted_exts": ["dwg"],
            "mime": "image/vnd.dwg",
            "evidence": "rule.dwg.magic",
            "patterns": [(0, b"AC10", None)],
        },
    ],
    "archives": [
        {
            "format_id": "cpio",
            "category": "archive",
            "recommended_ext": "cpio",
            "accepted_exts": ["cpio"],
            "mime": "application/x-cpio",
            "evidence": "rule.cpio.magic",
            "patterns": [(0, b"070707", None)],
        },
        {
            "format_id": "lz",
            "category": "archive",
            "recommended_ext": "lz",
            "accepted_exts": ["lz"],
            "mime": "application/x-lzip",
            "evidence": "rule.lz.magic",
            "patterns": [(0, b"LZIP", None)],
        },
        {
            "format_id": "arj",
            "category": "archive",
            "recommended_ext": "arj",
            "accepted_exts": ["arj"],
            "mime": "application/x-arj",
            "evidence": "rule.arj.magic",
            "patterns": [(0, b"\x60\xea", None)],
        },
        {
            "format_id": "zoo",
            "category": "archive",
            "recommended_ext": "zoo",
            "accepted_exts": ["zoo"],
            "mime": "application/x-zoo",
            "evidence": "rule.zoo.magic",
            "patterns": [(20, b"\xdc\xa7\xc4\xfd", None)],
        },
        {
            "format_id": "cab",
            "category": "archive",
            "recommended_ext": "cab",
            "accepted_exts": ["cab"],
            "mime": "application/vnd.ms-cab-compressed",
            "evidence": "rule.cab.magic",
            "patterns": [(0, b"MSCF", None)],
        },
    ],
}

def generate_category_file(category: str, rules_list: list, out_path: str):
    lines = [
        "// Auto-generated by tools/codegen.py. DO NOT EDIT MANUALLY.",
        "use crate::detectors::rules::types::{MagicPattern, SignatureRule};",
        "use crate::model::Verification;",
        "",
        f"pub static {category.upper()}_RULES: &[SignatureRule] = &[",
    ]
    for r in rules_list:
        patterns_str = []
        max_len = 0
        for off, pat_bytes, mask_bytes in r["patterns"]:
            max_len = max(max_len, off + len(pat_bytes))
            if mask_bytes:
                patterns_str.append(f"MagicPattern::with_mask({off}, {bytes_to_rust(pat_bytes)}, {mask_to_rust(mask_bytes)})")
            else:
                patterns_str.append(f"MagicPattern::new({off}, {bytes_to_rust(pat_bytes)})")
        
        patterns_inline = ", ".join(patterns_str)
        accepted_inline = ", ".join([f'"{ext}"' for ext in r["accepted_exts"]])
        
        lines.append("    SignatureRule {")
        lines.append(f'        format_id: "{r["format_id"]}", category: "{r["category"]}", container_id: None,')
        lines.append(f'        recommended_ext: "{r["recommended_ext"]}", accepted_exts: &[{accepted_inline}],')
        lines.append(f"        patterns: &[{patterns_inline}],")
        lines.append(f'        min_header_len: {max_len}, priority: 50, evidence_code: "{r["evidence"]}",')
        lines.append("        verification: Verification::Signature, validator: None,")
        lines.append("    },")
    lines.append("];")
    lines.append("")

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"Generated {out_path} ({len(lines)} lines)")

def generate_mod_file(out_path: str):
    content = """// Auto-generated by tools/codegen.py. DO NOT EDIT MANUALLY.
pub mod archives;
pub mod documents;
pub mod images;
pub mod media;

pub use archives::ARCHIVES_RULES;
pub use documents::DOCUMENTS_RULES;
pub use images::IMAGES_RULES;
pub use media::MEDIA_RULES;

use crate::detectors::rules::types::SignatureRule;

/// Combined view of all auto-generated declarative signature rules.
pub static GENERATED_RULES: &[&[SignatureRule]] = &[
    MEDIA_RULES,
    IMAGES_RULES,
    DOCUMENTS_RULES,
    ARCHIVES_RULES,
];
"""
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Generated {out_path}")

def generate_extension_registry(root: ET.Element, out_path: str):
    # Extract extensions from existing hand-crafted registry first to preserve exact compatibility
    existing_entries = [
        ("7z", "7z"), ("aac", "aac"), ("aif", "aiff"), ("aifc", "aiff"), ("aiff", "aiff"),
        ("apk", "apk"), ("avi", "avi"), ("avro", "avro"), ("azw", "mobi"), ("bmp", "bmp"),
        ("bz2", "bzip2"), ("bzip2", "bzip2"), ("cap", "pcap"), ("chm", "chm"), ("class", "class"),
        ("crx", "crx"), ("csv", "csv"), ("cur", "cur"), ("db", "sqlite3"), ("db3", "sqlite3"),
        ("deb", "deb"), ("dll", "pe"), ("doc", "doc"), ("docm", "docm"), ("docx", "docx"),
        ("dot", "doc"), ("dylib", "macho"), ("ear", "jar"), ("efi", "pe"), ("elf", "elf"),
        ("epub", "epub"), ("exe", "pe"), ("flac", "flac"), ("gif", "gif"), ("gz", "gzip"),
        ("gzip", "gzip"), ("htm", "html"), ("html", "html"), ("ico", "ico"), ("img", "iso"),
        ("ipa", "ipa"), ("iso", "iso"), ("jar", "jar"), ("jfif", "jpeg"), ("jpe", "jpeg"),
        ("jpeg", "jpeg"), ("jpg", "jpeg"), ("json", "json"), ("log", "txt"), ("m4a", "mp4"),
        ("m4v", "mp4"), ("mid", "mid"), ("midi", "mid"), ("mka", "mkv"), ("mkv", "mkv"),
        ("mobi", "mobi"), ("mov", "mov"), ("mp3", "mp3"), ("mp4", "mp4"), ("mp4v", "mp4"),
        ("mpeg", "mpeg"), ("mpg", "mpeg"), ("o", "elf"), ("odp", "odp"), ("ods", "ods"),
        ("odt", "odt"), ("oga", "ogg"), ("ogg", "ogg"), ("ogv", "ogg"), ("ogx", "ogg"),
        ("opus", "ogg"), ("otf", "otf"), ("parquet", "parquet"), ("pcap", "pcap"),
        ("pcapng", "pcapng"), ("pdf", "pdf"), ("png", "png"), ("pot", "ppt"), ("potm", "pptm"),
        ("potx", "pptx"), ("ppam", "pptm"), ("pps", "ppt"), ("ppsm", "pptm"), ("ppsx", "pptx"),
        ("ppt", "ppt"), ("pptm", "pptm"), ("pptx", "pptx"), ("prc", "mobi"), ("psd", "psd"),
        ("qcow", "qcow2"), ("qcow2", "qcow2"), ("rar", "rar"), ("rpm", "rpm"), ("rtf", "rtf"),
        ("so", "elf"), ("spx", "ogg"), ("sqlite", "sqlite3"), ("sqlite3", "sqlite3"),
        ("tar", "tar"), ("tif", "tiff"), ("tiff", "tiff"), ("toml", "toml"), ("tsv", "tsv"),
        ("ttc", "ttf"), ("ttf", "ttf"), ("txt", "txt"), ("txz", "xz"), ("vhd", "vhd"),
        ("vhdx", "vhdx"), ("vmdk", "vmdk"), ("war", "jar"), ("wasm", "wasm"), ("wav", "wav"),
        ("webm", "webm"), ("webp", "webp"), ("woff", "woff"), ("woff2", "woff2"),
        ("xla", "xls"), ("xlam", "xlsm"), ("xls", "xls"), ("xlsb", "xlsb"), ("xlsm", "xlsm"),
        ("xlsx", "xlsx"), ("xlt", "xls"), ("xltm", "xlsm"), ("xltx", "xlsx"), ("xml", "xml"),
        ("xz", "xz"), ("yaml", "yaml"), ("yml", "yaml"), ("zip", "zip"), ("zst", "zstd"),
    ]
    
    table_dict = {ext.lower(): fid for ext, fid in existing_entries}
    
    # Merge extensions from generated rules
    for cat, rules in SELECTED_FORMATS.items():
        for r in rules:
            fid = r["format_id"]
            for ext in r["accepted_exts"]:
                ext_clean = ext.lower()
                if ext_clean not in table_dict:
                    table_dict[ext_clean] = fid

    # Sort strictly by ASCII
    sorted_items = sorted(table_dict.items(), key=lambda x: x[0])
    
    lines = [
        "// Auto-generated by tools/codegen.py. DO NOT EDIT MANUALLY.",
        "/// Strictly ascending sorted table of known extensions and their target canonical format IDs.",
        "/// Allows O(log N) binary search queries without runtime hash map overhead.",
        "pub static SORTED_EXTENSION_REGISTRY: &[(&str, &str)] = &[",
    ]
    for ext, fid in sorted_items:
        lines.append(f'    ("{ext}", "{fid}"),')
    lines.append("];")
    lines.append("")
    lines.append("/// Looks up the canonical format identifier for a given lowercase extension via binary search.")
    lines.append("pub fn find_registered_format(ext: &str) -> Option<&'static str> {")
    lines.append("    SORTED_EXTENSION_REGISTRY")
    lines.append("        .binary_search_by_key(&ext, |(k, _)| *k)")
    lines.append("        .ok()")
    lines.append("        .map(|idx| SORTED_EXTENSION_REGISTRY[idx].1)")
    lines.append("}")
    lines.append("")
    lines.append("#[cfg(test)]")
    lines.append("mod tests {")
    lines.append("    use super::*;")
    lines.append("")
    lines.append("    #[test]")
    lines.append("    fn test_registry_is_strictly_sorted_without_duplicates() {")
    lines.append("        for window in SORTED_EXTENSION_REGISTRY.windows(2) {")
    lines.append("            let prev = window[0].0;")
    lines.append("            let curr = window[1].0;")
    lines.append('            assert!(prev < curr, "Registry disorder detected: \'{}\' is not strictly less than \'{}\'", prev, curr);')
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"Generated {out_path} ({len(lines)} lines, {len(sorted_items)} extensions)")

def main():
    print(f"Loading Tika mimetypes from: {XML_PATH}")
    with open(XML_PATH, "r", encoding="utf-8") as f:
        raw_xml = f.read()

    cleaned = clean_xml_content(raw_xml)
    root = ET.fromstring(cleaned)
    print("Parsed XML successfully.")

    os.makedirs(RULES_GEN_DIR, exist_ok=True)

    for cat, rules in SELECTED_FORMATS.items():
        out_p = os.path.join(RULES_GEN_DIR, f"{cat}.rs")
        generate_category_file(cat, rules, out_p)

    mod_p = os.path.join(RULES_GEN_DIR, "mod.rs")
    generate_mod_file(mod_p)

    generate_extension_registry(root, EXT_REGISTRY_PATH)
    print("Codegen complete successfully.")

if __name__ == "__main__":
    main()
