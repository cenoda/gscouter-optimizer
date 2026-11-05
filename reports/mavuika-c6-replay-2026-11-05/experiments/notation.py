from pathlib import Path
import json,subprocess,re
out=Path('runs/mavuika-replay/notation');out.mkdir(exist_ok=True)
main='RCQzsZhzzPszPszPszZsZhzcFdcFecdcdc'
tail='CdcFQZhzzPszPszZszhZ'
# Capitals describe the author's intended Melt hits, not different actions.
def actions(seq):
 s=''
 for c in seq:
  if c=='R':s+='if .mavuika.nightsoul.points { mavuika skill[recast=1]; } else { mavuika skill[hold=1]; }\n'
  elif c in 'CczZ':s+='mavuika charge;\n'
  elif c in 'PF':s+='mavuika charge[final=1];\n'
  elif c=='Q':s+='mavuika burst;\n'
  elif c=='e':s+='mavuika skill;\n'
  elif c in 'sdh':
   s+='while !.mavuika.dash.ready { wait(1); }\n'
   s+='if .mavuika.nightsoul.points>15 || .mavuika.status.mavuika-burst { mavuika dash[charge_cancel=1, collision='+str(0 if c=='h' else 1)+']; } else { mavuika jump; }\n'
  else:raise ValueError(c)
 return s
for em in [79.28,266.28]:
 src=Path(f'runs/mavuika-replay/panel-study/120-panel-em{em}.gcsl').read_text();header=src.split('active mavuika;')[0]
 for swap in [1,12]:
  for end in [False,True]:
   name=f'em{em}-swap{swap}-tail{int(end)}'
   s=header.replace('swap_delay=12',f'swap_delay={swap}')+'active mavuika;\nmavuika skill;\nwhile 1 {\nnicole skill;\ncitlali skill;\nxilonen skill, attack:2;\npick_up_crystallize("any");\nmavuika swap;\n'
   if end:s+='if f()>6480 {\nif .mavuika.nightsoul.points {mavuika skill[recast=1];}\n'+actions(tail)+'\n} else {\n'+actions(main)+'}\n'
   else:s+=actions(main)
   s+='}\n';p=out/(name+'.gcsl');p.write_text(s)
   r=subprocess.run(['target/release/examples/replay',str(p),'128','32000000000000','0'],capture_output=True,text=True)
   if r.returncode:print(name,r.stderr,flush=True)
   else:
    d=json.loads(r.stdout);(out/(name+'.json')).write_text(r.stdout);print(name,d['team'],flush=True)
