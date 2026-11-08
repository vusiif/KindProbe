
use std::io::{Cursor,Write};
use kindprobe::{detect_bytes,detect_reader,detect_path,DetectOptions};
use serde_json::json;
fn row(id:&str, pass:bool, detail:serde_json::Value){println!("{}",json!({"id":id,"pass":pass,"detail":detail}));}
fn cfb_case(name:&str,nested:bool,version:u16,table:bool){
 let p=format!("/accept/fixtures/{}.cfb",name);
 let mut comp=cfb::create(&p).unwrap();
 let mut fib=vec![0u8;64];
 fib[0..2].copy_from_slice(&0xa5ecu16.to_le_bytes());
 fib[2..4].copy_from_slice(&version.to_le_bytes());
 fib[10..12].copy_from_slice(&0x1000u16.to_le_bytes());
 fib[12..14].copy_from_slice(&0x00bfu16.to_le_bytes());
 if nested {
  comp.create_storage("/EmbeddedWord").unwrap();
  comp.create_stream("/Workbook").unwrap().write_all(b"OUTER WORKBOOK CONTAINER").unwrap();
 }
 let prefix=if nested {"/EmbeddedWord/"} else {"/"};
 comp.create_stream(format!("{}WordDocument",prefix)).unwrap().write_all(&fib).unwrap();
 if table { comp.create_stream(format!("{}0Table",prefix)).unwrap().write_all(&[0u8;64]).unwrap(); }
 drop(comp);
 let r=detect_path(std::path::Path::new(&p),&DetectOptions::default()).unwrap();
 row(name,r.format_id.as_deref()!=Some("doc"),json!({"expected":"must not confirm doc","actual":r}));
}
fn main(){
 let o=DetectOptions::default();
 let png=b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR\0\0\0\x01\0\0\0\x01\x08\x02\0\0\0";
 let p=std::path::Path::new("/accept/fixtures/png_without_suffix");std::fs::write(p,png).unwrap();
 let a=detect_path(p,&o).unwrap();let b=detect_bytes(png,&o).unwrap();let c=detect_reader(&mut Cursor::new(png),&o).unwrap();
 row("api_missing_extension",matches!(a.extension_status,kindprobe::ExtensionStatus::Missing)&&matches!(b.extension_status,kindprobe::ExtensionStatus::Missing)&&matches!(c.extension_status,kindprobe::ExtensionStatus::Missing),json!({"path":a.extension_status,"bytes":b.extension_status,"reader":c.extension_status}));
 row("api_content_identity",a.format_id==b.format_id&&b.format_id==c.format_id,json!({"path":a.format_id,"bytes":b.format_id,"reader":c.format_id}));
 let r=detect_bytes(b"\x89PNG\r\n\x1a\n",&o).unwrap();
 row("png_signature_only_evidence",!matches!(r.verification,kindprobe::Verification::Structure),json!({"expected":"no fabricated IHDR structure evidence on 8-byte input","actual":r}));
 let mut zero=o.clone();zero.max_source_bytes_read=0;
 let r=detect_bytes(png,&zero).unwrap();row("zero_read_budget",matches!(r.inspection,kindprobe::Inspection::Limited)&&r.stats.source_bytes_read==0,json!(r));
 cfb_case("cfb_forged_fib_missing_table",false,0x00c1,false);
 cfb_case("cfb_embedded_word_in_workbook",true,0x00c1,true);
 cfb_case("cfb_unsupported_fib_version",false,0xffff,true);
}
