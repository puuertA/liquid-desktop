import {useCallback,useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {emit,listen} from '@tauri-apps/api/event';
import type {Settings,Wallpaper,WidgetId} from '../types';
import {LiquidGlass} from '../glass/LiquidGlass';
import {useWidgetDrag} from './useWidgetDrag';
import {DigitalClock,AnalogClock} from './ClockWidgets';
import {CalendarWidget} from './CalendarWidget';
import {NotesWidget} from './NotesWidget';
import {SystemWidget} from './SystemWidget';
import {catalog} from './catalog';

export function WidgetWindow({id}:{id:WidgetId}) {
  const [settings,setSettings]=useState<Settings|null>(null),[wallpaper,setWallpaper]=useState<Wallpaper|null>(null),[error,setError]=useState('');
  const report=useCallback((message:string)=>{setError(message);if(message)void emit('widget-error',`${catalog[id].name}: ${message}`).catch(()=>{})},[id]);
  const drag=useWidgetDrag(settings?.edit??false,report);
  useEffect(()=>{let disposed=false;const off=listen<Settings>('settings',e=>setSettings(e.payload));
    invoke<Settings>('get_settings').then(value=>!disposed&&setSettings(value)).catch(e=>report(String(e)));
    return()=>{disposed=true;void off.then(f=>f())};
  },[report]);
  useEffect(()=>{if(!settings?.static_wallpaper){setWallpaper(null);return}
    let disposed=false,busy=false;async function refresh(){if(busy||document.hidden)return;busy=true;try{
      const next=await invoke<Wallpaper>('get_wallpaper');if(!disposed)setWallpaper(previous=>JSON.stringify(previous)===JSON.stringify(next)?previous:next);
    }catch(e){if(!disposed)report(String(e))}finally{busy=false}}
    void refresh();const timer=setInterval(refresh,15000);return()=>{disposed=true;clearInterval(timer)};
  },[settings?.static_wallpaper,report]);
  if(!settings)return error?<div className="widget-error">{error}</div>:null;
  return <LiquidGlass settings={settings} wallpaper={wallpaper} onError={report}><div className={`widget-surface kind-${id}`} data-editing={settings.edit} data-dragging={drag.dragging} tabIndex={settings.edit?0:-1} aria-label={catalog[id].name} {...drag.handlers}>
    <div className="widget-body">{id==='clock'?<DigitalClock settings={settings}/>:id==='analog'?<AnalogClock settings={settings}/>:id==='calendar'?<CalendarWidget/>:id==='notes'?<NotesWidget initial={settings.note}/>:<SystemWidget/>}</div>
    {(error||settings.edit)&&<div className="widget-status">{error?<div data-no-drag role="alert" className="widget-error">{error}</div>:<span className="edit-note">Arraste pelo fundo · Esc cancela</span>}</div>}
  </div></LiquidGlass>;
}
