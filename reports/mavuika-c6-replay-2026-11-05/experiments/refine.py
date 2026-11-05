from pathlib import Path
import subprocess,json,time,random,resource,copy
out=Path('runs/mavuika-replay/refine');out.mkdir(exist_ok=True)
header=Path('runs/mavuika-external-research/export-stronger/candidate-0.gcsl').read_text().split('active mavuika;')[0]
rng=random.Random(323);rows=[];cache={}
def render(v):
 s='active mavuika;\nfor let cycle=0; 1; cycle=cycle+1 {\nmavuika skill[hold='+str(v['holdE'])+'];\nnicole skill;\n'
 if v['nicN']:s+='nicole attack;\n'
 cit='citlali skill;\n'
 if v['citQ']:cit+='if !is_even(cycle) { citlali burst, attack:2, dash; }\n'
 if v['citN']:cit+='citlali attack;\n'
 xilo='xilonen skill, attack:2;\npick_up_crystallize("any");\n'
 s+=(cit+xilo if v['order'] else xilo+cit)
 s+='mavuika swap;\nif .mavuika.nightsoul.points { mavuika skill[recast=1];\n'
 s+='delay('+str(v['pre'])+');\nmavuika charge;\n}\nmavuika burst;\n'
 for k,c in enumerate(v['cs']):
  if k:
   s+='while !.mavuika.dash.ready { wait(1); }\n'
   s+='if .mavuika.nightsoul.points>15 || .mavuika.status.mavuika-burst { mavuika dash[charge_cancel=1,collision='+str(v['ds'][k-1])+']; } else { mavuika jump; }\n'
  s+='mavuika charge:'+str(c)+';\n'
  if v['hs'][k]:s+='mavuika charge[hold='+str(v['hs'][k])+'];\n'
  s+='mavuika charge[final=1];\n'
 return header+s+'}\n'
def score(v):
 key=json.dumps(v,sort_keys=True)
 if key in cache:return cache[key]
 i=len(rows);p=out/f'{i:05}.gcsl';p.write_text(render(v));cpu0=resource.getrusage(resource.RUSAGE_CHILDREN);w=time.monotonic()
 r=subprocess.run(['target/release/examples/replay',str(p),'8','25000000000000','150'],text=True,capture_output=True)
 cpu1=resource.getrusage(resource.RUSAGE_CHILDREN)
 row={'id':i,'parameters':copy.deepcopy(v),'cpu':cpu1.ru_utime+cpu1.ru_stime-cpu0.ru_utime-cpu0.ru_stime,'wall':time.monotonic()-w}
 if r.returncode:row['error']=r.stderr;val=-1
 else:
  d=json.loads(r.stdout);row.update(d['team']);val=d['team']['mean'];(out/f'{i:05}.json').write_text(r.stdout)
 rows.append(row);cache[key]=val
 return val
def mutate(v):
 v=copy.deepcopy(v);n=len(v['cs']);k=rng.randrange(13)
 if k<3:v['cs'][rng.randrange(n)]=rng.randrange(1,7)
 elif k<6:v['hs'][rng.randrange(n)]=rng.randrange(0,46)
 elif k==6 and n>1:v['ds'][rng.randrange(n-1)]^=1
 elif k==7:v['pre']=rng.randrange(0,31)
 elif k==8 and n<7:
  at=rng.randrange(n+1);v['cs'].insert(at,rng.randrange(1,5));v['hs'].insert(at,rng.choice([0,6,30,40]));v['ds'].insert(min(at,n-1),rng.randrange(2))
 elif k==9 and n>2:
  at=rng.randrange(n);v['cs'].pop(at);v['hs'].pop(at);v['ds'].pop(min(at,n-2))
 else:
  key=rng.choice(['holdE','order','nicN','citQ','citN']);v[key]^=1
 return v
base={'cs':[1]*6,'hs':[0,0,0,0,30,40],'ds':[1]*5,'holdE':1,'order':0,'nicN':1,'citQ':1,'citN':1,'pre':0}
best=copy.deepcopy(base);bestscore=score(base);print('base',bestscore,flush=True)
seeds=[base]
for cs in [[3,3,1,1],[3,3,3],[4,4,3],[5,5],[2,2,2,2],[1]*7]:
 for hide in [0,1]:
  for order in [0,1]:
   v=copy.deepcopy(base);v.update(cs=cs,hs=[6]*len(cs),ds=[hide]*(len(cs)-1),holdE=0,order=order);seeds.append(v)
for v in seeds:
 a=score(v)
 if a>bestscore:bestscore=a;best=copy.deepcopy(v);print('seed best',len(rows),round(a),best,flush=True)
beam=sorted([(r['mean'],r['parameters']) for r in rows if 'mean'in r],key=lambda r:r[0],reverse=True)[:12]
for i in range(1500):
 parent=beam[rng.randrange(len(beam))][1] if rng.random()<.92 else rng.choice(seeds)
 v=mutate(parent)
 if rng.random()<.15:v=mutate(v)
 a=score(v)
 if a>bestscore:bestscore=a;best=copy.deepcopy(v);print('best',len(rows),round(a),best,flush=True)
 if a>beam[-1][0] and v not in [x[1] for x in beam]:beam=sorted(beam+[(a,v)],key=lambda x:x[0],reverse=True)[:12]
 if i%100==0:(out/'summary.json').write_text(json.dumps(rows,indent=2))
(out/'summary.json').write_text(json.dumps(rows,indent=2));(out/'selected-parameters.json').write_text(json.dumps(best,indent=2));(out/'selected.gcsl').write_text(render(best))
print('FINAL',bestscore,best,flush=True)
