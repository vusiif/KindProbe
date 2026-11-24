#![forbid(unsafe_code)]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use kindprobe::budget::DetectOptions;
use kindprobe::{detect_bytes, detect_bytes_core};
use serde_json::Value;

#[allow(dead_code)]
#[derive(Clone)]
struct BenchmarkSample {
    id: &'static str,
    filename: &'static str,
    category: &'static str,
    data: Vec<u8>,
}

fn get_all_samples() -> Vec<BenchmarkSample> {
    vec![
        // ---------------------------------------------------------------------
        // 1. Image Formats
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "png_adam7",
            filename: "sample_interlaced.png",
            category: "image",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"\x89PNG\r\n\x1a\n");
                // IHDR chunk: 13 bytes data, width=32, height=32, bit_depth=8, color_type=6 (RGBA), interlaced=1
                let ihdr_len = 13u32.to_be_bytes();
                v.extend_from_slice(&ihdr_len);
                v.extend_from_slice(b"IHDR");
                v.extend_from_slice(&32u32.to_be_bytes());
                v.extend_from_slice(&32u32.to_be_bytes());
                v.extend_from_slice(&[8, 6, 0, 0, 1]); // interlaced
                v.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // CRC dummy
                // IDAT chunk dummy
                v.extend_from_slice(&4u32.to_be_bytes());
                v.extend_from_slice(b"IDAT");
                v.extend_from_slice(&[0x78, 0x9c, 0x01, 0x00]);
                v.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
                // IEND chunk
                v.extend_from_slice(&0u32.to_be_bytes());
                v.extend_from_slice(b"IEND");
                v.extend_from_slice(&[0xae, 0x42, 0x60, 0x82]);
                v
            },
        },
        BenchmarkSample {
            id: "jpeg_progressive",
            filename: "sample_prog.jpg",
            category: "image",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"\xff\xd8"); // SOI
                // APP0 JFIF
                v.extend_from_slice(b"\xff\xe0\x00\x10JFIF\x00\x01\x02\x01\x00\x48\x00\x48\x00\x00");
                // DQT
                v.extend_from_slice(b"\xff\xdb\x00\x43\x00");
                v.extend_from_slice(&[0x08; 64]);
                // SOF2 (Progressive)
                v.extend_from_slice(b"\xff\xc2\x00\x11\x08\x00\x20\x00\x20\x03\x01\x11\x00\x02\x11\x01\x03\x11\x01");
                // EOI
                v.extend_from_slice(b"\xff\xd9");
                v
            },
        },
        BenchmarkSample {
            id: "gif87a",
            filename: "sample_87a.gif",
            category: "image",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"GIF87a");
                v.extend_from_slice(&16u16.to_le_bytes()); // width
                v.extend_from_slice(&16u16.to_le_bytes()); // height
                v.extend_from_slice(&[0x80, 0x00, 0x00]); // Packed fields, bg, aspect
                // Global color table (2 colors)
                v.extend_from_slice(&[0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF]);
                // Trailer
                v.push(0x3B);
                v
            },
        },
        BenchmarkSample {
            id: "webp_vp8x",
            filename: "sample_anim.webp",
            category: "image",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"RIFF\x28\x00\x00\x00WEBPVP8X\x0a\x00\x00\x00\x12\x00\x00\x00\x1f\x00\x00\x1f\x00\x00");
                v
            },
        },
        BenchmarkSample {
            id: "bmp_v3",
            filename: "sample_dib.bmp",
            category: "image",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"BM");
                v.extend_from_slice(&70u32.to_le_bytes()); // file size
                v.extend_from_slice(&[0, 0, 0, 0]); // reserved
                v.extend_from_slice(&54u32.to_le_bytes()); // offset to pixels
                // BITMAPINFOHEADER (40 bytes)
                v.extend_from_slice(&40u32.to_le_bytes());
                v.extend_from_slice(&2u32.to_le_bytes()); // width
                v.extend_from_slice(&2u32.to_le_bytes()); // height
                v.extend_from_slice(&1u16.to_le_bytes()); // planes
                v.extend_from_slice(&24u16.to_le_bytes()); // bpp
                v.extend_from_slice(&0u32.to_le_bytes()); // compression BI_RGB
                v.extend_from_slice(&16u32.to_le_bytes()); // image size
                v.extend_from_slice(&[0u8; 16]); // ppm & colors
                v.extend_from_slice(&[0xFF; 16]); // pixel data
                v
            },
        },
        BenchmarkSample {
            id: "tiff_le",
            filename: "sample_le.tiff",
            category: "image",
            data: b"II\x2a\x00\x08\x00\x00\x00\x01\x00\x00\x01\x04\x00\x01\x00\x00\x00\x20\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "tiff_be",
            filename: "sample_be.tiff",
            category: "image",
            data: b"MM\x00\x2a\x00\x00\x00\x08\x00\x01\x01\x00\x00\x04\x00\x00\x00\x01\x00\x00\x00\x20\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "ico_icon",
            filename: "sample_app.ico",
            category: "image",
            data: b"\x00\x00\x01\x00\x01\x00\x20\x20\x00\x00\x01\x00\x20\x00\xa8\x08\x00\x00\x16\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "photoshop_psd",
            filename: "sample_design.psd",
            category: "image",
            data: b"8BPS\x00\x01\x00\x00\x00\x00\x00\x00\x00\x03\x00\x00\x00\x40\x00\x00\x00\x40\x00\x08\x00\x03".to_vec(),
        },

        // ---------------------------------------------------------------------
        // 2. Audio & Video
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "wav_pcm_stereo",
            filename: "sample_stereo.wav",
            category: "audio",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"RIFF\x2c\x00\x00\x00WAVE");
                v.extend_from_slice(b"fmt \x10\x00\x00\x00"); // 16 bytes chunk
                v.extend_from_slice(&1u16.to_le_bytes()); // PCM format
                v.extend_from_slice(&2u16.to_le_bytes()); // 2 channels
                v.extend_from_slice(&44100u32.to_le_bytes()); // sample rate
                v.extend_from_slice(&176400u32.to_le_bytes()); // byte rate
                v.extend_from_slice(&4u16.to_le_bytes()); // block align
                v.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
                v.extend_from_slice(b"data\x08\x00\x00\x00");
                v.extend_from_slice(&[0; 8]);
                v
            },
        },
        BenchmarkSample {
            id: "avi_divx",
            filename: "sample_clip.avi",
            category: "video",
            data: b"RIFF\x38\x00\x00\x00AVI LIST\x20\x00\x00\x00hdrlavih\x38\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "mp3_id3v23",
            filename: "sample_track.mp3",
            category: "audio",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"ID3\x03\x00\x00\x00\x00\x00\x30");
                v.extend_from_slice(b"TIT2\x00\x00\x00\x0e\x00\x00\x00KindProbe Track");
                // MPEG Layer 3 sync
                v.extend_from_slice(&[0xff, 0xfb, 0x90, 0x64]);
                v.extend_from_slice(&[0x00; 64]);
                v
            },
        },
        BenchmarkSample {
            id: "mp3_raw_frame",
            filename: "sample_stream.mp3",
            category: "audio",
            data: {
                let mut v = vec![0xff, 0xfb, 0xa0, 0x40]; // 11111111 11111011 (Layer III, 160kbps, 44.1kHz)
                v.extend_from_slice(&[0x11; 128]);
                v
            },
        },
        BenchmarkSample {
            id: "mp4_dash",
            filename: "sample_stream.mp4",
            category: "video",
            data: b"\x00\x00\x00\x1cftypdash\x00\x00\x00\x00iso6mp41dash".to_vec(),
        },
        BenchmarkSample {
            id: "mov_prores",
            filename: "sample_prores.mov",
            category: "video",
            data: b"\x00\x00\x00\x14ftypqt  \x20\x05\x03\x00qt  \x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "m4a_audio",
            filename: "sample_podcast.m4a",
            category: "audio",
            data: b"\x00\x00\x00\x20ftypM4A \x00\x00\x00\x00M4A mp42isom\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "matroska_mkv",
            filename: "sample_movie.mkv",
            category: "video",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"\x1a\x45\xdf\xa3"); // EBML Header
                v.extend_from_slice(&[0x01, 0x00, 0x00, 0x1f]); // size
                v.extend_from_slice(&[0x42, 0x82, 0x88]); // DocType (len=8)
                v.extend_from_slice(b"matroska");
                v.extend_from_slice(&[0x42, 0x87, 0x81, 0x04]); // DocTypeVersion=4
                v
            },
        },
        BenchmarkSample {
            id: "matroska_webm",
            filename: "sample_clip.webm",
            category: "video",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"\x1a\x45\xdf\xa3"); // EBML Header
                v.extend_from_slice(&[0x01, 0x00, 0x00, 0x1f]); // size
                v.extend_from_slice(&[0x42, 0x82, 0x84]); // DocType (len=4)
                v.extend_from_slice(b"webm");
                v.extend_from_slice(&[0x42, 0x87, 0x81, 0x02]); // DocTypeVersion=2
                v
            },
        },
        BenchmarkSample {
            id: "flac_audio",
            filename: "sample_lossless.flac",
            category: "audio",
            data: b"fLaC\x00\x00\x00\x22\x10\x00\x10\x00\x00\x00\x00\x00\x00\x00\x0a\xc4\x42\xf0\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "ogg_vorbis",
            filename: "sample_stream.ogg",
            category: "audio",
            data: {
                let mut v = Vec::new();
                v.extend_from_slice(b"OggS\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x01\x1e");
                v.extend_from_slice(b"\x01vorbis\x00\x00\x00\x00\x02\x44\xac\x00\x00\x00\x00\x00\x00");
                v
            },
        },
        BenchmarkSample {
            id: "flv_flash",
            filename: "sample_stream.flv",
            category: "video",
            data: b"FLV\x01\x05\x00\x00\x00\x09\x00\x00\x00\x00".to_vec(),
        },

        // ---------------------------------------------------------------------
        // 3. Archives & Compressed Packages
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "zip_stored_archive",
            filename: "sample_bundle.zip",
            category: "archive",
            data: {
                let mut v = Vec::new();
                // Local File Header
                v.extend_from_slice(b"PK\x03\x04\x0a\x00\x00\x00\x00\x00\x00\x00!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x05\x00\x00\x00hello");
                // Central Directory
                let cd_offset = v.len() as u32;
                v.extend_from_slice(b"PK\x01\x02\x14\x00\x0a\x00\x00\x00\x00\x00\x00\x00!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00hello");
                let cd_size = (v.len() as u32) - cd_offset;
                // EOCD
                v.extend_from_slice(b"PK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x00");
                v.extend_from_slice(&cd_size.to_le_bytes());
                v.extend_from_slice(&cd_offset.to_le_bytes());
                v.extend_from_slice(&[0, 0]); // comment len
                v
            },
        },
        BenchmarkSample {
            id: "gzip_fextra",
            filename: "sample_backup.gz",
            category: "archive",
            data: b"\x1f\x8b\x08\x04\x65\x21\xa4\x66\x02\x03\x06\x00BC\x02\x00\x42\x43\x0b\xc9\xc8\x2c\x56\x00\xa2\x44\x85\x92\xd4\xe2\x12\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "bzip2_block9",
            filename: "sample_archive.bz2",
            category: "archive",
            data: b"BZh91AY&SY\x7f\x8b\x12\x44\x00\x00\x01\x40\x00\x80\x00\x20\x00\x21\x9a\x68\x33\x4d\x04\x04\x80".to_vec(),
        },
        BenchmarkSample {
            id: "xz_stream",
            filename: "sample_data.xz",
            category: "archive",
            data: b"\xfd7zXZ\x00\x00\x01\x69\x22\xde\x36\x02\x00\x21\x01\x16\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "zstd_frame",
            filename: "sample_data.zst",
            category: "archive",
            data: b"\x28\xb5\x2f\xfd\x00\x58\x2a\x00\x00\x01\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "sevenz_archive",
            filename: "sample_files.7z",
            category: "archive",
            data: b"7z\xbc\xaf\x27\x1c\x00\x04\x6c\x83\x12\x90\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "rar5_archive",
            filename: "sample_files.rar",
            category: "archive",
            data: b"Rar!\x1a\x07\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "rar4_archive",
            filename: "sample_v4.rar",
            category: "archive",
            data: b"Rar!\x1a\x07\x00\xcf\x90\x73\x00\x00\x0d\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "tar_posix_ustar",
            filename: "sample_backup.tar",
            category: "archive",
            data: {
                let mut v = vec![0u8; 512];
                v[0..11].copy_from_slice(b"payload.txt");
                v[257..263].copy_from_slice(b"ustar\x00");
                v
            },
        },

        // ---------------------------------------------------------------------
        // 4. Executables & Binaries
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "elf64_x86_64",
            filename: "sample_binary.elf",
            category: "executable",
            data: {
                let mut v = vec![0u8; 64];
                v[0..4].copy_from_slice(b"\x7fELF");
                v[4] = 2; // 64-bit
                v[5] = 1; // Little endian
                v[6] = 1; // ELF version 1
                v[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
                v[18..20].copy_from_slice(&0x3eu16.to_le_bytes()); // x86-64
                v
            },
        },
        BenchmarkSample {
            id: "elf32_arm",
            filename: "sample_arm.elf",
            category: "executable",
            data: {
                let mut v = vec![0u8; 52];
                v[0..4].copy_from_slice(b"\x7fELF");
                v[4] = 1; // 32-bit
                v[5] = 1; // Little endian
                v[6] = 1; // Version 1
                v[16..18].copy_from_slice(&3u16.to_le_bytes()); // ET_DYN (Shared lib)
                v[18..20].copy_from_slice(&40u16.to_le_bytes()); // ARM
                v
            },
        },
        BenchmarkSample {
            id: "pe_exe_x64",
            filename: "sample_app.exe",
            category: "executable",
            data: {
                let mut v = vec![0u8; 512];
                v[0..2].copy_from_slice(b"MZ");
                v[0x3C..0x40].copy_from_slice(&128u32.to_le_bytes()); // PE header offset
                v[128..132].copy_from_slice(b"PE\x00\x00");
                v[132..134].copy_from_slice(&0x8664u16.to_le_bytes()); // AMD64
                v[150..152].copy_from_slice(&0x0002u16.to_le_bytes()); // Characteristics: EXECUTABLE_IMAGE
                v
            },
        },
        BenchmarkSample {
            id: "pe_dll_x86",
            filename: "sample_lib.dll",
            category: "executable",
            data: {
                let mut v = vec![0u8; 512];
                v[0..2].copy_from_slice(b"MZ");
                v[0x3C..0x40].copy_from_slice(&128u32.to_le_bytes());
                v[128..132].copy_from_slice(b"PE\x00\x00");
                v[132..134].copy_from_slice(&0x014cu16.to_le_bytes()); // i386
                v[150..152].copy_from_slice(&0x2002u16.to_le_bytes()); // DLL
                v
            },
        },
        BenchmarkSample {
            id: "wasm_binary",
            filename: "sample_module.wasm",
            category: "executable",
            data: b"\x00asm\x01\x00\x00\x00\x01\x07\x01\x60\x02\x7f\x7f\x01\x7f\x03\x02\x01\x00".to_vec(),
        },
        BenchmarkSample {
            id: "macho64_arm64",
            filename: "sample_bin.dylib",
            category: "executable",
            data: b"\xcf\xfa\xed\xfe\x0c\x00\x00\x01\x00\x00\x00\x00\x06\x00\x00\x00\x02\x00\x00\x00\x40\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "java_class",
            filename: "Sample.class",
            category: "executable",
            data: b"\xca\xfe\xba\xbe\x00\x00\x00\x3d\x00\x10\x07\x00\x02\x01\x00\x06Sample".to_vec(),
        },

        // ---------------------------------------------------------------------
        // 5. Database, Network & Scientific
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "sqlite3_wal",
            filename: "sample_app.db",
            category: "database",
            data: {
                let mut v = vec![0u8; 100];
                v[0..16].copy_from_slice(b"SQLite format 3\x00");
                v[16..18].copy_from_slice(&4096u16.to_be_bytes()); // page size
                v[18] = 2; // file format write version (WAL)
                v[19] = 2; // file format read version (WAL)
                v
            },
        },
        BenchmarkSample {
            id: "pcap_microsec",
            filename: "sample_traffic.pcap",
            category: "database",
            data: b"\xd4\xc3\xb2\xa1\x02\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x01\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "pcap_nanosec",
            filename: "sample_nano.pcap",
            category: "database",
            data: b"\xa1\xb2\x3c\x4d\x02\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x01\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "pcapng_dump",
            filename: "sample_trace.pcapng",
            category: "database",
            data: b"\x0a\x0d\x0d\x0a\x1c\x00\x00\x00\x4d\x3c\x2b\x1a\x01\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff".to_vec(),
        },
        BenchmarkSample {
            id: "parquet_data",
            filename: "sample_table.parquet",
            category: "database",
            data: b"PAR1\x15\x00\x15\x3c\x15\x00\x25\x00\x15\x00\x25\x00PAR1".to_vec(),
        },
        BenchmarkSample {
            id: "avro_data",
            filename: "sample_events.avro",
            category: "database",
            data: b"Obj\x01\x04\x14avro.codec\x08null\x16avro.schema\x0a\"string\"\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "hdf5_scientific",
            filename: "sample_experiment.h5",
            category: "database",
            data: b"\x89HDF\r\n\x1a\n\x02\x08\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "fits_astronomy",
            filename: "sample_telescope.fits",
            category: "image",
            data: {
                let mut v = vec![b' '; 2880];
                let header = b"SIMPLE  =                    T / file does conform to FITS standard             BITPIX  =                  -32 / number of bits per data pixel                  NAXIS   =                    2 / number of data axes                            END                                                                             ";
                v[0..header.len()].copy_from_slice(header);
                v
            },
        },
        BenchmarkSample {
            id: "dicom_scan",
            filename: "sample_mri.dcm",
            category: "image",
            data: {
                let mut v = vec![0u8; 136];
                v[128..132].copy_from_slice(b"DICM");
                v[132..136].copy_from_slice(b"\x02\x00\x00\x00");
                v
            },
        },

        // ---------------------------------------------------------------------
        // 6. Documents & Text
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "pdf_doc",
            filename: "sample_report.pdf",
            category: "document",
            data: b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\nxref\n0 2\n0000000000 65535 f \n0000000009 00000 n \ntrailer\n<< /Size 2 /Root 1 0 R >>\nstartxref\n68\n%%EOF\n".to_vec(),
        },
        BenchmarkSample {
            id: "rtf_rich_text",
            filename: "sample_doc.rtf",
            category: "document",
            data: b"{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat\\deflang1033{\\fonttbl{\\f0\\fnil\\fcharset0 Calibri;}}\n{\\*\\generator Riched20 10.0.19041}\\viewkind4\\uc1 \n\\pard\\sa200\\sl276\\slmult1\\b\\f0\\fs22\\lang9 KindProbe Engine\\b0\\par\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "postscript_eps",
            filename: "sample_diagram.eps",
            category: "document",
            data: b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 100\n%%Title: Test Diagram\nnewpath 0 0 moveto 100 100 lineto stroke\n%%EOF\n".to_vec(),
        },

        // ---------------------------------------------------------------------
        // 7. Virtual Disks
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "qcow2_disk",
            filename: "sample_vm.qcow2",
            category: "disk",
            data: {
                let mut v = vec![0u8; 72];
                v[0..4].copy_from_slice(b"QFI\xfb");
                v[4..8].copy_from_slice(&3u32.to_be_bytes()); // version 3
                v
            },
        },
        BenchmarkSample {
            id: "vmdk_sparse",
            filename: "sample_drive.vmdk",
            category: "disk",
            data: {
                let mut v = vec![0u8; 512];
                v[0..4].copy_from_slice(b"KDMV");
                v[4..8].copy_from_slice(&1u32.to_le_bytes()); // version 1
                v
            },
        },
        BenchmarkSample {
            id: "vhd_dynamic",
            filename: "sample_system.vhd",
            category: "disk",
            data: {
                let mut v = vec![0u8; 512];
                v[0..8].copy_from_slice(b"conectix");
                v
            },
        },
        BenchmarkSample {
            id: "vhdx_storage",
            filename: "sample_volume.vhdx",
            category: "disk",
            data: {
                let mut v = vec![0u8; 512];
                v[0..8].copy_from_slice(b"vhdxfile");
                v
            },
        },

        // ---------------------------------------------------------------------
        // 8. Source Code & Configuration
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "python_async",
            filename: "sample_service.py",
            category: "text",
            data: b"#!/usr/bin/env python3\n\"\"\"Service Module\"\"\"\nimport asyncio\nimport sys\n\nasync def handle_request(req):\n    await asyncio.sleep(0.01)\n    return {\"status\": 200}\n\nif __name__ == '__main__':\n    asyncio.run(handle_request({}))\n".to_vec(),
        },
        BenchmarkSample {
            id: "rust_tokio",
            filename: "sample_main.rs",
            category: "text",
            data: b"// SPDX-License-Identifier: MIT\nuse std::sync::Arc;\n\npub struct WorkerPool {\n    size: usize,\n}\n\nimpl WorkerPool {\n    pub fn new(size: usize) -> Self {\n        Self { size }\n    }\n}\n\nfn main() {\n    let pool = WorkerPool::new(4);\n    println!(\"Pool running\");\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "c_posix",
            filename: "sample_server.c",
            category: "text",
            data: b"/* High-performance socket server */\n#include <stdio.h>\n#include <stdlib.h>\n#include <unistd.h>\n\nint main(int argc, char **argv) {\n    printf(\"Server initialized\\n\");\n    return 0;\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "cpp_templates",
            filename: "sample_container.cpp",
            category: "text",
            data: b"// Modern C++ Header\n#include <iostream>\n#include <vector>\n#include <memory>\n\nnamespace engine {\ntemplate<typename T>\nclass RingBuffer {\npublic:\n    explicit RingBuffer(size_t cap);\n};\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "java_spring",
            filename: "sample_ctrl.java",
            category: "text",
            data: b"package org.kindprobe.api;\n\nimport java.util.List;\nimport java.util.Map;\n\npublic class HealthController {\n    public Map<String, Object> getStatus() {\n        return Map.of(\"ok\", true);\n    }\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "go_server",
            filename: "sample_app.go",
            category: "text",
            data: b"package main\n\nimport (\n    \"fmt\"\n    \"net/http\"\n)\n\nfunc main() {\n    http.HandleFunc(\"/\", func(w http.ResponseWriter, r *http.Request) {\n        fmt.Fprintf(w, \"KindProbe\")\n    })\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "javascript_esm",
            filename: "sample_index.js",
            category: "text",
            data: b"#!/usr/bin/env node\nimport { createServer } from 'http';\n\nconst server = createServer((req, res) => {\n    res.writeHead(200);\n    res.end('ok');\n});\nserver.listen(3000);\n".to_vec(),
        },
        BenchmarkSample {
            id: "html5_semantic",
            filename: "sample_page.html",
            category: "text",
            data: "<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n    <meta charset=\"UTF-8\">\n    <title>系统监控</title>\n</head>\n<body>\n    <main>\n        <article><h1>系统状态正常</h1></article>\n    </main>\n</body>\n</html>\n".as_bytes().to_vec(),
        },
        BenchmarkSample {
            id: "xml_soap",
            filename: "sample_envelope.xml",
            category: "text",
            data: b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\">\n  <soap:Header/>\n  <soap:Body>\n    <m:GetStatusResponse xmlns:m=\"http://kindprobe.org/\">\n      <m:Active>true</m:Active>\n    </m:GetStatusResponse>\n  </soap:Body>\n</soap:Envelope>\n".to_vec(),
        },
        BenchmarkSample {
            id: "json_api_payload",
            filename: "sample_payload.json",
            category: "text",
            data: b"{\n  \"data\": {\n    \"type\": \"probes\",\n    \"id\": \"1006\",\n    \"attributes\": {\n      \"title\": \"KindProbe Global Registry\",\n      \"active\": true,\n      \"categories\": [\"image\", \"audio\", \"video\", \"executable\", \"database\"]\n    }\n  }\n}\n".to_vec(),
        },
        BenchmarkSample {
            id: "yaml_k8s_deploy",
            filename: "sample_deploy.yaml",
            category: "text",
            data: b"---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: kindprobe-worker\nspec:\n  replicas: 4\n  selector:\n    matchLabels:\n      app: worker\n  template:\n    spec:\n      containers:\n      - name: probe\n        image: kindprobe:latest\n".to_vec(),
        },
        BenchmarkSample {
            id: "toml_workspace",
            filename: "sample_config.toml",
            category: "text",
            data: b"[workspace]\nmembers = [\n    \"crates/core\",\n    \"crates/cli\",\n    \"crates/rules\",\n]\nresolver = \"2\"\n\n[profile.release]\nlto = true\ncodegen-units = 1\npanic = \"abort\"\n".to_vec(),
        },
        BenchmarkSample {
            id: "markdown_gfm",
            filename: "sample_readme.md",
            category: "text",
            data: b"# KindProbe Benchmarks\n\n| Stage | Latency | QPS |\n| :--- | :--- | :--- |\n| Phase 13 | 47.17 ns | 21.2 M |\n| Phase 16 | 29.89 ns | 33.4 M |\n\n- Pure safe Rust\n- Zero panic\n".to_vec(),
        },
        BenchmarkSample {
            id: "csv_rfc4180",
            filename: "sample_records.csv",
            category: "text",
            data: b"id,name,description,score\n1,\"Standard, RFC 4180\",Valid CSV record,99.5\n2,\"Multi\nLine\",Quoted multiline,95.0\n3,\"Escaped \"\"quotes\"\"\",Quote escape,98.2\n".to_vec(),
        },
        BenchmarkSample {
            id: "tsv_records",
            filename: "sample_table.tsv",
            category: "text",
            data: b"metric\tphase13\tphase14\tphase15\tphase16\npng_core\t840ns\t479ns\t31ns\t36ns\njpeg_core\t960ns\t640ns\t42ns\t28ns\n".to_vec(),
        },
        BenchmarkSample {
            id: "shell_posix",
            filename: "sample_entrypoint.sh",
            category: "text",
            data: b"#!/bin/sh\nset -eu\necho \"Executing entrypoint script\"\nexec \"$@\"\n".to_vec(),
        },
        BenchmarkSample {
            id: "verilog_pipeline",
            filename: "sample_pipe.v",
            category: "text",
            data: b"`timescale 1ns / 1ps\nmodule pipeline_reg (\n    input wire clk,\n    input wire rst,\n    input wire [31:0] d,\n    output reg [31:0] q\n);\nalways @(posedge clk or posedge rst) begin\n    if (rst) q <= 32'b0;\n    else q <= d;\nend\nendmodule\n".to_vec(),
        },
        BenchmarkSample {
            id: "diff_unified",
            filename: "sample_changes.diff",
            category: "text",
            data: b"diff --git a/src/engine.rs b/src/engine.rs\nindex e69de29..4996f38 100644\n--- a/src/engine.rs\n+++ b/src/engine.rs\n@@ -1,3 +1,5 @@\n+pub fn init_buckets();\n pub fn probe();\n".to_vec(),
        },

        // ---------------------------------------------------------------------
        // 9. Near-Miss Rejection Samples & High-Entropy Noise
        // ---------------------------------------------------------------------
        BenchmarkSample {
            id: "near_miss_png",
            filename: "bad_header.png",
            category: "binary",
            data: b"\x89PNG\r\n\x1a\x01\x00\x00\x00\x0dIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "near_miss_jpeg",
            filename: "corrupt.jpg",
            category: "binary",
            data: b"\xff\xd8\x00\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xaa\xbb\xcc".to_vec(),
        },
        BenchmarkSample {
            id: "near_miss_zip",
            filename: "invalid_magic.zip",
            category: "binary",
            data: b"PK\x03\x09\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "near_miss_elf",
            filename: "invalid_class.elf",
            category: "binary",
            data: b"\x7fELF\x03\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        },
        BenchmarkSample {
            id: "near_miss_pdf",
            filename: "corrupt_magic.pdf",
            category: "binary",
            data: b"%PDX-1.5\n1 0 obj\n<< /Type /Catalog >>\nendobj\n".to_vec(),
        },
        BenchmarkSample {
            id: "near_miss_sqlite",
            filename: "corrupt_version.db",
            category: "binary",
            data: b"SQLite format 4\x00\x10\x00\x01\x01\x00\x40\x20\x20".to_vec(),
        },
        BenchmarkSample {
            id: "random_noise_64",
            filename: "random_64.bin",
            category: "binary",
            data: (0..64u8).map(|i| (i.wrapping_mul(37)).wrapping_add(13)).collect(),
        },
        BenchmarkSample {
            id: "random_noise_4k",
            filename: "random_4k.bin",
            category: "binary",
            data: (0..4096u16).map(|i| ((i * 109 + 89) % 256) as u8).collect(),
        },
        BenchmarkSample {
            id: "random_noise_64k",
            filename: "random_64k.bin",
            category: "binary",
            data: (0..65536u32).map(|i| ((i * 104729 + 123) % 256) as u8).collect(),
        },
    ]
}

