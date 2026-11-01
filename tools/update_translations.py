#!/usr/bin/env python3
"""Extract Leaf's literal t! messages for gettext/Weblate; deterministic output."""
import argparse,json,pathlib,re,subprocess,sys,gettext,xml.etree.ElementTree as ET
ROOT=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--compile',action='store_true');a=p.parse_args()
refs={}
for file in sorted((ROOT/'src').rglob('*.rs')):
 if file.name=='i18n.rs':continue
 text=file.read_text()
 for m in re.finditer(r'(?:wordcraft_gnome::)?t!\(("(?:\\.|[^"\\])*")\)',text):
  value=json.loads(m[1]);refs.setdefault(value,[]).append(f'{file.relative_to(ROOT)}:{text.count(chr(10),0,m.start())+1}')
desktop=ROOT/'data/io.github.frandavid100.leaf.desktop'
meta=ROOT/'data/io.github.frandavid100.leaf.metainfo.xml'
desktop_source=pathlib.Path(str(desktop)+'.in').read_text()
meta_root=ET.parse(str(meta)+'.in').getroot()
metadata_messages=[]
for line in desktop_source.splitlines():
 if line.startswith(('GenericName=','Comment=','Keywords=')):metadata_messages.append(line.split('=',1)[1])
for node in list(meta_root.findall('summary'))+list(meta_root.findall('description/p')):
 metadata_messages.append(node.text or '')
for msg in metadata_messages:refs.setdefault(msg,[]).append('data/*.in')
def q(s):return json.dumps(s,ensure_ascii=False)
header='Project-Id-Version: Leaf\nReport-Msgid-Bugs-To: https://github.com/frandavid100/leaf/issues\nMIME-Version: 1.0\nContent-Type: text/plain; charset=UTF-8\nContent-Transfer-Encoding: 8bit\n'
pot='msgid ""\nmsgstr '+q(header)+'\n\n'
for key in sorted(refs):pot+='#: '+' '.join(refs[key])+'\nmsgid '+q(key)+'\nmsgstr ""\n\n'
messages='pub const MESSAGES: &[&str] = &[\n'+''.join('    '+q(key)+',\n' for key in sorted(refs))+'];\n'
for name,content in [('po/leaf.pot',pot),('po/messages.rs',messages)]:
 path=ROOT/name
 if a.check:
  if not path.exists() or path.read_text()!=content:sys.exit(f'{name} is outdated; run tools/update_translations.py')
 else:path.write_text(content)
catalogs={}
for path in sorted((ROOT/'po').glob('*.po')):
 locale=path.stem
 dest=ROOT/'po/locale'/locale/'LC_MESSAGES';dest.mkdir(parents=True,exist_ok=True)
 subprocess.run(['msgfmt','--check','--check-format','-o',str(dest/'leaf.mo'),str(path)],check=True)
 with (dest/'leaf.mo').open('rb') as f:catalog=gettext.GNUTranslations(f)
 catalogs[locale]=catalog
 if locale in ['en','es']:
  missing=[key for key in refs if key not in catalog._catalog or not catalog._catalog[key]]
  if missing:sys.exit(f'{locale}: missing translations: {missing}')
 for key in refs:
  translated=catalog.gettext(key)
  def fields(s):return sorted(re.findall(r'(?<!\{)\{([^{}]*)\}(?!\})',s))
  if fields(key)!=fields(translated):sys.exit(f'{locale}: changed placeholders: {key}')
# Export translated desktop/AppStream metadata from the same gettext catalogs.
desktop_out=[]
for line in desktop_source.splitlines():
 desktop_out.append(line)
 if line.startswith(('GenericName=','Comment=','Keywords=')):
  key,value=line.split('=',1)
  for lang,cat in catalogs.items():
   if lang!='en' and cat.gettext(value)!=value:desktop_out.append(f'{key}[{lang}]={cat.gettext(value)}')
for summary in list(meta_root.findall('summary')):
 for lang,cat in catalogs.items():
  if lang!='en' and cat.gettext(summary.text)!=summary.text:ET.SubElement(meta_root,'summary',{'{http://www.w3.org/XML/1998/namespace}lang':lang}).text=cat.gettext(summary.text)
for desc in meta_root.findall('description'):
 for child in list(desc):
  for lang,cat in catalogs.items():
   if lang!='en' and cat.gettext(child.text)!=child.text:ET.SubElement(desc,child.tag,{'{http://www.w3.org/XML/1998/namespace}lang':lang}).text=cat.gettext(child.text)
ET.indent(meta_root,space='  ')
meta_out='<?xml version="1.0" encoding="UTF-8"?>\n'+ET.tostring(meta_root,encoding='unicode')+'\n'
for path,content in [(desktop,'\n'.join(desktop_out)+'\n'),(meta,meta_out)]:
 if a.check:
  if path.read_text()!=content:sys.exit(f'{path} is outdated; regenerate translations')
 else:path.write_text(content)
print(f'{len(refs)} source messages; complete English and Spanish catalogs')
