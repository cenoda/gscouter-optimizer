from pathlib import Path
import json,subprocess,re,resource,time
out=Path('runs/mavuika-replay/panel-study');out.mkdir(exist_ok=True)
source=Path('runs/mavuika-external-research/export-stronger/candidate-0.gcsl').read_text()
rows=[]
for duration in [450,120]:
 for mode in ['fixed90','fixed100','panel-em0','panel-em79.28','panel-em158.56','panel-em266.28']:
  s=source
  warmup=150 if duration==450 else 0
  s=s.replace('duration=450',f'duration={duration}')
  if mode!='fixed90':s=s.replace('char lvl=90/90','char lvl=100/100')
  if mode.startswith('panel'):
   em=float(mode.split('em')[1]);s='\n'.join(x for x in s.splitlines() if not x.startswith('mavuika add stats'))
   insert='mavuika add stats hp=4780 atk=311 atk%='+str((2861-311)/1180.0654244-1-.25)+' cr='+str(.59-.160256)+' cd='+str(2.52-.884)+' pyro%=0.466 em='+str(em)+';\n'
   s=s.replace('active mavuika;',insert+'active mavuika;')
  if duration==120:
   s=s.replace('target lvl=100','target lvl=110')
   # The video starts with empty energy indicators. No periodic energy injection.
   s=re.sub(r'energy every .*?;\n','',s)
   s=re.sub(r'(char lvl=\S+ cons=6 talent=10,10,10);',r'\1 +params=[start_energy=0];',s)
  name=f'{duration}-{mode}';p=out/(name+'.gcsl');p.write_text(s)
  t0=resource.getrusage(resource.RUSAGE_CHILDREN);w=time.monotonic()
  r=subprocess.run(['target/release/examples/replay',str(p),'256','27000000000000',str(warmup)],text=True,capture_output=True)
  t1=resource.getrusage(resource.RUSAGE_CHILDREN);row={'name':name,'cpu':t1.ru_utime+t1.ru_stime-t0.ru_utime-t0.ru_stime,'wall':time.monotonic()-w}
  if r.returncode:row['error']=r.stderr
  else:
   d=json.loads(r.stdout);row.update(d['team']);(out/(name+'.json')).write_text(r.stdout)
  print(row,flush=True);rows.append(row)
(out/'summary.json').write_text(json.dumps(rows,indent=2))