#[test]
fn test_fresh_large_scale_multi_stage_benchmark() {
    let samples = get_all_samples();
    let total_sample_count = samples.len();

    println!("\n==================================================================");
    println!("     KindProbe Fresh Large-Scale Multi-Stage Benchmark Suite      ");
    println!("==================================================================");
    println!("Total Unique Fresh Formats & Payloads: {}", total_sample_count);

    // =========================================================================
    // Part 1: High-Volume In-Memory Multi-Format Stress Test (Phase 16 Engine)
    // =========================================================================
    // 1,000,000 total interleaved operations across all samples
    let total_ops = 1_000_000usize;
    let rounds = total_ops / total_sample_count;
    let actual_ops = rounds * total_sample_count;

    let opts = DetectOptions::default();

    println!("\n[1] Massive Interleaved Memory Stress Test (Phase 16 Engine):");
    println!("    Total Operations:     {} iterations ({} rounds x {} samples)", actual_ops, rounds, total_sample_count);

    // 1.1 Full Report
    let start_full = Instant::now();
    for _ in 0..rounds {
        for s in &samples {
            let _ = detect_bytes(&s.data, &opts);
        }
    }
    let dur_full = start_full.elapsed();
    let ns_per_full = dur_full.as_nanos() as f64 / actual_ops as f64;
    let qps_full = actual_ops as f64 / dur_full.as_secs_f64();
    println!("    Full Report Latency:  {:8.2} ns / op", ns_per_full);
    println!("    Full Report QPS:      {:8.2} thousand ops / sec", qps_full / 1000.0);

    // 1.2 Zero-Allocation Core
    let start_core = Instant::now();
    for _ in 0..rounds {
        for s in &samples {
            let _ = detect_bytes_core(&s.data, &opts);
        }
    }
    let dur_core = start_core.elapsed();
    let ns_per_core = dur_core.as_nanos() as f64 / actual_ops as f64;
    let qps_core = actual_ops as f64 / dur_core.as_secs_f64();
    let speedup = ns_per_full / ns_per_core;
    println!("    Zero-Alloc Core:      {:8.2} ns / op", ns_per_core);
    println!("    Zero-Alloc Core QPS:  {:8.2} thousand ops / sec", qps_core / 1000.0);
    println!("    Zero-Alloc Speedup:   {:8.2}x faster", speedup);

    // =========================================================================
    // Part 2: Real Disk Dataset Export & Phase Binaries Cross-Benchmark
    // =========================================================================
    let target_dir = Path::new("target").join("fresh_benchmark_dataset");
    if target_dir.exists() {
        let _ = fs::remove_dir_all(&target_dir);
    }
    fs::create_dir_all(&target_dir).expect("create fresh_benchmark_dataset");

    // Write all samples to disk
    let mut file_paths: Vec<(PathBuf, &'static str)> = Vec::new();
    for s in &samples {
        let path = target_dir.join(s.filename);
        let mut f = File::create(&path).expect("create benchmark sample file");
        f.write_all(&s.data).expect("write sample data");
        file_paths.push((path, s.id));
    }

    println!("\n[2] Real Out-of-Process Phase Binary Benchmark on Disk Files:");
    println!("    Generated Test Dataset: {} real files in target/fresh_benchmark_dataset", file_paths.len());

    let phase_binaries = [
        ("Phase 13 (Baseline)", "target/phase_binaries/kindprobe_phase13.exe"),
        ("Phase 14 (Buckets)",  "target/phase_binaries/kindprobe_phase14.exe"),
        ("Phase 15 (7-Domains)","target/phase_binaries/kindprobe_phase15.exe"),
        ("Phase 16 (Token+SWAR)","target/phase_binaries/kindprobe_phase16.exe"),
    ];

    struct PhaseStats {
        name: &'static str,
        total_engine_micros: u64,
        avg_engine_micros: f64,
        wall_clock_ms: f64,
        files_passed: usize,
    }

    let mut results: Vec<PhaseStats> = Vec::new();

    for &(phase_name, exe_path) in &phase_binaries {
        let path = Path::new(exe_path);
        if !path.exists() {
            println!("    [SKIP] {} binary not found at {}", phase_name, exe_path);
            continue;
        }

        let start_phase = Instant::now();
        let mut total_engine_micros = 0u64;
        let mut files_passed = 0usize;

        for (fp, _id) in &file_paths {
            let output = Command::new(path)
                .arg("--json")
                .arg(fp)
                .output()
                .expect("execute phase binary");

            if output.status.success() || output.status.code() == Some(3) {
                // Parse stdout JSON
                if let Ok(v) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(stats) = v.get("stats") {
                        if let Some(dur) = stats.get("duration_micros").and_then(|d| d.as_u64()) {
                            total_engine_micros += dur;
                            files_passed += 1;
                        }
                    }
                }
            }
        }
        let wall_clock = start_phase.elapsed();

        let avg = if files_passed > 0 {
            total_engine_micros as f64 / files_passed as f64
        } else {
            0.0
        };

        results.push(PhaseStats {
            name: phase_name,
            total_engine_micros,
            avg_engine_micros: avg,
            wall_clock_ms: wall_clock.as_secs_f64() * 1000.0,
            files_passed,
        });
    }

    // Detailed per-file differential analysis between Phase 15 and Phase 16
    let p15_path = Path::new("target/phase_binaries/kindprobe_phase15.exe");
    let p16_path = Path::new("target/phase_binaries/kindprobe_phase16.exe");
    if p15_path.exists() && p16_path.exists() {
        println!("\n    Per-File Detailed Comparison (Phase 15 vs Phase 16, delta > 5 µs):");
        for (fp, id) in &file_paths {
            let out15 = Command::new(p15_path).arg("--json").arg(fp).output().unwrap();
            let out16 = Command::new(p16_path).arg("--json").arg(fp).output().unwrap();
            let v15: Value = serde_json::from_slice(&out15.stdout).unwrap_or(Value::Null);
            let v16: Value = serde_json::from_slice(&out16.stdout).unwrap_or(Value::Null);
            let d15 = v15["stats"]["duration_micros"].as_u64().unwrap_or(0);
            let d16 = v16["stats"]["duration_micros"].as_u64().unwrap_or(0);
            let diff = (d16 as i64) - (d15 as i64);
            if diff.abs() > 5 {
                println!("      {:<20}: P15 = {:4} µs, P16 = {:4} µs (diff = {:+4} µs)", id, d15, d16, diff);
            }
        }
    }

    println!("\n    +-----------------------+--------------+---------------+----------------+----------------+---------------+");
    println!("    | Phase Stage           | Files Tested | Total Engine  | Avg Latency    | Rel. Speedup   | Wall Clock ms |");
    println!("    +-----------------------+--------------+---------------+----------------+----------------+---------------+");

    let baseline_avg = results.first().map(|r| r.avg_engine_micros).unwrap_or(1.0);
    for r in &results {
        let rel_speedup = if r.avg_engine_micros > 0.0 {
            baseline_avg / r.avg_engine_micros
        } else {
            1.0
        };
        println!(
            "    | {:<21} | {:>12} | {:>10} µs | {:>11.2} µs | {:>13.2}x | {:>10.1} ms |",
            r.name, r.files_passed, r.total_engine_micros, r.avg_engine_micros, rel_speedup, r.wall_clock_ms
        );
    }
    println!("    +-----------------------+--------------+---------------+----------------+----------------+---------------+");

    // Clean up temporary dataset
    let _ = fs::remove_dir_all(&target_dir);

    println!("\n==================================================================\n");
}
