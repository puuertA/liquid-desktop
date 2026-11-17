import {useEffect,useState} from 'react';
import type {Settings} from '../types';
export function useNow(seconds=true) {
  const [now,setNow]=useState(()=>new Date());
  useEffect(()=>{const timer=setInterval(()=>{if(!document.hidden)setNow(new Date())},seconds?1000:10000);return()=>clearInterval(timer)},[seconds]);
  return now;
}
export function DigitalClock({settings}:{settings:Settings}) {
  const now=useNow(settings.show_seconds);
  return <div className="digital-clock"><div className="clock-label">{now.toLocaleDateString('pt-BR',{weekday:'long'})}</div><time className={settings.show_seconds?'with-seconds':''}>{now.toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit',...(settings.show_seconds?{second:'2-digit'}:{}),hour12:!settings.hour24})}</time><div className="clock-date">{now.toLocaleDateString('pt-BR',{day:'numeric',month:'long'})}</div></div>;
}
export function AnalogClock({settings}:{settings:Settings}) {
  const now=useNow(settings.show_seconds);
  const seconds=now.getSeconds(),minute=now.getMinutes()+seconds/60,hour=now.getHours()%12+minute/60;
  return <div className="analog-clock"><svg viewBox="0 0 200 200" role="img" aria-label={now.toLocaleTimeString('pt-BR')}>
    {Array.from({length:60},(_,i)=><line key={i} x1="100" y1={i%5===0?15:19} x2="100" y2={i%5===0?28:24} transform={`rotate(${i*6} 100 100)`} stroke="currentColor" opacity={i%5===0?.85:.3} strokeWidth={i%5===0?2:1}/>)}
    <line x1="100" y1="104" x2="100" y2="56" stroke="currentColor" strokeWidth="6" strokeLinecap="round" transform={`rotate(${hour*30} 100 100)`}/>
    <line x1="100" y1="106" x2="100" y2="35" stroke="currentColor" strokeWidth="4" strokeLinecap="round" transform={`rotate(${minute*6} 100 100)`}/>
    {settings.show_seconds&&<line x1="100" y1="112" x2="100" y2="26" stroke={settings.tint} strokeWidth="1.7" transform={`rotate(${seconds*6} 100 100)`}/>}
    <circle cx="100" cy="100" r="4" fill={settings.tint}/>
  </svg><span>{now.toLocaleDateString('pt-BR',{day:'numeric',month:'short'})}</span></div>;
}
