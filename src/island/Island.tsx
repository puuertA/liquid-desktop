import {useCallback,useEffect,useRef,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {islandState,timeLabel,nextNotice} from './state.mjs';
import {useIslandMotion,type IslandState} from './useIslandMotion';
import {defaultIsland,type IslandConfig,type IslandSnapshot} from './types';
import './island.css';
import {SurfaceGlass} from '../glass/SurfaceGlass';
import {CommitSlider} from './CommitSlider';
import {useGlassLight} from '../glass/useGlassLight';

function Glyph({name}:{name:'play'|'pause'|'previous'|'next'|'volume'|'timer'}){
  const paths={play:'M8 5l10 7-10 7z',pause:'M8 5v14M16 5v14',previous:'M5 5v14M19 5L7 12l12 7z',next:'M19 5v14M5 5l12 7-12 7z',volume:'M4 9h4l5-4v14l-5-4H4z M17 8q5 4 0 8',timer:'M9 2h6M12 8v5l3 2M12 5a8 8 0 1 0 0 16 8 8 0 0 0 0-16'};
  return <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name]}/></svg>;
}
export function Island(){
  const [config,setConfig]=useState<IslandConfig>(defaultIsland),[snapshot,setSnapshot]=useState<IslandSnapshot|null>(null),[selected,setSelected]=useState<string|null>(null);
  const [hover,setHover]=useState(false),[pinned,setPinned]=useState(false),[focused,setFocused]=useState(false),[now,setNow]=useState(Date.now()),[notice,setNotice]=useState({text:'',until:0,priority:0}),[error,setError]=useState(''),[busy,setBusy]=useState(false),[minutes,setMinutes]=useState(5);
  const previousTimer=useRef<number|null>(null),polling=useRef(false),collapse=useRef<ReturnType<typeof setTimeout>|null>(null),mounted=useRef(true);
  const notify=useCallback((text:string,priority=1)=>setNotice(previous=>nextNotice(previous,text,Date.now(),priority)),[]);
  const report=useCallback((message:string)=>setError(message),[]);
  const refresh=useCallback(async()=>{
    if(polling.current||document.hidden)return;polling.current=true;
    try{const value=await invoke<IslandSnapshot>('island_snapshot',{selected});if(!mounted.current)return;
      if(value.remaining===0&&previousTimer.current!==0)notify('Temporizador concluído',3);
      previousTimer.current=value.remaining;setSnapshot(value);
    }catch(e){if(mounted.current)setError(String(e))}finally{polling.current=false}
  },[selected,notify]);
  useEffect(()=>{mounted.current=true;let disposed=false;const off=listen<IslandConfig>('island-config',e=>setConfig(e.payload));
    void invoke<IslandConfig>('get_island_config').then(value=>{if(!disposed)setConfig(value)}).catch(e=>report(String(e)));
    return()=>{mounted.current=false;disposed=true;void off.then(f=>f());if(collapse.current)clearTimeout(collapse.current)};
  },[report]);
  useEffect(()=>{void refresh();const timer=setInterval(()=>void refresh(),2000);document.addEventListener('visibilitychange',refresh);return()=>{clearInterval(timer);document.removeEventListener('visibilitychange',refresh)}},[refresh]);
  useEffect(()=>{if(!notice.until)return;const timer=setTimeout(()=>setNow(Date.now()),Math.max(0,notice.until-Date.now()+10));return()=>clearTimeout(timer)},[notice]);
  const track=snapshot?.media?.track??null,timerActive=snapshot?.remaining!==null&&snapshot?.remaining!==undefined;
  const expanded=hover||pinned||focused;
  const state=islandState({expanded,notificationUntil:notice.until,now,media:!!track,timer:timerActive}) as IslandState;
  const root=useIslandMotion(state,config.motion,config.form,report);
  useGlassLight(root,config.motion);
  const [ready,setReady]=useState(false);
  useEffect(()=>{setReady(false);if(state!=='expanded')return;const timer=setTimeout(()=>setReady(true),config.motion&&!matchMedia('(prefers-reduced-motion: reduce)').matches?130:0);return()=>clearTimeout(timer)},[state,config.motion]);
  useEffect(()=>{const blur=()=>{setFocused(false);setHover(false)};const onKey=(e:KeyboardEvent)=>{if(e.key==='Escape'){setPinned(false);blur();(document.activeElement as HTMLElement)?.blur()}};window.addEventListener('keydown',onKey);window.addEventListener('blur',blur);return()=>{window.removeEventListener('keydown',onKey);window.removeEventListener('blur',blur)}},[]);
  async function action(run:()=>Promise<unknown>,message?:string){setBusy(true);try{await run();setError('');if(message)notify(message);await refresh()}catch(e){setError(String(e))}finally{setBusy(false)}}
  const media=(command:string,position?:number)=>track&&action(()=>invoke('island_media_action',{id:track.id,action:command,position:position??null}));
  return <div className="island-host">{(config.appearance==='liquid'||config.appearance==='adaptive')&&<SurfaceGlass selector=".island" staticWallpaper={config.static_wallpaper} onError={report}/>}<div ref={root} className={`island appearance-${config.appearance} state-${state}`} data-motion={config.motion} data-ready={ready} tabIndex={0} role="region" aria-label="Dynamic Island"
    onPointerEnter={()=>{if(collapse.current)clearTimeout(collapse.current);setHover(true)}}
    onPointerLeave={()=>{collapse.current=setTimeout(()=>setHover(false),180)}}
    onFocus={()=>setFocused(true)} onBlur={e=>{if(!e.currentTarget.contains(e.relatedTarget))setFocused(false)}}
    onKeyDown={e=>{if(e.target===e.currentTarget&&(e.key==='Enter'||e.key===' ')){e.preventDefault();setPinned(p=>!p)}}}
    onClick={e=>{if(!(e.target as HTMLElement).closest('button,input,select'))setPinned(p=>!p)}}>
    <div className="island-sheen" aria-hidden="true"/><div className="island-caustic" aria-hidden="true"/><div className="island-edge" aria-hidden="true"/>
    <div className="island-topline">
      {track?<img className="island-art" src={track.artwork??undefined} alt="" style={{visibility:track.artwork?'visible':'hidden'}}/>:<span className="island-symbol"><Glyph name={timerActive?'timer':'volume'}/></span>}
      <div className="island-title">{state==='notification'?<strong>{notice.text}</strong>:state==='idle'?<span className="idle-mark">Liquid</span>:<><strong>{track?.title||'Temporizador'}</strong>{state==='expanded'&&<small>{track?.artist||'Um momento de foco'}</small>}</>}</div>
      {timerActive&&<span className="timer-chip" title="Tempo restante">{snapshot!.remaining===0?'Concluído':timeLabel(snapshot!.remaining)}</span>}
      {track&&state!=='expanded'&&state!=='notification'&&<button aria-label={track.playing?'Pausar':'Reproduzir'} disabled={busy||!track.toggle} onClick={()=>void media('toggle')}><Glyph name={track.playing?'pause':'play'}/></button>}
    </div>
    <div className="island-expanded" inert={!ready} aria-hidden={!ready}>
      {notice.until>Date.now()&&<div className="island-notice" role="status">{notice.text}</div>}
      {track?<>
        <div className="island-progress"><CommitSlider label="Posição da música" min={track.start} max={Math.max(track.start+1,track.end)} value={track.position} disabled={busy||!track.seek} onCommit={position=>void media('seek',position)}/><span>{timeLabel(track.position-track.start)}</span><span>{timeLabel(track.end-track.start)}</span></div>
        <div className="island-playback"><button aria-label="Anterior" disabled={busy||!track.previous} onClick={()=>void media('previous')}><Glyph name="previous"/></button><button className="main-play" aria-label={track.playing?'Pausar':'Reproduzir'} disabled={busy||!track.toggle} onClick={()=>void media('toggle')}><Glyph name={track.playing?'pause':'play'}/></button><button aria-label="Próxima" disabled={busy||!track.next} onClick={()=>void media('next')}><Glyph name="next"/></button><select aria-label="Sessão multimídia" value={track.id} onChange={e=>setSelected(e.target.value)}>{snapshot?.media?.sessions.map(s=><option key={s.id} value={s.id}>{s.app}</option>)}</select></div>
      </>:<p className="island-empty">{snapshot?.media_error?'Mídia indisponível neste momento.':'Abra uma música em um player compatível.'}</p>}
      <div className="island-volume"><button aria-label={snapshot?.volume?.muted?'Ativar áudio':'Silenciar'} disabled={busy||!snapshot?.volume} onClick={()=>void action(()=>invoke('island_volume',{level:null,muted:!snapshot?.volume?.muted}))}><Glyph name="volume"/></button><CommitSlider label="Volume do sistema" min={0} max={100} value={Math.round((snapshot?.volume?.level??0)*100)} disabled={busy||!snapshot?.volume} onCommit={value=>void action(()=>invoke('island_volume',{level:value/100,muted:null}),`Volume ${value}%`)}/><span>{snapshot?.volume?(snapshot.volume.muted?'Mudo':`${Math.round(snapshot.volume.level*100)}%`):'—'}</span></div>
      <div className="island-timer"><Glyph name="timer"/>{timerActive?<><span>{snapshot!.remaining===0?'Tempo concluído':timeLabel(snapshot!.remaining)}</span><button disabled={busy} onClick={()=>void action(()=>invoke('island_timer',{seconds:null}))}>Encerrar</button></>:<><input aria-label="Duração em minutos" type="number" min="1" max="1440" value={minutes} onChange={e=>setMinutes(+e.target.value)}/><span>min</span><button disabled={busy||!Number.isInteger(minutes)||minutes<1||minutes>1440} onClick={()=>void action(()=>invoke('island_timer',{seconds:minutes*60}),'Temporizador iniciado')}>Iniciar</button></>}<button className="pin-control" aria-pressed={pinned} onClick={()=>{setPinned(p=>!p);(document.activeElement as HTMLElement)?.blur()}}>{pinned?'Soltar':'Fixar'}</button></div>
      {(error||snapshot?.volume_error)&&<p className="island-error" role="alert">{error||'Saída de áudio indisponível'}</p>}
      <small className="island-hint">Esc recolhe · Liquid Desktop</small>
    </div>
  </div></div>;
}
