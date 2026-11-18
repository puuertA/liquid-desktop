import {useEffect,useRef} from 'react';
import gsap from 'gsap';
import {invoke} from '@tauri-apps/api/core';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {stepIsland,contentGeometry} from './motion.mjs';
import {geometries,geometryQueue} from './state.mjs';
export type IslandState=keyof typeof geometries;
export function useIslandMotion(state:IslandState,enabled:boolean,form:string,onError:(message:string)=>void){
  const root=useRef<HTMLDivElement>(null),values=useRef({width:140,height:36,radius:18,vw:0,vh:0,vr:0});
  const queue=useRef<ReturnType<typeof geometryQueue>|null>(null);
  // Keep one queue across transitions so a stale contour cannot overtake a new one.
  useEffect(()=>{
    const current=geometryQueue((value:{width:number;height:number;radius:number})=>invoke('island_region',value));queue.current=current;
    return()=>{current.dispose();queue.current=null};
  },[]);
  useEffect(()=>{
    const element=root.current;if(!element)return;
    const target=geometries[state],query=matchMedia('(prefers-reduced-motion: reduce)');
    let previous=performance.now(),active=false,lastGeometry='';
    const draw=(done=false)=>{
      const value=values.current,{expansion,topline}=contentGeometry(value.height);
      element.style.width=`${value.width}px`;element.style.height=`${value.height}px`;
      element.style.borderRadius=form==='notch'?`0 0 ${value.radius}px ${value.radius}px`:`${value.radius}px`;
      element.style.setProperty('--expansion',String(expansion));element.style.setProperty('--topline',`${topline}px`);
      const density=window.devicePixelRatio||1;
      const geometry={width:Math.round(value.width*density)/density,height:Math.round(value.height*density)/density,radius:Math.round(value.radius*density)/density};
      const key=JSON.stringify(geometry);
      if(done||key!==lastGeometry){lastGeometry=key;void queue.current?.set(geometry).catch(e=>onError(String(e)))}
    };
    const stop=()=>{gsap.ticker.remove(tick);active=false};
    const snap=()=>{stop();Object.assign(values.current,{width:target[0],height:target[1],radius:target[2],vw:0,vh:0,vr:0});draw(true)};
    const tick=()=>{
      const now=performance.now(),dt=(now-previous)/1000;previous=now;
      const v=stepIsland(values.current,target,dt);values.current=v;
      draw();
      if(Math.abs(v.width-target[0])+Math.abs(v.height-target[1])+Math.abs(v.radius-target[2])<.08&&Math.abs(v.vw)+Math.abs(v.vh)+Math.abs(v.vr)<.15)snap();
    };
    const preference=()=>{if(query.matches||!enabled)snap()};
    query.addEventListener('change',preference);
    const dpi=getCurrentWindow().onScaleChanged(()=>draw(true));
    if(query.matches||!enabled)snap();else{active=true;gsap.ticker.add(tick)}
    return()=>{if(active)stop();query.removeEventListener('change',preference);void dpi.then(f=>f()).catch(e=>onError(String(e)))};
  },[state,enabled,form,onError]);
  return root;
}
