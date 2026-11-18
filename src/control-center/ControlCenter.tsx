import {useEffect,useRef,useState} from 'react';import {invoke} from '@tauri-apps/api/core';import {listen} from '@tauri-apps/api/event';import {CommitSlider} from '../island/CommitSlider';import {useGlassLight} from '../glass/useGlassLight';import './center.css';
type Snapshot={devices:{id:string;name:string;is_default:boolean}[];selected:string|null;volume:{level:number;muted:boolean}|null;audio_error:string|null;power:{ac:boolean|null;percent:number|null;present:boolean|null;charging:boolean;saver:boolean}|null;network:string};
export function ControlCenter(){
 const root=useRef<HTMLDivElement>(null);useGlassLight(root,true);
 const [snapshot,setSnapshot]=useState<Snapshot|null>(null),[selected,setSelected]=useState<string|null>(null),[busy,setBusy]=useState(false),[error,setError]=useState(''),[opening,setOpening]=useState(0);
 const hide=()=>void invoke('center_hide').catch(e=>setError(String(e)));
 useEffect(()=>{let disposed=false,pending=false;async function refresh(){if(pending||disposed||document.hidden)return;pending=true;try{const next=await invoke<Snapshot>('center_snapshot',{selected});if(!disposed)setSnapshot(next)}catch(e){if(!disposed)setError(String(e))}finally{pending=false}}
  void refresh();const timer=setInterval(()=>void refresh(),2000),off=listen('center-open',()=>{setOpening(n=>n+1);void refresh()});const visible=()=>void refresh(),key=(e:KeyboardEvent)=>{if(e.key==='Escape')hide()};document.addEventListener('visibilitychange',visible);window.addEventListener('keydown',key);return()=>{disposed=true;clearInterval(timer);void off.then(f=>f());document.removeEventListener('visibilitychange',visible);window.removeEventListener('keydown',key)};
 },[selected]);
 async function volume(level:number|null,muted:boolean|null){if(!snapshot?.selected)return;setBusy(true);try{const next=await invoke<NonNullable<Snapshot['volume']>>('center_volume',{id:snapshot.selected,level,muted});setSnapshot(s=>s?{...s,volume:next}:s);setError('')}catch(e){setError(String(e))}finally{setBusy(false)}}
 const settings=(page:string)=>void invoke('center_settings',{page}).catch(e=>setError(String(e)));
 const power=snapshot?.power,volumeValue=snapshot?.volume;
 return <div className="control-center" ref={root} role="dialog" aria-label="Control Center"><div className="center-content" key={opening}>
 <header><div><small>Liquid Desktop</small><h1>Control Center</h1></div><button onClick={hide} aria-label="Fechar">×</button></header>
 <section className="center-audio"><div className="center-label"><h2>Áudio</h2><span>{volumeValue?`${Math.round(volumeValue.level*100)}%`:'—'}</span></div>
 <label className="center-output">Ajustar dispositivo<select disabled={busy||!snapshot?.devices.length} value={snapshot?.selected??''} onChange={e=>{setSelected(e.target.value);setSnapshot(s=>s?{...s,selected:e.target.value,volume:null}:s)}}>{!snapshot?.devices.length&&<option value="">Nenhuma saída disponível</option>}{snapshot?.devices.map(d=><option key={d.id} value={d.id}>{d.name}{d.is_default?' · padrão':''}</option>)}</select></label>
 <div className="center-volume"><button disabled={busy||!volumeValue} onClick={()=>void volume(null,!volumeValue?.muted)} aria-pressed={volumeValue?.muted??false}>{volumeValue?.muted?'Ativar':'Silenciar'}</button><CommitSlider label="Volume do dispositivo" min={0} max={100} value={Math.round((volumeValue?.level??0)*100)} disabled={busy||!volumeValue} onCommit={v=>void volume(v/100,null)}/></div>
 <button className="center-link" onClick={()=>settings('sound')}>Escolher saída padrão no Windows ↗</button></section>
 <div className="center-status"><button onClick={()=>settings('network')}><span className="center-status-icon">↗</span><span><strong>Rede</strong><small>{snapshot?.network??'Consultando o Windows…'}</small></span></button>
 <button onClick={()=>settings('power')}><span className="center-status-icon">◴</span><span><strong>{power?.present===false?'Energia':power?.percent!=null?`Bateria · ${power.percent}%`:'Bateria'}</strong><small>{!power?'Estado indisponível':power.present===null?'Estado não informado':power.present===false?'Sem bateria':power.charging?'Carregando':power.ac?'Conectado à energia':power.saver?'Economia de bateria':'Usando bateria'}</small></span></button></div>
 {(error||snapshot?.audio_error)&&<p className="center-error" role="alert">{error||snapshot?.audio_error}</p>}
 <footer>Esc ou clique fora para fechar</footer></div></div>;
}
