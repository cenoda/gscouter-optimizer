from pathlib import Path
import subprocess,json,time,itertools
root=Path('runs/mavuika-replay'); out=root/'ablations';out.mkdir(exist_ok=True)
header=Path('runs/mavuika-external-research/export-stronger/candidate-0.gcsl').read_text().split('active mavuika;')[0]
rotation=Path('runs/mavuika-external-research/external-stronger-rotation.gcsl').read_text()
rows=[]
for late,reorder,final,hidden in itertools.product([False,True],repeat=4):
    name=''.join(str(int(x)) for x in (late,reorder,final,hidden))
    r=rotation
    if late:
        r=r.replace('  mavuika skill[hold=1];\n','')
        r=r.replace('    mavuika swap;','    mavuika swap;\n    mavuika skill[hold=1];')
        r=r.replace('      mavuika skill[recast=1];','')
    if reorder:
        r=r.replace('  xilo skill, attack:2;\n  pick_up_crystallize("any");\n','')
        r=r.replace('    mavuika swap;', '    xilo skill, attack:2;\n    pick_up_crystallize("any");\n    mavuika swap;')
    if final:
        r=r.replace('      mavuika charge;', '      mavuika charge, charge[final=1];')
    if hidden:
        r=r.replace('charge_cancel=1, travel=d_travel_f','charge_cancel=1, collision=0')
    r=r.replace('active mavuika;','')
    config=header+'active mavuika;\nwhile 1 {\n'+r+'\n}\n'
    path=out/(name+'.gcsl');path.write_text(config)
    t=time.process_time(); wall=time.monotonic()
    p=subprocess.run(['target/release/examples/replay',str(path),'64','22000000000000','150'],text=True,capture_output=True)
    if p.returncode:
        row={'name':name,'error':p.stderr,'wall':time.monotonic()-wall}
    else:
        d=json.loads(p.stdout);(out/(name+'.json')).write_text(p.stdout)
        row={'name':name,'mean':d['team']['mean'],'se':d['team']['se'],'wall':time.monotonic()-wall}
    rows.append(row);print(json.dumps(row),flush=True)
    (out/'summary.json').write_text(json.dumps(rows,indent=2))
