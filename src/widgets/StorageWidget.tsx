import {useEffect,useRef,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import gsap from 'gsap';
import type {Settings} from '../types';
type Drive={id:string;name:string;total:number;available:number};
const size=(bytes:number)=>`${(bytes/1073741824).toFixed(bytes>=107374182400?0:1)} GiB`;
const circumference=2*Math.PI*26;
function StorageRing({percent}:{percent:number}){const circle=useRef<SVGCircleElement>(null);useEffect(()=>{const el=circle.current;if(!el)return;const target=circumference*(1-percent/100);if(matchMedia('(prefers-reduced-motion: reduce)').matches){el.style.strokeDashoffset=String(target);return}const tween=gsap.to(el,{strokeDashoffset:target,duration:.65,ease:'power2.out'});return()=>{tween.kill()}},[percent]);const color=percent>=90?'#f38272':percent>=75?'#edc477':'#93d7bb';return <div className="storage-ring"><svg viewBox="0 0 64 64" aria-hidden="true"><circle cx="32" cy="32" r="26" fill="none" stroke="#ffffff35" strokeWidth="6"/><circle ref={circle} cx="32" cy="32" r="26" fill="none" stroke={color} strokeWidth="6" strokeLinecap="round" strokeDasharray={circumference} strokeDashoffset={circumference}/></svg><b>{percent}%</b></div>}
export function StorageWidget({settings}:{settings:Settings}){
  const [drives,setDrives]=useState<Drive[]>([]),[error,setError]=useState('');
  useEffect(()=>{let disposed=false,busy=false;async function refresh(){if(busy||document.hidden)return;busy=true;try{const next=await invoke<Drive[]>('storage_snapshot');if(!disposed){setDrives(next);setError('')}}catch(e){if(!disposed)setError(String(e))}finally{busy=false}}void refresh();const timer=setInterval(refresh,15000);return()=>{disposed=true;clearInterval(timer)}},[]);
  const selected=drives.filter(d=>!settings.storage_devices.length||settings.storage_devices.includes(d.id));
  return <div className="storage-widget"><div className="widget-heading"><h2>Armazenamento</h2><span className="live-label">{selected.length} {selected.length===1?'unidade':'unidades'}</span></div>{selected.length?<div className="storage-list">{selected.map(d=>{const used=Math.max(0,d.total-d.available),percent=Math.min(100,Math.round(used/d.total*100));return <div className="storage-drive" key={d.id}><StorageRing percent={percent}/><div><strong>{d.name||d.id}</strong><small>{d.id}</small><span>{size(used)} usados de {size(d.total)}</span><small>{size(d.available)} livres</small></div></div>})}</div>:<p className="widget-empty">{error||'Nenhuma unidade disponível ou selecionada.'}</p>}</div>;
}
