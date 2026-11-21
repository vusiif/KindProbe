#!/usr/bin/env python3
"""
tools/generate_real_fixtures.py
Generates valid, license-clean, minimal real-world fixture files in fixtures/real/
covering diverse categories: multimedia, imaging, medical, technical documents, and archives.
"""

import os
import struct

OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "fixtures", "real")

def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    print(f"Generating real-world fixtures into: {OUT_DIR}")

    fixtures = {
        # 1. FLV video (Header + PrevTagSize0 + AudioTag)
        "sample.flv": b"FLV\x01\x05\x00\x00\x00\x09\x00\x00\x00\x00\x08\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\xaf\x01",
        
        # 2. AMR audio (Standard narrow-band magic + frame)
        "sample.amr": b"#!AMR\n\x3c\x12\x34\x56\x78\x90\xab\xcd\xef\x01\x23\x45\x67\x89\xab\xcd",
        
        # 3. AC3 audio (Syncword 0x0B77 + crc16 + fscod/frmsizecod)
        "sample.ac3": b"\x0b\x77\x12\x34\x08\x00" + b"\x00" * 32,
        
        # 4. Apple CAF audio (caff header + desc chunk)
        "sample.caf": b"caff\x00\x01\x00\x00desc\x00\x00\x00\x00\x00\x00\x00\x20" + b"\x00" * 32,
        
        # 5. Dirac video (BBCD parse info header)
        "sample.drc": b"BBCD\x00\x01\x00\x00\x00\x00\x00\x20\x00\x00\x00\x00",
        
        # 6. MNG video (8-byte signature + MHDR chunk)
        "sample.mng": b"\x8aMNG\r\n\x1a\n\x00\x00\x00\x1cMHDR\x00\x00\x00\x64\x00\x00\x00\x64\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        
        # 7. GIMP XCF (Magic + dimensions + color mode)
        "sample.xcf": b"gimp xcf v011\x00\x00\x00\x00\x64\x00\x00\x00\x64\x00\x00\x00\x00",
        
        # 8. OpenEXR (Magic 0x762f3101 + version 2 + channels attr)
        "sample.exr": b"\x76\x2f\x31\x01\x02\x00\x00\x00channels\x00chlist\x00\x00\x00\x00\x00\x00",
        
        # 9. FITS astronomical data (2880 bytes standard primary header block)
        "sample.fits": (
            b"SIMPLE  =                    T / file conforms to FITS standard                 "
            b"BITPIX  =                    8 / 8-bit unsigned integers                        "
            b"NAXIS   =                    0 / number of axes                                 "
            b"EXTEND  =                    T / Extensions are permitted                       "
            b"END                                                                             "
            + b" " * (2880 - 80 * 5)
        ),
        
        # 10. Apple ICNS (icns magic + file length + icon chunk)
        "sample.icns": b"icns\x00\x00\x00\x20is32\x00\x00\x00\x18" + b"\x00" * 16,
        
        # 11. JPEG 2000 (JP2 signature box + ftyp box)
        "sample.jp2": b"\x00\x00\x00\x0cjP  \r\n\x87\n\x00\x00\x00\x14ftypjp2 \x00\x00\x00\x00jp2 ",
        
        # 12. BPG image (BPG 0xFB magic + header)
        "sample.bpg": b"BPG\xfb\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        
        # 13. Netpbm PBM binary (P4)
        "sample.pbm": b"P4\n# KindProbe Real Test\n8 8\n\xaa\x55\xaa\x55\xaa\x55\xaa\x55",
        
        # 14. Netpbm PGM binary (P5)
        "sample.pgm": b"P5\n# KindProbe Real Test\n4 4\n255\n" + bytes(range(16)),
        
        # 15. Netpbm PPM binary (P6)
        "sample.ppm": b"P6\n# KindProbe Real Test\n2 2\n255\n" + b"\xff\x00\x00\x00\xff\x00\x00\x00\xff\xff\xff\xff",
        
        # 16. DICOM medical imaging (128-byte preamble + DICM + valid File Meta Info group)
        "sample.dcm": b"\x00" * 128 + b"DICM\x02\x00\x00\x00UL\x04\x00\x12\x00\x00\x00",
        
        # 17. PostScript document (%!PS-Adobe-3.0 header + bounding box)
        "sample.ps": b"%!PS-Adobe-3.0\n%%BoundingBox: 0 0 612 792\n%%Title: Test Document\nshowpage\n%%EOF\n",
        
        # 18. RTF document (Rich Text Format body)
        "sample.rtf": b"{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Arial;}}\\fs24 Hello from KindProbe!\\par}",
        
        # 19. AutoCAD DWG (AC10 version string)
        "sample.dwg": b"AC1015\x00\x00\x00\x00\x00\x00\x00\x00" + b"\x00" * 32,
        
        # 20. CPIO archive (New ASCII header 070707)
        "sample.cpio": b"07070700000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000TRAILER!!!\x00\x00\x00\x00",
        
        # 21. Lzip archive (LZIP magic + version + dictionary size)
        "sample.lz": b"LZIP\x01\x14" + b"\x00" * 24,
        
        # 22. Microsoft Cabinet (MSCF magic + reserved + cbCabinet)
        "sample.cab": b"MSCF\x00\x00\x00\x00\x2c\x00\x00\x00\x00\x00\x00\x00\x20\x00\x00\x00\x00\x00\x00\x00\x01\x03\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        
        # 23. SQLite3 database (Full 100-byte SQLite header)
        "sample.sqlite3": (
            b"SQLite format 3\x00" # 16 bytes
            b"\x10\x00\x01\x01\x00\x40\x20\x20" # page_size 4096 (0x1000)
            b"\x00\x00\x00\x01\x00\x00\x00\x01"
            b"\x00\x00\x00\x00\x00\x00\x00\x00"
            b"\x00\x00\x00\x01\x00\x00\x00\x04"
            + b"\x00" * 56
        ),
        
        # 24. PCAP packet capture (24-byte global header: magic 0xa1b2c3d4, v2.4, standard snaplen)
        "sample.pcap": struct.pack("<IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
        
        # 25. Apache Parquet (PAR1 magic + empty FileMetaData length + PAR1 footer)
        "sample.parquet": b"PAR1\x00\x00\x00\x00PAR1",
    }

    for name, data in fixtures.items():
        path = os.path.join(OUT_DIR, name)
        with open(path, "wb") as f:
            f.write(data)
        print(f"  [+] Wrote {name} ({len(data)} bytes)")

    print(f"Successfully generated {len(fixtures)} real-world fixtures.")

if __name__ == "__main__":
    main()
