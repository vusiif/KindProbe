#!/usr/bin/env python3
"""
Generate reproducible test fixtures for KindProbe benchmark and test suites.
All fixtures are synthesized deterministically without external dependencies or proprietary data.
"""

import os
import struct
import zipfile
import hashlib

FIXTURES_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "fixtures"))

def make_cfb(stream_name: str, stream_data: bytes, cyclic: bool = False) -> bytes:
    # CFB v3 (512-byte sectors)
    hdr = bytearray(512)
    hdr[0:8] = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
    struct.pack_into("<HHH", hdr, 24, 0x003e, 0x0003, 0xfffe)
    struct.pack_into("<HH", hdr, 30, 9, 6) # sector shift 9 (512), mini shift 6 (64)
    struct.pack_into("<I", hdr, 44, 1) # num fat sectors = 1
    struct.pack_into("<I", hdr, 48, 1) # first dir sector = 1
    struct.pack_into("<I", hdr, 56, 4096) # mini cutoff = 4096
    struct.pack_into("<I", hdr, 60, 0xfffffffe) # first mini fat = ENDOFCHAIN
    struct.pack_into("<I", hdr, 68, 0xfffffffe) # first difat = ENDOFCHAIN
    # DIFAT table (109 entries)
    struct.pack_into("<I", hdr, 76, 0) # sector 0 is FAT
    for i in range(1, 109):
        struct.pack_into("<I", hdr, 76 + i*4, 0xffffffff)
    
    # Sector 0: FAT (128 entries of 4 bytes)
    fat = bytearray(512)
    for i in range(128):
        struct.pack_into("<I", fat, i*4, 0xffffffff)
    struct.pack_into("<I", fat, 0, 0xfffffffd) # Sector 0: FAT sector
    struct.pack_into("<I", fat, 4, 0xfffffffe) # Sector 1: Dir sector (ENDOFCHAIN)
    
    # Stream will occupy sectors 2..9 (8 sectors = 4096 bytes)
    if cyclic:
        # Loop in stream chain: 2 -> 3 -> 2
        struct.pack_into("<I", fat, 2*4, 3)
        struct.pack_into("<I", fat, 3*4, 2)
    else:
        for s in range(2, 9):
            struct.pack_into("<I", fat, s*4, s + 1)
        struct.pack_into("<I", fat, 9*4, 0xfffffffe) # Sector 9: ENDOFCHAIN
    
    # Sector 1: Directory (4 entries of 128 bytes)
    dir_sec = bytearray(512)
    
    # Entry 0: Root Entry
    root_name = "Root Entry\0".encode("utf-16le")
    dir_sec[0:len(root_name)] = root_name
    struct.pack_into("<H", dir_sec, 64, len(root_name))
    dir_sec[66] = 5 # root storage
    dir_sec[67] = 1 # black
    struct.pack_into("<III", dir_sec, 68, 0xffffffff, 0xffffffff, 1) # left, right, child=1
    struct.pack_into("<I", dir_sec, 116, 0xfffffffe)
    struct.pack_into("<Q", dir_sec, 120, 0)
    
    # Entry 1: Stream Entry
    s_name = (stream_name + "\0").encode("utf-16le")
    dir_sec[128:128+len(s_name)] = s_name
    struct.pack_into("<H", dir_sec, 128+64, len(s_name))
    dir_sec[128+66] = 2 # user stream
    dir_sec[128+67] = 1 # black
    struct.pack_into("<III", dir_sec, 128+68, 0xffffffff, 0xffffffff, 0xffffffff)
    struct.pack_into("<I", dir_sec, 128+116, 2) # start sector = 2
    struct.pack_into("<Q", dir_sec, 128+120, 4096) # size >= 4096 (regular FAT)
    
    # Sectors 2..9: Stream Data padded to 4096 bytes
    s_data = bytearray(stream_data)
    if len(s_data) < 4096:
        s_data.extend(b"\x00" * (4096 - len(s_data)))
    elif len(s_data) > 4096:
        s_data = s_data[:4096]
        
    return bytes(hdr + fat + dir_sec + s_data)

