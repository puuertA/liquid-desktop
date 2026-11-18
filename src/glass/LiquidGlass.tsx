import {useEffect,useRef,useMemo,useId,useState, type ReactNode, type CSSProperties} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {getCurrentWindow} from '@tauri-apps/api/window';
import type {Settings,Wallpaper} from '../types';
import {imageRect,union} from './geometry.mjs';
import {adaptiveShade,relativeLuminance} from './optics.mjs';
import {lensMap} from './maps';
import {pressSurface,stopSurface} from './motion';
import './glass.css';
import {useGlassLight} from './useGlassLight';

type Props={settings:Settings;wallpaper:Wallpaper|null;children:ReactNode;onError:(s:string)=>void;previewRect?:()=>[number,number,number,number]};
export function LiquidGlass({settings:s,wallpaper,children,onError,previewRect}:Props) {
  const canvas=useRef<HTMLCanvasElement>(null),root=useRef<HTMLDivElement>(null);
  useGlassLight(root,s.motion);
  const id=`lens-${useId().replace(/:/g,'')}`;
  const [luminance,setLuminance]=useState<number|null>(null);
  const cap=Math.ceil(Math.max(s.width,s.height)*window.devicePixelRatio*(s.quality==='high'?1.5:1));
  const map=useMemo(()=>s.static_wallpaper?lensMap({width:s.width,height:s.height,radius:32,bezel:s.bezel,thickness:s.thickness,cap}):null,[s.static_wallpaper,s.width,s.height,s.bezel,s.thickness,cap]);
  useEffect(()=>{
    const element=root.current;if(!element)return;
    const query=matchMedia('(prefers-reduced-motion: reduce)');
    const reset=()=>stopSurface(element);
    query.addEventListener('change',reset);window.addEventListener('blur',reset);
    return()=>{query.removeEventListener('change',reset);window.removeEventListener('blur',reset);reset()};
  },[]);
  useEffect(()=>{
    setLuminance(null);
    if(!s.static_wallpaper||!wallpaper)return;
    let disposed=false,busy=false,dirty=false,frame=0;
    const images=new Map<string,HTMLImageElement>();
    const load=Promise.all(wallpaper.monitors.map(m=>new Promise<void>(resolve=>{
      if(m.error)onError(m.error);
      if(!m.image)return resolve();
      const img=new Image();img.onload=()=>{images.set(m.id,img);resolve()};
      img.onerror=()=>{onError('Formato de wallpaper não suportado pelo WebView2');resolve()};img.src=m.image;
    })));
    const sample=document.createElement('canvas');sample.width=32;sample.height=32;
    async function draw(){
      frame=0;if(disposed||document.hidden)return;if(busy){dirty=true;return}busy=true;dirty=false;
      try{
        await load;if(disposed)return;
        const r=previewRect?previewRect():await invoke<[number,number,number,number]>('widget_rect');if(disposed)return;
        const c=canvas.current;if(!c)return;const ctx=c.getContext('2d');if(!ctx)throw Error('Canvas 2D indisponível');
        const density=Math.min(1,cap*2/Math.max(r[2],r[3]));
        c.width=Math.max(1,Math.round(r[2]*density));c.height=Math.max(1,Math.round(r[3]*density));
        ctx.scale(c.width/r[2],c.height/r[3]);
        const color=wallpaper!.color;ctx.fillStyle=`rgb(${color&255},${(color>>8)&255},${(color>>16)&255})`;ctx.fillRect(0,0,r[2],r[3]);
        const bounds=union(wallpaper!.monitors.map(m=>m.rect));
        for(const m of wallpaper!.monitors){const img=images.get(m.id);if(!img)continue;
          ctx.save();ctx.beginPath();ctx.rect(m.rect[0]-r[0],m.rect[1]-r[1],m.rect[2],m.rect[3]);ctx.clip();
          if(wallpaper!.mode===1){const pattern=ctx.createPattern(img,'repeat');if(pattern){ctx.translate(bounds[0]-r[0],bounds[1]-r[1]);ctx.fillStyle=pattern;ctx.fillRect(r[0]-bounds[0],r[1]-bounds[1],r[2],r[3])}}
          else{const [x,y,w,h]=imageRect(wallpaper!.mode,img.naturalWidth,img.naturalHeight,wallpaper!.mode===5?bounds:m.rect);ctx.drawImage(img,x-r[0],y-r[1],w,h)}ctx.restore();
        }
        if(s.material==='adaptive'){
          const context=sample.getContext('2d',{willReadFrequently:true})!;context.drawImage(c,0,0,32,32);
          const pixels=context.getImageData(0,0,32,32).data,levels=[];
          for(let i=0;i<pixels.length;i+=4)levels.push(relativeLuminance(pixels[i],pixels[i+1],pixels[i+2]));
          levels.sort((a,b)=>a-b);setLuminance(levels[Math.floor(levels.length*.85)]);
        }
      }catch(e){if(!disposed)onError(String(e))}finally{busy=false;if(dirty&&!disposed)schedule()}
    }
    function schedule(){if(!frame&&!disposed)frame=requestAnimationFrame(()=>void draw())}
    const off=previewRect?null:listen('surface-moved',schedule);
    const moved=previewRect?null:getCurrentWindow().onMoved(schedule);
    const dpi=previewRect?null:getCurrentWindow().onScaleChanged(schedule);
    const observer=new ResizeObserver(schedule);if(root.current)observer.observe(root.current);
    document.addEventListener('visibilitychange',schedule);schedule();
    return()=>{disposed=true;cancelAnimationFrame(frame);observer.disconnect();document.removeEventListener('visibilitychange',schedule);for(const subscription of [off,moved,dpi])void subscription?.then(f=>f()).catch(()=>{})};
  },[wallpaper,s.static_wallpaper,s.x,s.y,s.width,s.height,s.material,cap,onError,previewRect]);
  const shade=s.material==='adaptive'&&luminance!==null?adaptiveShade(luminance,s.shade):s.shade;
  const scale=s.refraction*2, dispersion=s.quality==='economy'?0:s.dispersion;
  return <div className="glass glass-v2" data-material={s.material} data-optics={s.static_wallpaper?'texture':'translucent'} style={{'--glass-tint':s.tint,'--glass-shine':s.shine,'--glass-shade':shade,'--glass-thickness':`${s.thickness}px`} as CSSProperties} ref={root}
    onPointerDown={e=>{if(e.button===0&&!(e.target as HTMLElement).closest('button,input,textarea,select,[data-no-drag]'))pressSurface(e.currentTarget,true,s.motion)}}
    onPointerUp={e=>pressSurface(e.currentTarget,false,s.motion)} onPointerCancel={e=>stopSurface(e.currentTarget)} onPointerLeave={e=>pressSurface(e.currentTarget,false,s.motion)}>
    {map&&<svg className="filter-defs" aria-hidden="true"><defs><filter id={id} x="0" y="0" width="100%" height="100%" colorInterpolationFilters="sRGB">
      <feGaussianBlur in="SourceGraphic" stdDeviation={s.frost} edgeMode="duplicate" result="texture"/>
      <feImage href={map} result="normal" preserveAspectRatio="none" width="100%" height="100%"/>
      {dispersion>0?<>
        <feDisplacementMap in="texture" in2="normal" scale={scale*(1-dispersion*.7)} xChannelSelector="R" yChannelSelector="G" result="redWarp"/>
        <feColorMatrix in="redWarp" type="matrix" values="1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0" result="red"/>
        <feDisplacementMap in="texture" in2="normal" scale={scale} xChannelSelector="R" yChannelSelector="G" result="greenWarp"/>
        <feColorMatrix in="greenWarp" type="matrix" values="0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0" result="green"/>
        <feDisplacementMap in="texture" in2="normal" scale={scale*(1+dispersion*.7)} xChannelSelector="R" yChannelSelector="G" result="blueWarp"/>
        <feColorMatrix in="blueWarp" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0" result="blue"/>
        <feBlend in="red" in2="green" mode="screen" result="rg"/><feBlend in="rg" in2="blue" mode="screen"/>
      </>:<feDisplacementMap in="texture" in2="normal" scale={scale} xChannelSelector="R" yChannelSelector="G"/>}
    </filter></defs></svg>}
    {s.static_wallpaper&&<canvas ref={canvas} className="wallpaper-texture" style={{filter:`url(#${id})`}}/>}
    <div className="glass-tint" style={{background:s.tint,opacity:s.opacity}}/><div className="glass-readability"/><div className="glass-light"/><div className="glass-rim"/><div className="glass-caustic"/><div className="glass-content">{children}</div>
  </div>;
}
