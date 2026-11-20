import {useCallback,useEffect,useState,type CSSProperties} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {emit,listen} from '@tauri-apps/api/event';
import type {Settings,WidgetId} from '../types';
import {SurfaceGlass} from '../glass/SurfaceGlass';
import {useWidgetDrag} from './useWidgetDrag';
import {WidgetContent} from './WidgetContent';
import {catalog} from './catalog';

export function WidgetWindow({id}:{id:WidgetId}) {
  const [settings,setSettings]=useState<Settings|null>(null),[error,setError]=useState('');
  const report=useCallback((message:string)=>{setError(message);if(message)void emit('widget-error',`${catalog[id].name}: ${message}`).catch(()=>{})},[id]);
  const drag=useWidgetDrag(settings?.edit??false,report);
  useEffect(()=>{let disposed=false;const off=listen<Settings>('settings',e=>setSettings(e.payload));
    invoke<Settings>('get_settings').then(value=>!disposed&&setSettings(value)).catch(e=>report(String(e)));
    return()=>{disposed=true;void off.then(f=>f())};
  },[report]);

  if(!settings)return error?<div className="widget-error">{error}</div>:null;
  return <div className="unified-widget" style={{'--glass-tint':settings.tint} as CSSProperties}>{(id!=='gallery'||settings.gallery_frame)&&<SurfaceGlass selector=".widget-surface" material={settings} onError={report}/>}<div className={`widget-surface kind-${id}`} data-editing={settings.edit} data-dragging={drag.dragging} tabIndex={settings.edit?0:-1} aria-label={catalog[id].name} {...drag.handlers}>
    <div className="widget-body"><WidgetContent id={id} settings={settings}/></div>
    {(error||settings.edit)&&<div className="widget-status">{error?<div data-no-drag role="alert" className="widget-error">{error}</div>:<span className="edit-note">Arraste pelo fundo · Esc cancela</span>}</div>}
  </div></div>;
}
