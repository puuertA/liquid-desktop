import {useCallback,useEffect,useRef,useState,type CSSProperties} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {CommitSlider} from '../island/CommitSlider';
import type {TopBarSnapshot} from './types';
import {rate,percent,memoryPercent,tintColor} from './format.mjs';
import './topbar.css';
import {SurfaceGlass} from '../glass/SurfaceGlass';

function Icon({audio=false,muted=false}:{audio?:boolean;muted?:boolean}){
  return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={audio?`M3 9h4l5-4v14l-5-4H3z ${muted?'M16 9l5 6M21 9l-5 6':'M16 8q5 4 0 8'}`:'M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z'}/></svg>;
}
export function TopBar(){
  const [snapshot,setSnapshot]=useState<TopBarSnapshot|null>(null),[now,setNow]=useState(new Date()),[error,setError]=useState(''),[busy,setBusy]=useState(false);
  const mounted=useRef(false);
  const report=useCallback((message:string)=>setError(message),[]);
  useEffect(()=>{
    mounted.current=true;let disposed=false,pending=false;
    async function refresh(){
      if(disposed||pending||document.hidden)return;pending=true;
      try{const value=await invoke<TopBarSnapshot>('topbar_snapshot');if(!disposed){setSnapshot(value);setError('')}}
      catch(e){if(!disposed)setError(String(e))}finally{pending=false}
    }
    const off=listen('topbar-refresh',()=>void refresh());
    void refresh();const timer=setInterval(()=>void refresh(),2000);
    const clock=setInterval(()=>{if(!document.hidden)setNow(new Date())},1000);
    const visible=()=>{setNow(new Date());void refresh()};document.addEventListener('visibilitychange',visible);
    return()=>{disposed=true;mounted.current=false;clearInterval(timer);clearInterval(clock);document.removeEventListener('visibilitychange',visible);void off.then(f=>f())};
  },[]);
  async function volume(level:number|null,muted:boolean|null){
    setBusy(true);
    try{const next=await invoke<NonNullable<TopBarSnapshot['volume']>>('topbar_volume',{level,muted});if(mounted.current){setSnapshot(s=>s?{...s,volume:next,volume_error:null}:s);setError('')}}
    catch(e){if(mounted.current)setError(String(e))}finally{if(mounted.current)setBusy(false)}
  }
  const open=()=>void invoke('topbar_open_panel').catch(e=>setError(String(e)));
  if(!snapshot)return <div className="bar-loading"><button title={error||'Carregando Top Bar'} onClick={open} aria-label="Abrir painel"><Icon/></button></div>;
  const {config:c,system,volume:audio,shared_island:shared}=snapshot;
  const time=now.toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit',...(c.show_seconds?{second:'2-digit'}:{})});
  const status=error||snapshot.volume_error;
  return <div className="topbar" data-shared={shared} data-optical={true} style={{'--bar-tint':tintColor(c.tint,c.opacity),'--bar-gap':shared?'480px':'0px'} as CSSProperties}>
    <SurfaceGlass selector={shared?'.bar-segment':'.topbar'} staticWallpaper={c.static_wallpaper} tint={c.tint} opacity={c.opacity*.35} onError={report}/><div className="bar-segment bar-left"><div className="bar-items">
      <button className="bar-launcher" onClick={open} title="Abrir painel Liquid Desktop" aria-label="Abrir painel Liquid Desktop"><Icon/><span>Liquid Desktop</span></button>
      <time dateTime={now.toISOString()} title={now.toLocaleString('pt-BR')}><span className="bar-date">{c.show_date&&now.toLocaleDateString('pt-BR',{weekday:'short',day:'2-digit',month:'short'})}</span><strong>{time}</strong></time>
    </div></div>
    <div className="bar-gap" aria-hidden="true"/>
    <div className="bar-segment bar-right"><div className="bar-items">
      {c.show_cpu&&<span className="bar-cpu bar-metric" title="Uso total da CPU"><small>CPU</small>{percent(system?.cpu)}</span>}
      {c.show_memory&&<span className="bar-memory bar-metric" title={system?`${(system.memory_used/1073741824).toFixed(1)} / ${(system.memory_total/1073741824).toFixed(1)} GiB de RAM`:'Aguardando leitura de RAM'}><small>RAM</small>{memoryPercent(system)}</span>}
      {c.show_network&&<span className="bar-network bar-metric" title={`Download / upload · soma de ${system?.interfaces??0} interfaces; pode incluir VPN`}><span>↓ {rate(system?.download)}</span><span>↑ {rate(system?.upload)}</span></span>}
      {c.show_volume&&<div className="bar-audio"><button onClick={()=>void invoke('toggle_center').catch(e=>setError(String(e)))} aria-label="Abrir Control Center" title={audio?`Volume ${Math.round(audio.level*100)}%${audio.muted?' · mudo':''}`:'Áudio indisponível'}><Icon audio muted={audio?.muted}/></button><div className="bar-volume-slider"><CommitSlider label="Volume do sistema" min={0} max={100} value={Math.round((audio?.level??0)*100)} disabled={busy||!audio} onCommit={value=>void volume(value/100,null)}/></div><span className="bar-volume-value">{audio?(audio.muted?'Mudo':percent(audio.level*100)):'—'}</span></div>}
      {status&&<button className="bar-error" title={status} aria-label={`Abrir painel: ${status}`} onClick={open}>!</button>}
    </div></div>
  </div>;
}
