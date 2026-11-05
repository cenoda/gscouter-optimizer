from pathlib import Path
import subprocess,json,random,time,resource,copy,re
out=Path('runs/mavuika-replay/refine-v3');out.mkdir(exist_ok=True)
header=Path('runs/mavuika-external-research/export-stronger/candidate-0.gcsl').read_text().split('active mavuika;')[0]
raw=Path('runs/mavuika-external-research/external-stronger-rotation.gcsl').read_text()
a=raw.index('    mavuika burst;');b=raw.index('  } else {',a);c=raw.index('    mavuika burst;',b);e=raw.index('  }\n}',c)
parts=[raw[:a],raw[b:c],raw[e:]]
hidden=raw[:raw.index('active mavuika;')].replace('mav_dc_cancel(', 'mav_dc_hidden(').replace('charge_cancel=1, travel=d_travel_f','charge_cancel=1, collision=0')
rows=[];cache={};rng=random.Random(3233)
def render(v):
 def post(odd):
  t='mavuika burst;\n'
  for j,count in enumerate(v['cs']):
   if j:t+=('mav_dc_cancel(6);' if v['ds'][j-1] else 'mav_dc_hidden(6);')+'\n'
   if not j:t+='mavuika charge;\n'
   if count>1:t+='mavuika charge:'+str(count-1)+';\n'
   hold=v['hs'][j]
   if odd and j==len(v['cs'])-2:hold=min(45,hold+v['odd'])
   if hold:t+='mavuika charge[hold='+str(hold)+'];\n'
   t+='mavuika charge[final=1];\n'
  if odd:t+='mav_dc_cancel(6);mavuika jump;\n'
  return t
 s=hidden+parts[0]+post(False)+parts[1]+post(True)+parts[2]
 s=s.replace('  mavuika skill[hold=1];','  mavuika skill[hold='+str(v['holdE'])+'];')
 if v['order']:
  s=s.replace('  xilo skill, attack:2;\n  pick_up_crystallize("any");\n','')
  s=s.replace('    mavuika swap;','    xilo skill, attack:2;\n    pick_up_crystallize("any");\n    mavuika swap;')
 if v['pre']:s=s.replace('    mavuika burst;', '    delay('+str(v['pre'])+');\n    mavuika burst;')
 s=s.replace('active mavuika;','')
 return header+'active mavuika;\n'+s[:s.index('for let i=0;')]+'\nwhile 1 {\n'+s[s.index('for let i=0;'):]+'\n}\n'
def score(v):
 key=json.dumps(v,sort_keys=True)
 if key in cache:return cache[key]
 i=len(rows);p=out/f'{i:05}.gcsl';p.write_text(render(v));r0=resource.getrusage(resource.RUSAGE_CHILDREN);w=time.monotonic()
 r=subprocess.run(['target/release/examples/replay',str(p),'8','26000000000000','150'],text=True,capture_output=True)
 r1=resource.getrusage(resource.RUSAGE_CHILDREN);row={'id':i,'parameters':copy.deepcopy(v),'cpu':r1.ru_utime+r1.ru_stime-r0.ru_utime-r0.ru_stime,'wall':time.monotonic()-w}
 if r.returncode:row['error']=r.stderr;val=-1
 else:
  d=json.loads(r.stdout);row.update(d['team']);val=d['team']['mean'];(out/f'{i:05}.json').write_text(r.stdout)
 rows.append(row);cache[key]=val;return val
base={'cs':[1]*6,'hs':[0,0,0,0,30,40],'ds':[1]*5,'odd':10,'holdE':1,'order':0,'pre':0}
val=score(base);print('base',val,flush=True)
if not 1850000<val<1950000:raise RuntimeError('baseline reconstruction failed')
seeds=[base]
for cs in [[3,3,1,1],[3,3,3],[4,4,3],[5,5],[2,2,2,2],[1]*7]:
 for hide in [0,1]:
  for order in [0,1]:
   v=copy.deepcopy(base);v.update(cs=cs,hs=[6]*len(cs),ds=[hide]*(len(cs)-1),holdE=0,order=order,odd=0);seeds.append(v);score(v)
beam=sorted([(r['mean'],r['parameters']) for r in rows if 'mean'in r],key=lambda r:r[0],reverse=True)[:12];bestscore,best=beam[0];print('seeds best',bestscore,best,flush=True)
for i in range(2000):
 v=copy.deepcopy(beam[rng.randrange(len(beam))][1] if rng.random()<.96 else rng.choice(seeds));n=len(v['cs'])
 for _ in range(2 if rng.random()<.15 else 1):
  k=rng.randrange(13)
  if k<3:v['cs'][rng.randrange(n)]=rng.randrange(1,7)
  elif k<7:v['hs'][rng.randrange(n)]=rng.randrange(0,46)
  elif k==7 and n>1:v['ds'][rng.randrange(n-1)]^=1
  elif k==8 and n<7:
   at=rng.randrange(n+1);v['cs'].insert(at,1);v['hs'].insert(at,rng.choice([0,6,30,40]));v['ds'].insert(min(at,n-1),rng.randrange(2));n+=1
  elif k==9 and n>2:
   at=rng.randrange(n);v['cs'].pop(at);v['hs'].pop(at);v['ds'].pop(min(at,n-2));n-=1
  elif k==10:v['odd']=rng.randrange(-10,16)
  elif k==11:v['holdE']^=1
  else:v['order']^=1
 a=score(v)
 if a>bestscore:bestscore=a;best=copy.deepcopy(v);print('best',len(rows),round(a),best,flush=True)
 if a>beam[-1][0] and v not in [x[1] for x in beam]:beam=sorted(beam+[(a,v)],key=lambda x:x[0],reverse=True)[:12]
 if i%100==0:(out/'summary.json').write_text(json.dumps(rows,indent=2))
(out/'summary.json').write_text(json.dumps(rows,indent=2));(out/'selected-parameters.json').write_text(json.dumps(best,indent=2));(out/'selected.gcsl').write_text(render(best));print('FINAL',bestscore,best,flush=True)
