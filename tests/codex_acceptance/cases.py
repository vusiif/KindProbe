
from pathlib import Path
import zipfile, json, subprocess,hashlib
root=Path('/accept');fixtures=root/'fixtures';exe=root/'build/release/kindprobe'
ct='<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'
rels='<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="r1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'
valid='<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document>'
cases=[('valid_docx',valid,'docx'),('invalid_root_shallow','<evil><x/></evil>',None),('invalid_root_deep','<evil>'+'<x>'*70+'</x>'*70+'</evil>',None),('wrong_namespace_prefixed','<w:document xmlns:w="urn:evil"><w:body/></w:document>',None),('no_namespace_empty_root','<document/>',None),('no_namespace_document','<document><body/></document>',None),('invalid_multiple_roots',valid+'<evil/>',None)]
rows=[]
for name,xml,expected in cases:
 p=fixtures/(name+'.docx')
 with zipfile.ZipFile(p,'w',zipfile.ZIP_DEFLATED) as z:
  for filename,data in [('[Content_Types].xml',ct),('_rels/.rels',rels),('word/document.xml',xml)]:z.writestr(filename,data)
 # All generated XML strings are deliberate fixtures; package bytes are hashed.
 proc=subprocess.run([str(exe),'--json',str(p)],capture_output=True,text=True,timeout=5)
 actual=json.loads(proc.stdout)
 passed=actual.get('format_id')==expected
 row={'id':name,'pass':passed,'expected_format':expected,'returncode':proc.returncode,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'actual':actual}
 rows.append(row)
 print(json.dumps({'id':name,'pass':passed,'actual_format':actual.get('format_id'),'inspection':actual.get('inspection')}))
for name,args,expected in [('missing_path',['--json','/accept/fixtures/not_present'],4),('directory',['--json','/accept/fixtures'],4),('missing_arg',[],2),('unknown_option',['--bad-option'],2)]:
 p=subprocess.run([str(exe)]+args,capture_output=True,text=True,timeout=5)
 rows.append({'id':name,'pass':p.returncode==expected,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
(root/'cli-cases.json').write_text(json.dumps(rows,indent=2))
