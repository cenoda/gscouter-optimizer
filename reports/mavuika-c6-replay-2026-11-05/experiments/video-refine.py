from pathlib import Path
import subprocess,json,random,resource,copy,time
out=Path('runs/mavuika-replay/video-refine');out.mkdir(exist_ok=True)
src=Path('runs/mavuika-replay/video-approx-swap1.gcsl').read_text();header=src[:src.index('while 1 {')];prefix=Path('runs/mavuika-replay/video-prefix.txt').read_text()
rows=[];cache={};rng=random.Random(3234)
def render(v):
 p=prefix.replace('xilonen skill, attack:2;', 'xilonen skill;\ndelay('+str(v['xnwait'])+');\nxilonen attack:2;')
 p=p.replace('mavuika burst;', 'delay('+str(v['preQ'])+');\nmavuika burst;')
 s=header+'while 1 {\n'+p
 for j,c in enumerate(v['cs']):
  if j:s+=('mav_dc_cancel(6);' if v['ds'][j-1] else 'mav_dc_hidden(6);')+'\n'
  else:s+='mavuika charge;\n'
  if c>1:s+='mavuika charge:'+str(c-1)+';\n'
  if v['hs'][j]:s+='mavuika charge[hold='+str(v['hs'][j])+'];\n'
  s+='mavuika charge[final=1];\n'
 return s+'}\n'
def score(v):
 key=json.dumps(v,sort_keys=True)
 if key in cache:return cache[key]
 i=len(rows);p=out/f'{i:05}.gcsl';p.write_text(render(v));r0=resource.getrusage(resource.RUSAGE_CHILDREN);w=time.monotonic()
 r=subprocess.run(['target/release/examples/replay',str(p),'8','30000000000000','0'],text=True,capture_output=True)
 r1=resource.getrusage(resource.RUSAGE_CHILDREN);row={'id':i,'parameters':copy.deepcopy(v),'cpu':r1.ru_utime+r1.ru_stime-r0.ru_utime-r0.ru_stime,'wall':time.monotonic()-w}
 if r.returncode:row['error']=r.stderr;val=-1
 else:
  d=json.loads(r.stdout);row.update(d['team']);val=d['team']['mean'];(out/f'{i:05}.json').write_text(r.stdout)
 rows.append(row);cache[key]=val;return val
seeds=[]
for cs in [[1]*6,[1]*7,[3,3,1,1],[3,3,3],[4,4,3],[5,5],[2,2,2,2]]:
 for d in [0,1]:
  v={'cs':cs,'hs':[0]*len(cs),'ds':[d]*(len(cs)-1),'preQ':0,'xnwait':0};seeds.append(v);score(v)
beam=sorted([(r['mean'],r['parameters']) for r in rows if 'mean'in r],key=lambda r:r[0],reverse=True)[:12];bestscore,best=beam[0];print('base',bestscore,best,flush=True)
for i in range(2500):
 v=copy.deepcopy(beam[rng.randrange(len(beam))][1] if rng.random()<.96 else rng.choice(seeds));n=len(v['cs'])
 for _ in range(2 if rng.random()<.15 else 1):
  k=rng.randrange(13)
  if k<3:v['cs'][rng.randrange(n)]=rng.randrange(1,7)
  elif k<7:v['hs'][rng.randrange(n)]=rng.randrange(0,46)
  elif k==7 and n>1:v['ds'][rng.randrange(n-1)]^=1
  elif k==8 and n<8:
   at=rng.randrange(n+1);v['cs'].insert(at,1);v['hs'].insert(at,rng.choice([0,6,30,40]));v['ds'].insert(min(at,n-1),rng.randrange(2));n+=1
  elif k==9 and n>2:
   at=rng.randrange(n);v['cs'].pop(at);v['hs'].pop(at);v['ds'].pop(min(at,n-2));n-=1
  elif k==10:v['preQ']=rng.randrange(0,46)
  else:v['xnwait']=rng.randrange(0,46)
 a=score(v)
 if a>bestscore:bestscore=a;best=copy.deepcopy(v);print('best',len(rows),round(a),best,flush=True)
 if a>beam[-1][0] and v not in [x[1] for x in beam]:beam=sorted(beam+[(a,v)],key=lambda x:x[0],reverse=True)[:12]
 if i%100==0:(out/'summary.json').write_text(json.dumps(rows,indent=2))
(out/'summary.json').write_text(json.dumps(rows,indent=2));(out/'selected-parameters.json').write_text(json.dumps(best,indent=2));(out/'selected.gcsl').write_text(render(best));print('FINAL',bestscore,best,flush=True)
