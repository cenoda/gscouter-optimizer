from pathlib import Path
import subprocess,json,resource,time
r=Path('runs/mavuika-replay');out=r/'audit';out.mkdir(exist_ok=True)
selected=(r/'video-refine/selected.gcsl').read_text()
fixed=(r/'panel-study/120-fixed90.gcsl').read_text().split('active mavuika;')[0].replace('swap_delay=12','swap_delay=1')
selected_fixed=fixed+selected[selected.index('fn mav_dc_cancel'):]
cases={
 'published-fixed90-450':(Path('runs/mavuika-external-research/export-stronger/candidate-0.gcsl').read_text(),150),
 'refined-fixed90-450':((r/'refine-v3/selected.gcsl').read_text(),150),
 'published-panel79-120':((r/'panel-study/120-panel-em79.28.gcsl').read_text(),0),
 'video-approx-panel79-120':((r/'video-approx-swap1.gcsl').read_text(),0),
 'video-refined-panel79-120':(selected,0),
 'video-refined-fixed90-120':(selected_fixed,0),
 'video-refined-panel158-120':(selected.replace('em=79.28;','em=158.56;'),0),
 'video-refined-panel266-120':(selected.replace('em=79.28;','em=266.28;'),0),
}
rows=[]
for name,(src,warmup) in cases.items():
 p=out/(name+'.gcsl');p.write_text(src);t0=resource.getrusage(resource.RUSAGE_CHILDREN);w=time.monotonic()
 proc=subprocess.run(['target/release/examples/replay',str(p),'2048','31000000000000',str(warmup)],capture_output=True,text=True,check=True)
 t1=resource.getrusage(resource.RUSAGE_CHILDREN);d=json.loads(proc.stdout);(out/(name+'.json')).write_text(proc.stdout)
 row={'name':name,'warmup':warmup,**d['team'],'cpu':t1.ru_utime+t1.ru_stime-t0.ru_utime-t0.ru_stime,'wall':time.monotonic()-w}
 print(row,flush=True);rows.append(row)
(out/'summary.json').write_text(json.dumps(rows,indent=2))
