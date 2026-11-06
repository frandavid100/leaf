#!/usr/bin/env python3
"""Compare actual instrumented before/after GUI binaries on the same runner.

AB/BA order, isolated fresh profiles, identical synthetic inputs. Timings are
reported rather than asserted; deterministic host work counts are asserted.
"""
import json, os, pathlib, statistics, subprocess, sys, tempfile
out=pathlib.Path('performance-results');out.mkdir(exist_ok=True)
binaries={'before':str(pathlib.Path(sys.argv[1]).resolve()),'after':str(pathlib.Path(sys.argv[2]).resolve())}
for binary in binaries.values():os.chmod(binary,0o755)
results=[]
for size in [120,600,1800]:
 for repeat,order in enumerate([['before','after'],['after','before']]):
  for label in order:
   report=(out/f'{size}-{repeat}-{label}.json').resolve()
   with tempfile.TemporaryDirectory(prefix='leaf-perf-profile-') as profile:
    env=os.environ|{'XDG_CONFIG_HOME':profile+'/config','XDG_CACHE_HOME':profile+'/cache','XDG_STATE_HOME':profile+'/state','LANGUAGE':'en','GSK_RENDERER':'cairo','LEAF_PERF_PARAGRAPHS':str(size),'LEAF_PERF_REPORT':str(report)}
    for folder in ['config','cache','state']:pathlib.Path(profile,folder).mkdir()
    with (out/f'{size}-{repeat}-{label}.log').open('w') as log:
     subprocess.run(['xvfb-run','-a','timeout','180s',binaries[label],'--performance-ui-test'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=190)
   value=json.loads(report.read_text());assert value['canonical_document_unchanged'];value.update(label=label,repeat=repeat);results.append(value)
summary=[]
for size in [120,600,1800]:
 group=[r for r in results if r['paragraphs']==size];pages={r['pages'] for r in group};assert len(pages)==1
 for name in [p['phase'] for p in group[0]['phases']]:
  row={'paragraphs':size,'pages':group[0]['pages'],'phase':name}
  for label in ['before','after']:
   phase=[next(p for p in r['phases'] if p['phase']==name) for r in group if r['label']==label]
   row[label]={'median_ms':statistics.median(p['wall_ms'] for p in phase),'counts':phase[0]['counts'],'median_rss_kib':statistics.median(p['rss_kib'] for p in phase)}
  if name=='cursor_right_80':assert row['after']['counts']['overlay_queue']<=160
  if name=='thumbnail_cache_hits_80':assert row['after']['counts']['thumbnail_bounds']<row['before']['counts']['thumbnail_bounds']
  if name=='inactive_search_refresh_4':assert row['after']['counts'].get('search_rows',0)==0
  summary.append(row)
(out/'comparison.json').write_text(json.dumps({'schema':1,'order':'AB/BA','repeats_per_size':2,'summary':summary,'limits':'Synthetic ordinary body text, Xvfb/Cairo, fresh profiles, same runner. Inclusive span timings overlap; boundary calls include adapter overhead. No general speed/memory or cold-disk claims.'},indent=2)+'\n')
print('LEAF_PERFORMANCE_COMPARISON_OK',flush=True)
for row in summary:
 if row['phase'] in ['thumbnail_cache_hits_80','cursor_right_80','inactive_search_refresh_4']:print(json.dumps(row),flush=True)
