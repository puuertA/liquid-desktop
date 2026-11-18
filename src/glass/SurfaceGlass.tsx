import {useEffect,useMemo,useState} from 'react';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {FusionGlass} from '../fusion/FusionGlass';
import type {FusionSnapshot,Shape} from '../fusion/types';
import type {Settings,WidgetId} from '../types';
import {materials} from './materials';
import './surface.css';

// One canvas per native surface. Resizing the island updates geometry only;
// the wallpaper crop remains anchored to the native window, not its animated card.
export function SurfaceGlass({selector,staticWallpaper,tint='#23333f',opacity=.1,onError}:{selector:string;staticWallpaper:boolean;tint?:string;opacity?:number;onError:(message:string)=>void}){
  const [geometry,setGeometry]=useState<{shapes:Shape[];screen:[number,number,number,number];scale:number;width:number;height:number}|null>(null);
  useEffect(()=>{
    let disposed=false,frame=0,position={x:0,y:0},scale=devicePixelRatio,located=false;
    const win=getCurrentWindow();
    function measure(){frame=0;if(disposed||!located)return;
      const shapes=Array.from(document.querySelectorAll<HTMLElement>(selector)).slice(0,5).map((el,i)=>{const r=el.getBoundingClientRect();return {id:(i?'notes':'clock') as WidgetId,x:r.x,y:r.y,width:r.width,height:r.height,radius:parseFloat(getComputedStyle(el).borderBottomLeftRadius)||0,topRadius:parseFloat(getComputedStyle(el).borderTopLeftRadius)||0}});
      setGeometry({shapes,screen:[position.x,position.y,Math.round(innerWidth*scale),Math.round(innerHeight*scale)],scale,width:innerWidth,height:innerHeight});
    }
    const schedule=()=>{if(!frame&&!disposed)frame=requestAnimationFrame(measure)};
    async function locate(){try{const [p,dpi]=await Promise.all([win.outerPosition(),win.scaleFactor()]);if(disposed)return;position=p;scale=dpi;located=true;schedule()}catch(e){if(!disposed)onError(String(e))}}
    const observer=new ResizeObserver(schedule);document.querySelectorAll(selector).forEach(el=>observer.observe(el));
    // Radius changes during the spring even on frames whose rounded size is equal.
    const mutation=new MutationObserver(schedule);document.querySelectorAll(selector).forEach(el=>mutation.observe(el,{attributes:true,attributeFilter:['style']}));
    const moved=win.onMoved(()=>void locate()),dpi=win.onScaleChanged(()=>void locate());
    window.addEventListener('resize',schedule);void locate();
    return()=>{disposed=true;cancelAnimationFrame(frame);observer.disconnect();mutation.disconnect();window.removeEventListener('resize',schedule);for(const off of [moved,dpi])void off.then(f=>f()).catch(()=>{})};
  },[selector,onError]);
  const snapshot=useMemo<FusionSnapshot|null>(()=>{
    if(!geometry)return null;
    const settings:Settings={...materials.liquid,x:0,y:0,width:geometry.width,height:geometry.height,enabled:true,edit:false,static_wallpaper:staticWallpaper,tint,opacity,shade:staticWallpaper?.3:.22,show_seconds:false,hour24:true,note:'',material:'liquid',quality:'balanced',motion:false,bezel:12,thickness:16,refraction:12,dispersion:.4};
    return {...geometry,config:{enabled:true,monitor:0,source:'clock',positions:{}},widgets:Object.fromEntries(['clock','analog','notes','calendar','system'].map(id=>[id,settings])) as Record<WidgetId,Settings>};
  },[geometry,staticWallpaper,tint,opacity]);
  return <div className="surface-optics" aria-hidden="true">{snapshot&&<FusionGlass snapshot={snapshot} shapes={snapshot.shapes} onError={onError}/>}</div>;
}
