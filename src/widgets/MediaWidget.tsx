import {useEffect,useRef,useState,type CSSProperties} from 'react';
import {invoke} from '@tauri-apps/api/core';
import gsap from 'gsap';
import type {Settings} from '../types';
import type {IslandSnapshot,Track} from '../island/types';

const clock=(seconds:number)=>{const value=Math.max(0,Math.floor(seconds));return `${Math.floor(value/60)}:${String(value%60).padStart(2,'0')}`};
export function MediaWidget({settings}:{settings:Settings}){
  const [track,setTrack]=useState<Track|null>(null),[error,setError]=useState(''),[busy,setBusy]=useState(false);
  const art=useRef<HTMLDivElement>(null),info=useRef<HTMLDivElement>(null);
  useEffect(()=>{if(!settings.motion||matchMedia('(prefers-reduced-motion: reduce)').matches)return;const targets=[art.current,info.current].filter(Boolean);if(!targets.length)return;const tween=gsap.fromTo(targets,{opacity:.3,y:6},{opacity:1,y:0,duration:.42,ease:'power2.out',stagger:.05});return()=>{tween.kill()}},[track?.id,track?.title,settings.motion]);
  const [accent,setAccent]=useState('#d1eee7');
  const [peak,setPeak]=useState<number|null>(null);
  const [seekDraft,setSeekDraft]=useState<number|null>(null);
  useEffect(()=>{if(!settings.media_visualizer)return;let disposed=false,busy=false;async function read(){if(disposed||busy||document.hidden)return;busy=true;try{const value=await invoke<number>('media_peak');if(!disposed)setPeak(value)}catch{if(!disposed)setPeak(null)}finally{busy=false}}void read();const timer=setInterval(read,250);return()=>{disposed=true;clearInterval(timer)}},[settings.media_visualizer]);
  function sampleCover(image:HTMLImageElement){try{const canvas=document.createElement('canvas');canvas.width=canvas.height=16;const ctx=canvas.getContext('2d');if(!ctx)return;ctx.drawImage(image,0,0,16,16);const pixels=ctx.getImageData(0,0,16,16).data;let r=0,g=0,b=0,n=0;for(let i=0;i<pixels.length;i+=4){if(pixels[i+3]<128)continue;r+=pixels[i];g+=pixels[i+1];b+=pixels[i+2];n++}if(n)setAccent(`rgb(${Math.round(r/n)},${Math.round(g/n)},${Math.round(b/n)})`)}catch{setAccent('#d1eee7')}}
  useEffect(()=>{let disposed=false,flight=false;async function refresh(){if(flight||document.hidden)return;flight=true;try{const result=await invoke<IslandSnapshot>('island_snapshot',{selected:null});if(!disposed){setTrack(result.media?.track??null);setError(result.media_error??'')}}catch(e){if(!disposed)setError(String(e))}finally{flight=false}}
    void refresh();const timer=setInterval(refresh,1000);return()=>{disposed=true;clearInterval(timer)};
  },[]);
  async function action(name:string,position?:number){if(!track||busy)return;setBusy(true);try{await invoke('island_media_action',{id:track.id,action:name,position:position??null});setError('')}catch(e){setError(String(e))}finally{setSeekDraft(null);setBusy(false)}}
  const duration=track?Math.max(0,track.end-track.start):0,elapsed=track?Math.max(0,track.position-track.start):0;
  return <div className={`media-widget media-${settings.media_layout}`} style={{'--media-accent':accent} as CSSProperties}>
    <div className="widget-heading"><h2>Mídia</h2><span className="live-label">{track?.app||'Windows'}</span></div>
    {track?<><div className="media-main"><div className="media-art" ref={art}>{track.artwork?<img key={track.artwork.slice(0,64)} src={track.artwork} alt="Capa do álbum" onLoad={e=>sampleCover(e.currentTarget)}/>:<span aria-hidden="true">♫</span>}</div><div className="media-info" ref={info}><strong title={track.title}>{track.title||'Faixa sem título'}</strong><span>{track.artist||'Artista desconhecido'}</span></div></div>
      <div className="media-timeline"><input aria-label="Posição da faixa" type="range" min={0} max={duration||1} step={1} value={seekDraft??Math.min(elapsed,duration||1)} disabled={!track.seek||!duration||busy} onChange={e=>setSeekDraft(Number(e.target.value))} onPointerUp={()=>{if(seekDraft!=null)void action('seek',track.start+seekDraft)}} onKeyUp={e=>{if(seekDraft!=null&&['ArrowLeft','ArrowRight','Home','End'].includes(e.key))void action('seek',track.start+seekDraft)}}/><div><span>{clock(seekDraft??elapsed)}</span><span>{clock(duration)}</span></div></div>
      <div className="media-controls"><button aria-label="Faixa anterior" disabled={!track.previous||busy} onClick={()=>void action('previous')}>‹‹</button><button className="media-play" aria-label={track.playing?'Pausar':'Reproduzir'} disabled={!track.toggle||busy} onClick={()=>void action('toggle')}>{track.playing?'Ⅱ':'▶'}</button><button aria-label="Próxima faixa" disabled={!track.next||busy} onClick={()=>void action('next')}>››</button></div>
      {settings.media_visualizer&&<div className="media-meter" role="meter" aria-label="Nível real da saída de áudio" aria-valuemin={0} aria-valuemax={100} aria-valuenow={peak==null?undefined:Math.round(peak*100)}><span style={{width:peak==null?'0%':`${Math.round(peak*100)}%`}}/></div>}
    </>:<p className="widget-empty">{error||'Nenhuma sessão de mídia ativa'}</p>}
    {track&&error&&<small role="status">{error}</small>}
  </div>;
}
