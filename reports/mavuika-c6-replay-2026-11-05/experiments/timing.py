from pathlib import Path
import itertools,json,subprocess,time,resource
out=Path('runs/mavuika-replay/timing');out.mkdir(exist_ok=True)
header=Path('runs/mavuika-external-research/export-stronger/candidate-0.gcsl').read_text().split('active mavuika;')[0]
rows=[];start=time.monotonic()
patterns={'cf':'mavuika charge, charge[final=1];','ccf':'mavuika charge:2, charge[final=1];','c':'mavuika charge;','f':'mavuika charge[final=1];','cc':'mavuika charge:2;'}
for late,order,pre,pat,hide,n in itertools.product([0,1],[0,1],['','charge;','charge, charge[final=1];'],patterns,[0,1],range(5,11)):
 name=f'l{late}o{order}p{len(pre)}-{pat}h{hide}n{n}'
 r='while 1 {\n'
 if not late:r+='mavuika skill[hold=1];\n'
 r+='nicole skill;\n'
 if order:r+='citlali skill;\nxilonen skill, attack:2;\n'
 else:r+='xilonen skill, attack:2;\ncitlali skill;\n'
 r+='pick_up_crystallize("pyro");\n'
 if late:r+='mavuika skill[hold=1];\n'
 elif pre:r+='mavuika skill[recast=1];\n'
 if pre:r+='mavuika '+pre+'\n'
 r+='mavuika burst;\n'
 r+='for let j=0; j<'+str(n)+'; j=j+1 {\n'+patterns[pat]+'\n'
 r+='if j<'+str(n-1)+' && (.mavuika.nightsoul.points>15 || .mavuika.status.mavuika-burst) { mavuika dash[charge_cancel=1,collision='+str(1-hide)+']; }\n}\n}\n'
 p=out/(name+'.gcsl');p.write_text(header+'active mavuika;\n'+r)
 cpu0=resource.getrusage(resource.RUSAGE_CHILDREN);w=time.monotonic()
 proc=subprocess.run(['target/release/examples/replay',str(p),'8','23000000000000','150'],text=True,capture_output=True)
 cpu1=resource.getrusage(resource.RUSAGE_CHILDREN)
 row={'name':name,'wall':time.monotonic()-w,'cpu':cpu1.ru_utime+cpu1.ru_stime-cpu0.ru_utime-cpu0.ru_stime}
 if proc.returncode:row['error']=proc.stderr
 else:
  d=json.loads(proc.stdout);row.update(d['team']);(out/(name+'.json')).write_text(proc.stdout)
 rows.append(row)
 if len(rows)%60==0:print(len(rows),round(time.monotonic()-start,1),sorted([r for r in rows if 'mean'in r],key=lambda r:r['mean'],reverse=True)[:2],flush=True)
(out/'summary.json').write_text(json.dumps(rows,indent=2))
print('FINAL',sorted([r for r in rows if 'mean'in r],key=lambda r:r['mean'],reverse=True)[:10],flush=True)
