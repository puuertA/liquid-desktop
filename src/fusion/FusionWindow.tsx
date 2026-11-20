import {useCallback,useEffect,useRef,useState,type PointerEvent,type CSSProperties} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {listen,emit} from '@tauri-apps/api/event';
import type {WidgetId} from '../types';
import type {FusionSnapshot,Shape} from './types';
import {FusionGlass} from './FusionGlass';
import {WidgetContent} from '../widgets/WidgetContent';
import './fusion.css';
type Positions=Partial<Record<WidgetId,[number,number]>>;
export function FusionWindow(){
  const [state,setState]=useState<FusionSnapshot|null>(null),[shapes,setShapes]=useState<Shape[]>([]),[error,setError]=useState('');
  const drag=useRef<{id:WidgetId;pointer:number;element:HTMLElement;x:number;y:number;origin:[number,number];current:[number,number]}|null>(null),ending=useRef(false),pending=useRef<Positions|null>(null),flight=useRef<Promise<void>|null>(null),last=useRef(0);
  const report=useCallback((e:string)=>{setError(e);if(e)void emit('widget-error',e)},[]);
  const refresh=useCallback(async()=>{try{const next=await invoke<FusionSnapshot>('fusion_snapshot');setState(next);if(!drag.current&&!ending.current){setShapes(next.shapes);await invoke('fusion_positions',{positions:{},commit:false})}}catch(e){report(String(e))}},[report]);
  function flush():Promise<void>{if(flight.current)return flight.current;flight.current=(async()=>{while(pending.current){const positions=pending.current;pending.current=null;await invoke('fusion_positions',{positions,commit:false})}})().finally(()=>{flight.current=null});return flight.current}
  const finish=useCallback(async(cancel=false)=>{
    const d=drag.current;if(!d||ending.current)return;ending.current=true;drag.current=null;
    if(d.element.hasPointerCapture(d.pointer))d.element.releasePointerCapture(d.pointer);
    pending.current=null;
    try{await flight.current;await invoke('fusion_positions',{positions:{[d.id]:cancel?d.origin:d.current},commit:!cancel});setError('')}
    catch(e){report(String(e))}finally{ending.current=false;await refresh()}
  },[refresh,report]);
  useEffect(()=>{void refresh();const off=listen('fusion-refresh',()=>void refresh());const blur=()=>void finish(true);const key=(e:KeyboardEvent)=>{if(e.key==='Escape')void finish(true)};window.addEventListener('blur',blur);window.addEventListener('keydown',key);return()=>{void off.then(f=>f());window.removeEventListener('blur',blur);window.removeEventListener('keydown',key);void finish(true)}},[refresh,finish]);
  function down(e:PointerEvent<HTMLDivElement>,shape:Shape){if(!state?.widgets[shape.id].edit||ending.current||drag.current||e.button!==0||(e.target as HTMLElement).closest('button,input,textarea,select,a'))return;e.preventDefault();e.currentTarget.focus();e.currentTarget.setPointerCapture(e.pointerId);drag.current={id:shape.id,pointer:e.pointerId,element:e.currentTarget,x:e.clientX,y:e.clientY,origin:[shape.x,shape.y],current:[shape.x,shape.y]}}
  function move(e:PointerEvent){const d=drag.current;if(!d||!state)return;const shape=shapes.find(s=>s.id===d.id)!;d.current=[Math.max(0,Math.min(state.width-shape.width,d.origin[0]+e.clientX-d.x)),Math.max(0,Math.min(state.height-shape.height,d.origin[1]+e.clientY-d.y))];setShapes(rows=>rows.map(r=>r.id===d.id?{...r,x:d.current[0],y:d.current[1]}:r));if(performance.now()-last.current>80){last.current=performance.now();pending.current={[d.id]:d.current};void flush().catch(e=>report(String(e)))}}
  if(!state)return null;
  return <div className="fusion-host" style={{'--glass-tint':state.widgets[state.config.source].tint} as CSSProperties}><FusionGlass snapshot={state} shapes={shapes.filter(shape=>shape.id!=='gallery'||state.widgets.gallery.gallery_frame)} onError={report}/>{shapes.map(shape=>{const s=state.widgets[shape.id];return <div key={shape.id} className={`fusion-widget widget-surface kind-${shape.id}`} data-editing={s.edit} tabIndex={s.edit?0:-1} style={{left:shape.x,top:shape.y,width:shape.width,height:shape.height}} onPointerDown={e=>down(e,shape)} onPointerMove={move} onPointerUp={()=>void finish()} onPointerCancel={()=>void finish(true)} onLostPointerCapture={()=>void finish()}>
    <div className="widget-body"><WidgetContent id={shape.id} settings={s}/></div>{(s.edit||error)&&<div className="widget-status"><span className="edit-note">{error||'Aproxime para unir · Esc cancela'}</span></div>}
  </div>})}</div>;
}
