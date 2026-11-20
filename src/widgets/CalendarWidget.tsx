import {useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {monthCells} from './calendar.mjs';
import {useNow} from './ClockWidgets';
import type {CalendarEvent,Settings} from '../types';
const key=(date:Date)=>`${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')}`;
export function CalendarWidget({settings}:{settings:Settings}) {
  const today=useNow(false),[offset,setOffset]=useState(0),[selected,setSelected]=useState(key(today)),[events,setEvents]=useState(settings.calendar_events),[adding,setAdding]=useState(false),[title,setTitle]=useState(''),[time,setTime]=useState(''),[reminder,setReminder]=useState(false),[error,setError]=useState('');
  useEffect(()=>setEvents(settings.calendar_events),[settings.calendar_events]);
  const month=new Date(today.getFullYear(),today.getMonth()+offset,1);
  const sameMonth=month.getFullYear()===today.getFullYear()&&month.getMonth()===today.getMonth();
  const upcoming=events.filter(e=>e.date>=key(today)).sort((a,b)=>(a.date+a.time).localeCompare(b.date+b.time)).slice(0,3);
  async function save(next:CalendarEvent[]){try{await invoke('save_calendar_events',{events:next});setEvents(next);setError('')}catch(e){setError(String(e))}}
  function add(){const text=title.trim();if(!text)return;void save([...events,{id:String(Date.now()),date:selected,time,title:text,reminder}]);setTitle('');setTime('');setReminder(false);setAdding(false)}
  return <div className="calendar-widget"><div className="widget-heading"><h2>{month.toLocaleDateString('pt-BR',{month:'long',year:'numeric'})}</h2><div className="calendar-nav" data-no-drag><button aria-label="Mês anterior" onClick={()=>setOffset(v=>v-1)}>‹</button><button aria-label="Mês seguinte" onClick={()=>setOffset(v=>v+1)}>›</button></div></div>
    <div className="calendar-week">{['S','T','Q','Q','S','S','D'].map((day,i)=><span key={i}>{day}</span>)}</div>
    <div className="calendar-grid">{monthCells(month.getFullYear(),month.getMonth()).map((day,index)=>{const date=day?key(new Date(month.getFullYear(),month.getMonth(),day)):'';return <button key={index} disabled={!day} className={`${sameMonth&&day===today.getDate()?'today':''} ${selected===date?'selected':''} ${events.some(e=>e.date===date)?'has-event':''}`} aria-current={sameMonth&&day===today.getDate()?'date':undefined} onClick={()=>setSelected(date)}>{day}</button>})}</div>
    <div className="calendar-agenda" data-no-drag><div><strong>{selected===key(today)?'Hoje':new Date(`${selected}T12:00:00`).toLocaleDateString('pt-BR',{day:'numeric',month:'short'})}</strong><button onClick={()=>setAdding(v=>!v)}>{adding?'Cancelar':'+ Evento'}</button></div>
      {adding?<div className="calendar-form"><input aria-label="Título do evento" value={title} maxLength={160} placeholder="Evento ou lembrete" onChange={e=>setTitle(e.target.value)} onKeyDown={e=>{if(e.key==='Enter')add()}}/><input aria-label="Horário" type="time" value={time} onChange={e=>setTime(e.target.value)}/><label><input type="checkbox" checked={reminder} onChange={e=>setReminder(e.target.checked)}/> Lembrete</label><button onClick={add}>Salvar</button></div>:<div className="calendar-events">{events.filter(e=>e.date===selected).map(e=><div key={e.id}><span>{e.reminder?'• ':''}{e.time&&`${e.time} · `}{e.title}</span><button aria-label={`Remover ${e.title}`} onClick={()=>void save(events.filter(row=>row.id!==e.id))}>×</button></div>)}{!events.some(e=>e.date===selected)&&<small>Sem compromissos neste dia</small>}</div>}
      {!!upcoming.length&&<small className="calendar-upcoming">Próximos: {upcoming.map(e=>`${e.date.slice(8)}/${e.date.slice(5,7)} ${e.title}`).join(' · ')}</small>}{error&&<small role="alert">{error}</small>}
    </div>
  </div>;
}
