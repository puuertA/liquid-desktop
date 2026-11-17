import {useState} from 'react';
import {monthCells} from './calendar.mjs';
import {useNow} from './ClockWidgets';
export function CalendarWidget() {
  const today=useNow(false),[offset,setOffset]=useState(0);
  const month=new Date(today.getFullYear(),today.getMonth()+offset,1);
  const sameMonth=month.getFullYear()===today.getFullYear()&&month.getMonth()===today.getMonth();
  return <div className="calendar-widget"><div className="widget-heading"><h2>{month.toLocaleDateString('pt-BR',{month:'long',year:'numeric'})}</h2><div className="calendar-nav" data-no-drag><button aria-label="Mês anterior" onClick={()=>setOffset(v=>v-1)}>‹</button><button aria-label="Mês seguinte" onClick={()=>setOffset(v=>v+1)}>›</button></div></div>
    <div className="calendar-week">{['S','T','Q','Q','S','S','D'].map((day,i)=><span key={i}>{day}</span>)}</div>
    <div className="calendar-grid">{monthCells(month.getFullYear(),month.getMonth()).map((day,index)=><span key={index} className={sameMonth&&day===today.getDate()?'today':''} aria-current={sameMonth&&day===today.getDate()?'date':undefined}>{day}</span>)}</div>
    <button className="calendar-today" onClick={()=>setOffset(0)}>Hoje · {today.toLocaleDateString('pt-BR',{weekday:'short',day:'numeric',month:'short'})}</button>
  </div>;
}