def generate_all():
    os.makedirs(FIXTURES_DIR, exist_ok=True)
    manifest = []
    
    def save(name, data, expected_format, expected_container, description):
        path = os.path.join(FIXTURES_DIR, name)
        with open(path, "wb") as f:
            f.write(data)
        sha = hashlib.sha256(data).hexdigest()
        manifest.append({
            "name": name,
            "path": f"fixtures/{name}",
            "size": len(data),
            "sha256": sha,
            "expected_format": expected_format,
            "expected_container": expected_container,
            "description": description
        })
        return data

    # 1. Valid PNG
    png_sig = b"\x89PNG\r\n\x1a\n"
    # IHDR chunk: length 13, type IHDR, data (width=1, height=1, bitdepth=8, colortype=2, comp=0, filter=0, interlace=0), crc
    ihdr_data = struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0)
    ihdr_crc = 0x5c721c5d # precomputed crc32 for 'IHDR' + ihdr_data
    ihdr_chunk = struct.pack(">I", 13) + b"IHDR" + ihdr_data + struct.pack(">I", ihdr_crc)
    iend_chunk = struct.pack(">I", 0) + b"IEND" + struct.pack(">I", 0xae426082)
    png_bytes = save("valid_png.png", png_sig + ihdr_chunk + iend_chunk, "png", None, "Valid 1x1 PNG image")

    # 2. Valid PDF
    pdf_bytes = save("valid_pdf.pdf", b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n", "pdf", None, "Minimal valid PDF document")

    # 3. Plain ZIP
    import io
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("notes.txt", "Plain text notes inside a standard zip archive.\n")
    zip_bytes = save("plain_zip.zip", bio.getvalue(), "zip", "zip", "Standard plain ZIP archive with text file")

    # 4. Fake DOCX ZIP (contains word/document.xml but no Content_Types or rels)
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("word/document.xml", "<fake>Only word/ directory, not genuine OOXML</fake>")
    save("fake_docx_zip.zip", bio.getvalue(), None, "zip", "ZIP containing word/document.xml but no [Content_Types].xml or _rels (suspected OOXML without required evidence)")

    # 5. Partial OOXML missing rels
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("word/document.xml", '<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document>')
    save("partial_ooxml_missing_rels.zip", bio.getvalue(), None, "zip", "Partial OOXML ZIP: has [Content_Types].xml but missing _rels/.rels")

    # 6. Partial OOXML missing main part
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        # Note: word/document.xml is NOT present!
    save("partial_ooxml_missing_main_part.zip", bio.getvalue(), None, "zip", "Partial OOXML ZIP: relationship targets word/document.xml which does not exist")

    # 7. True DOCX
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        z.writestr("word/document.xml", '<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>KindProbe Test</w:t></w:r></w:p></w:body></w:document>')
    docx_bytes = save("true_docx.docx", bio.getvalue(), "docx", "zip", "Genuine Transitional DOCX with verified OPC parts and namespaces")

    # 8. True DOCX Strict
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        z.writestr("word/document.xml", '<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://purl.oclc.org/ooxml/wordprocessingml/main"><w:body><w:p><w:r><w:t>Strict DOCX</w:t></w:r></w:p></w:body></w:document>')
    save("true_docx_strict.docx", bio.getvalue(), "docx", "zip", "Genuine Strict DOCX with purl.oclc.org namespaces")

    # 9. True DOCX Relative Path (target = 'document.xml')
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="document.xml"/></Relationships>')
        z.writestr("document.xml", '<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p/></w:body></w:document>')
    save("true_docx_rel_path.docx", bio.getvalue(), "docx", "zip", "Genuine DOCX with relative target path document.xml in root")

    # 10. Plain CFB (not Word)
    plain_cfb_data = make_cfb("Data", b"Plain compound document stream data")
    save("plain_cfb.cfb", plain_cfb_data, None, "cfb", "Valid CFB container with non-Word Data stream")

    # 11. Fake DOC in CFB (WordDocument stream name with dummy bytes)
    fake_doc_data = make_cfb("WordDocument", b"Fake stream data: NOT a FIB header at all")
    save("fake_doc_cfb.doc", fake_doc_data, None, "cfb", "CFB containing WordDocument stream name but invalid FIB magic/version")

    # 12. True DOC in CFB (Word 97+ FIB)
    fib = bytearray(512)
    struct.pack_into("<HH", fib, 0, 0xA5EC, 0x00C1) # wIdent = 0xA5EC, nFib = 0x00C1 (Word 97)
    struct.pack_into("<H", fib, 10, 0x0000) # flags (not encrypted)
    struct.pack_into("<II", fib, 0x0020, 0x00000400, 0x00000800) # fcMin, fcMac
    true_doc_data = make_cfb("WordDocument", fib)
    save("true_doc.doc", true_doc_data, "doc", "cfb", "Genuine Word 97+ binary DOC with verified FIB wIdent 0xA5EC and nFib 0x00C1")

    # 13. Mismatched extensions
    save("mismatched_png.pdf", png_bytes, "png", None, "Valid PNG renamed with .pdf extension")
    save("mismatched_docx.zip", docx_bytes, "docx", "zip", "Valid DOCX renamed with .zip extension")
    save("mismatched_doc.docx", true_doc_data, "doc", "cfb", "Valid DOC renamed with .docx extension")

    # 14. Edge & Truncated
    save("empty_file.bin", b"", None, None, "0-byte empty file")
    save("truncated_png.png", b"\x89PNG", None, None, "Truncated 4-byte PNG prefix")
    save("truncated_pdf.pdf", b"%PD", None, None, "Truncated 3-byte PDF prefix")
    save("truncated_zip.zip", b"PK\x03\x04\x14\x00\x00\x00", None, None, "Truncated 8-byte ZIP local header")
    save("truncated_cfb.doc", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1\x00\x00\x00\x00", None, None, "Truncated 12-byte CFB header")
    save("random_garbage.bin", os.urandom(256), None, None, "256 bytes of pseudo-random binary data")

    # 15. Adversarial Budget fixtures
    # Deep XML: 80 levels of nested tags
    deep_xml = '<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
    for i in range(80):
        deep_xml += f'<w:level{i}>'
    deep_xml += 'Deep'
    for i in reversed(range(80)):
        deep_xml += f'</w:level{i}>'
    deep_xml += '</w:document>'
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr("_rels/.rels", '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        z.writestr("word/document.xml", deep_xml)
    save("budget_xml_deep.docx", bio.getvalue(), "docx", "zip", "Adversarial DOCX with 80 nested XML levels (exceeds depth budget 64)")

    # Cyclic CFB
    cyclic_cfb = make_cfb("WordDocument", fib, cyclic=True)
    save("cfb_cyclic_chain.doc", cyclic_cfb, None, "cfb", "CFB with FAT sector cycle (sector 2 points to sector 2)")

    import json
    manifest_path = os.path.join(os.path.dirname(__file__), "..", "research", "benchmark-manifest.json")
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"Generated {len(manifest)} fixtures in {FIXTURES_DIR}")
    print(f"Benchmark manifest saved to {manifest_path}")

if __name__ == "__main__":
    generate_all()
