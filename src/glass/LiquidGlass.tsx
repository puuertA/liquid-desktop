import {useEffect,useRef,useMemo, type ReactNode, type CSSProperties} from 'react';
import gsap from 'gsap';
import {invoke} from '@tauri-apps/api/core';
import type {Settings,Wallpaper} from '../types';
import {imageRect,union} from './geometry.mjs';

function displacement(width:number,height:number) {
  const c=document.createElement('canvas');c.width=width;c.height=height;
  const ctx=c.getContext('2d')!;const data=ctx.createImageData(width,height);
  for(let y=0;y<height;y++)for(let x=0;x<width;x++){
    const qx=Math.abs(x-width/2)-(width/2-32),qy=Math.abs(y-height/2)-(height/2-32);
    const ax=Math.max(qx,0),ay=Math.max(qy,0),len=Math.hypot(ax,ay);
    const distance=len+Math.min(Math.max(qx,qy),0)-32;
    const strength=Math.pow(Math.max(0,1-Math.abs(distance)/26),2);
    const nx=len?ax/len:(qx>qy?1:0),ny=len?ay/len:(qy>=qx?1:0),i=(y*width+x)*4;
    data.data[i]=128+Math.sign(x-width/2)*nx*strength*110;
    data.data[i+1]=128+Math.sign(y-height/2)*ny*strength*110;data.data[i+2]=128;data.data[i+3]=255;
  }ctx.putImageData(data,0,0);return c.toDataURL();
}
export function LiquidGlass({settings,wallpaper,children,onError}:{settings:Settings;wallpaper:Wallpaper|null;children:ReactNode;onError:(s:string)=>void}) {
  const canvas=useRef<HTMLCanvasElement>(null),root=useRef<HTMLDivElement>(null);
  const map=useMemo(()=>displacement(settings.width,settings.height),[settings.width,settings.height]);
  useEffect(()=>{
    if(!settings.static_wallpaper||!wallpaper)return;let disposed=false,busy=false;const images=new Map<string,HTMLImageElement>();
    const load=Promise.all(wallpaper.monitors.map(m=>new Promise<void>(resolve=>{if(!m.image)return resolve();const img=new Image();img.onload=()=>{images.set(m.id,img);resolve()};img.onerror=()=>{onError('Formato de wallpaper não suportado pelo WebView2');resolve()};img.src=m.image})));
    let last='';
    async function draw(){if(disposed||busy||document.hidden)return;busy=true;try{
      await load;const r=await invoke<[number,number,number,number]>('widget_rect');if(disposed)return;
      const key=r.join(',');if(key===last)return;last=key;
      const c=canvas.current;if(!c)return;const ctx=c.getContext('2d')!;
      c.width=r[2];c.height=r[3];
      const color=wallpaper!.color;ctx.fillStyle=`rgb(${color&255},${(color>>8)&255},${(color>>16)&255})`;ctx.fillRect(0,0,c.width,c.height);
      const bounds=union(wallpaper!.monitors.map(m=>m.rect));
      for(const m of wallpaper!.monitors){const img=images.get(m.id);if(!img)continue;
        ctx.save();ctx.beginPath();ctx.rect(m.rect[0]-r[0],m.rect[1]-r[1],m.rect[2],m.rect[3]);ctx.clip();
        if(wallpaper!.mode===1){const pattern=ctx.createPattern(img,'repeat')!;ctx.translate(bounds[0]-r[0],bounds[1]-r[1]);ctx.fillStyle=pattern;ctx.fillRect(r[0]-bounds[0],r[1]-bounds[1],r[2],r[3]);}
        else {const [x,y,w,h]=imageRect(wallpaper!.mode,img.naturalWidth,img.naturalHeight,wallpaper!.mode===5?bounds:m.rect);ctx.drawImage(img,x-r[0],y-r[1],w,h)}ctx.restore();
      }
    }catch(e){onError(String(e))}finally{busy=false}}
    void draw();const timer=window.setInterval(draw,settings.edit?33:1000);return()=>{disposed=true;clearInterval(timer)};
  },[wallpaper,settings.edit,settings.static_wallpaper,onError]);
  return <div className="glass" style={{"--glass-tint":settings.tint,"--glass-shine":settings.shine,"--glass-shade":settings.shade} as CSSProperties} ref={root} onPointerMove={e=>{const b=e.currentTarget.getBoundingClientRect();e.currentTarget.style.setProperty('--light-x',`${(e.clientX-b.left)/b.width*100}%`);e.currentTarget.style.setProperty('--light-y',`${(e.clientY-b.top)/b.height*100}%`)}} onPointerDown={()=>{if(!matchMedia('(prefers-reduced-motion: reduce)').matches)gsap.to(root.current,{scale:.985,duration:.16})}} onPointerUp={()=>gsap.to(root.current,{scale:1,duration:.55,ease:'elastic.out(1,.5)'})} onPointerLeave={()=>gsap.to(root.current,{scale:1,duration:.3})}>
    <svg className="filter-defs" aria-hidden="true"><defs><filter id="lens" x="0" y="0" width="100%" height="100%" colorInterpolationFilters="sRGB"><feImage href={map} result="normal" preserveAspectRatio="none" width="100%" height="100%"/><feDisplacementMap in="SourceGraphic" in2="normal" scale={settings.refraction} xChannelSelector="R" yChannelSelector="G"/><feGaussianBlur stdDeviation="0.35"/></filter></defs></svg>
    {settings.static_wallpaper&&<canvas ref={canvas} className="wallpaper-texture" style={{filter:'url(#lens)'}}/>}<div className="glass-tint" style={{background:settings.tint,opacity:settings.opacity}}/><div className="glass-light"/><div className="glass-rim"/><div className="glass-caustic"/><div className="glass-readability"/><div className="glass-content">{children}</div>
  </div>
}
