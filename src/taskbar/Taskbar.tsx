import {useCallback,useEffect,useMemo,useRef,useState,type CSSProperties,type DragEvent} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {gsap} from 'gsap';
import {SurfaceGlass} from '../glass/SurfaceGlass';
import {materialPatch} from '../glass/materials';
import type {Layout,Settings} from '../types';
import type {TaskbarApp,TaskbarSnapshot} from './types';
import './taskbar.css';

type IconName='start'|'search'|'windows'|'volume'|'mute'|'network'|'offline'|'battery'|'settings'|'bell'|'chevron'|'pin'|'close'|'clipboard'|'bluetooth'|'play';
const paths:Record<IconName,string>={start:'M3 3h8v8H3z M13 3h8v8h-8z M3 13h8v8H3z M13 13h8v8h-8z',search:'m20 20-4.3-4.3 M10.8 17a6.2 6.2 0 1 0 0-12.4 6.2 6.2 0 0 0 0 12.4z',windows:'M3 5h8v6H3z M13 5h8v6h-8z M3 13h8v6H3z M13 13h8v6h-8z',volume:'M4 9h4l5-4v14l-5-4H4z M17 8c2 2 2 6 0 8 M20 5c4 4 4 10 0 14',mute:'M4 9h4l5-4v14l-5-4H4z M17 9l5 6 M22 9l-5 6',network:'M3 9a15 15 0 0 1 18 0 M6 12a10 10 0 0 1 12 0 M9 15a5 5 0 0 1 6 0 M12 19h.01',offline:'M4 4l16 16 M5 9a15 15 0 0 1 14-1 M7 13a10 10 0 0 1 7-1 M10 16a5 5 0 0 1 2 0',battery:'M3 7h16v10H3z M21 10v4',settings:'M12 2v3 M12 19v3 M2 12h3 M19 12h3 M5 5l2 2 M17 17l2 2 M19 5l-2 2 M7 17l-2 2 M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',bell:'M18 9a6 6 0 0 0-12 0c0 6-2 7-2 8h16c0-1-2-2-2-8 M10 20h4',chevron:'m6 9 6 6 6-6',pin:'M8 3h8l-1 6 3 3v2H6v-2l3-3z M12 14v7',close:'M5 5l14 14 M19 5 5 19',clipboard:'M8 4h8 M9 2h6v4H9z M6 4H4v18h16V4h-2',bluetooth:'M7 7l10 10-5 5V2l5 5L7 17',play:'m8 5 11 7-11 7z'};
function Icon({name,size=18}:{name:IconName;size?:number}){return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name]}/></svg>}
function AppIcon({app}:{app:TaskbarApp}){return app.icon?<img src={app.icon} alt="" draggable={false}/>:<span className="taskbar-fallback-icon" aria-hidden="true">{app.label.slice(0,1).toLocaleUpperCase('pt-BR')}</span>}
export function Taskbar(){
 const [snapshot,setSnapshot]=useState<TaskbarSnapshot|null>(null),[shared,setShared]=useState<Settings|null>(null),[now,setNow]=useState(new Date()),[error,setError]=useState(''),[menu,setMenu]=useState<{app:TaskbarApp;x:number;y:number}|null>(null),[hidden,setHidden]=useState(false),[weather,setWeather]=useState<string|null>(null);
 const host=useRef<HTMLDivElement>(null),leaveTimer=useRef<ReturnType<typeof setTimeout>|null>(null),dragged=useRef<string|null>(null);
 const report=useCallback((message:string)=>setError(message),[]);
 const refresh=useCallback(async()=>{try{const result=await invoke<TaskbarSnapshot>('taskbar_snapshot');setSnapshot(result);setError('')}catch(e){setError(String(e))}},[]);
 useEffect(()=>{let disposed=false;let busy=false;const poll=async()=>{if(disposed||busy||document.hidden)return;busy=true;try{await refresh()}finally{busy=false}};
  const off=listen('taskbar-refresh',()=>void poll());const materialOff=listen<Settings>('surface-material',e=>setShared(e.payload));void invoke<Settings>('surface_material').then(setShared).catch(e=>setError(String(e)));void poll();const timer=setInterval(()=>void poll(),1800),clock=setInterval(()=>setNow(new Date()),1000);
  return()=>{disposed=true;clearInterval(timer);clearInterval(clock);void off.then(f=>f());void materialOff.then(f=>f())};
 },[refresh]);
 const config=snapshot?.config;
 useEffect(()=>{const dismiss=(event:KeyboardEvent)=>{if(event.key==='Escape')setMenu(null)};const blur=()=>setMenu(null);window.addEventListener('keydown',dismiss);window.addEventListener('blur',blur);return()=>{window.removeEventListener('keydown',dismiss);window.removeEventListener('blur',blur)}},[]);
 useEffect(()=>{if(!config?.show_weather)return;let disposed=false;async function update(){try{const layout=await invoke<Layout>('get_layout');const settings=layout.widgets.weather;if(settings.weather_latitude===null||settings.weather_longitude===null){if(!disposed)setWeather(null);return}const url=new URL('https://api.open-meteo.com/v1/forecast');url.search=new URLSearchParams({latitude:String(settings.weather_latitude),longitude:String(settings.weather_longitude),current:'temperature_2m',temperature_unit:settings.weather_unit}).toString();const response=await fetch(url);if(!response.ok)throw Error('Clima indisponível');const data=await response.json() as {current?:{temperature_2m?:number}};if(!disposed)setWeather(data.current?.temperature_2m===undefined?null:`${Math.round(data.current.temperature_2m)}°`)}catch{if(!disposed)setWeather(null)}}void update();const timer=setInterval(()=>void update(),20*60_000);return()=>{disposed=true;clearInterval(timer)}},[config?.show_weather]);
 const material=useMemo(()=>{if(!shared||!config)return undefined;const preset=config.material==='shared'?{}:materialPatch(config.material);return {...shared,...preset,opacity:config.material==='shared'?shared.opacity:config.opacity,refraction:shared.refraction*config.refraction} as Settings},[shared,config]);
 // The HWND region follows the rendered cards, including the temporary menu.
 useEffect(()=>{if(!config||!host.current)return;let frame=0,disposed=false;const update=()=>{frame=0;if(disposed)return;const regions=Array.from(host.current!.querySelectorAll<HTMLElement>(hidden?'.taskbar-peek':'.taskbar-segment,.taskbar-menu')).filter(el=>getComputedStyle(el).display!=='none').map(el=>{const r=el.getBoundingClientRect();return [r.left,r.top,r.width,r.height] as [number,number,number,number]});void invoke('taskbar_region',{rects:regions}).catch(e=>setError(String(e)))};const schedule=()=>{if(!frame)frame=requestAnimationFrame(update)};
 const observer=new ResizeObserver(schedule);observer.observe(host.current);host.current.querySelectorAll('.taskbar-segment,.taskbar-menu,.taskbar-peek').forEach(el=>observer.observe(el));window.addEventListener('resize',schedule);const moved=listen('taskbar-placement',schedule),resized=getCurrentWindow().onResized(schedule);schedule();return()=>{disposed=true;cancelAnimationFrame(frame);observer.disconnect();window.removeEventListener('resize',schedule);void moved.then(f=>f());void resized.then(f=>f())};
 },[config,menu,hidden,snapshot?.apps.length]);
 useEffect(()=>{if(!host.current||!config||matchMedia('(prefers-reduced-motion: reduce)').matches)return;const nodes=host.current.querySelectorAll('.taskbar-segment');gsap.fromTo(nodes,{opacity:0,y:12},{opacity:1,y:0,duration:.42,stagger:.045,ease:'power2.out'});return()=>{gsap.killTweensOf(nodes)}},[Boolean(config)]);
 function shortcut(action:string){setMenu(null);void invoke('taskbar_shortcut',{action}).catch(e=>setError(String(e)))}
 function act(app:TaskbarApp,action:string){setMenu(null);void invoke('taskbar_app_action',{id:app.id,action}).then(refresh).catch(e=>setError(String(e)))}
 function pin(app:TaskbarApp,pinned:boolean){setMenu(null);void invoke('taskbar_pin',{path:app.path,pinned}).then(refresh).catch(e=>setError(String(e)))}
 function click(app:TaskbarApp){act(app,app.running?(app.active?'minimize':'focus'):'launch')}
 function magnify(e:React.PointerEvent<HTMLDivElement>){if(!config||matchMedia('(prefers-reduced-motion: reduce)').matches)return;const icons=e.currentTarget.querySelectorAll<HTMLElement>('.taskbar-app');for(const icon of Array.from(icons)){const r=icon.getBoundingClientRect(),distance=Math.abs(e.clientX-r.left-r.width/2),influence=Math.max(0,1-distance/105);gsap.to(icon,{scale:1+config.magnification*influence,y:-6*influence,duration:.22,ease:'power2.out',overwrite:true})}}
 function resetMagnify(e:React.PointerEvent<HTMLDivElement>){gsap.to(e.currentTarget.querySelectorAll('.taskbar-app'),{scale:1,y:0,duration:.3,ease:'power2.out',overwrite:true})}
 function drop(e:DragEvent<HTMLButtonElement>,target:TaskbarApp){e.preventDefault();const source=dragged.current;if(!source||source===target.id||!config)return;const apps=snapshot!.apps;const order=apps.filter(a=>a.pinned).map(a=>a.path);const from=apps.find(a=>a.id===source);if(!from||!from.pinned||!target.pinned)return;const without=order.filter(p=>p.toLowerCase()!==source);const idx=without.findIndex(p=>p.toLowerCase()===target.id);without.splice(Math.max(0,idx),0,from.path);void invoke('taskbar_reorder',{paths:without}).then(refresh).catch(err=>setError(String(err)));dragged.current=null}
 function enter(){if(leaveTimer.current)clearTimeout(leaveTimer.current);setHidden(false)}
 function leave(){if(config?.auto_hide){leaveTimer.current=setTimeout(()=>{setHidden(true);setMenu(null)},850)}}
 if(!config)return <div className="taskbar-load" title={error}>Liquid Desktop</div>;
 const apps=snapshot?.apps??[],time=now.toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'}),day=now.toLocaleDateString('pt-BR',{day:'2-digit',month:'2-digit'});
 const style={'--taskbar-height':`${config.height}px`,'--taskbar-radius':`${config.radius}px`,'--taskbar-icon':`${config.icon_size}px`} as CSSProperties;
 return <div ref={host} className={`taskbar-host taskbar-width-${config.width}`} data-hidden={hidden} style={style} onPointerEnter={enter} onPointerLeave={leave}>
  <SurfaceGlass selector=".taskbar-segment" material={material} onError={report}/>
  <div className="taskbar-peek" title="Mostrar barra de tarefas" onPointerEnter={enter}/>
  <div className="taskbar-row" role="toolbar" aria-label="Barra de tarefas Liquid Desktop">
   <section className="taskbar-segment taskbar-start" style={{order:config.segment_order.indexOf('start')}}>
    <button title="Iniciar" aria-label="Abrir Iniciar" onClick={()=>shortcut('start')}><Icon name="start" size={21}/></button>
    {config.show_search&&<button title="Pesquisar" aria-label="Pesquisar no Windows" onClick={()=>shortcut('search')}><Icon name="search"/></button>}
    {config.show_task_view&&<button title="Visão de tarefas" aria-label="Visão de tarefas" onClick={()=>shortcut('task_view')}><Icon name="windows"/></button>}
    {config.show_weather&&weather&&<span className="taskbar-chip" title="Temperatura atual da cidade configurada no widget Clima">{weather}</span>}
   </section>
   <section className="taskbar-segment taskbar-apps" style={{order:config.segment_order.indexOf('apps')}} onPointerMove={magnify} onPointerLeave={resetMagnify} onWheel={e=>{e.currentTarget.scrollLeft+=e.deltaY}} aria-label="Aplicativos fixados e abertos">
    {apps.length?apps.map(app=><button key={app.id} className="taskbar-app" data-active={app.active} data-running={app.running} data-minimized={app.minimized} title={`${app.label}${app.windows.length>1?` · ${app.windows.length} janelas`:''}`} aria-label={`${app.label}${app.active?' — ativo':''}`} onClick={()=>click(app)} onContextMenu={e=>{e.preventDefault();const root=host.current?.getBoundingClientRect();setMenu({app,x:e.clientX-(root?.left??0),y:e.clientY-(root?.top??0)})}} draggable={app.pinned} onDragStart={()=>{dragged.current=app.id}} onDragOver={e=>e.preventDefault()} onDrop={e=>drop(e,app)}>
     <AppIcon app={app}/>{config.show_labels&&<span className="taskbar-app-label">{app.label}</span>}<span className="taskbar-active-line"/>{app.windows.length>1&&<span className="taskbar-multiple"/>}
    </button>):<span className="taskbar-empty">Abra ou fixe um aplicativo</span>}
   </section>
   <section className="taskbar-segment taskbar-system" style={{order:config.segment_order.indexOf('system')}}>
    {config.show_media&&snapshot?.media&&<span className="taskbar-chip" title={snapshot.media}><Icon name="play" size={14}/></span>}
    {config.show_system&&<><button title={snapshot?.network?'Rede conectada':'Rede desconectada'} onClick={()=>shortcut('quick_settings')} aria-label="Rede e configurações rápidas"><Icon name={snapshot?.network?'network':'offline'}/></button><button title={`Volume ${Math.round((snapshot?.volume?.level??0)*100)}%`} onClick={()=>void invoke('toggle_center').catch(e=>setError(String(e)))} aria-label="Abrir Control Center"><Icon name={snapshot?.volume?.muted?'mute':'volume'}/></button>{snapshot?.battery!==null&&<button title={`Bateria ${snapshot?.battery??0}%${snapshot?.charging?' · carregando':''}`} onClick={()=>shortcut('quick_settings')} aria-label="Bateria e energia"><Icon name="battery"/><small>{snapshot?.battery}%</small></button>}</>}
    {config.show_bluetooth&&<button title="Abrir Bluetooth nas configurações rápidas" onClick={()=>shortcut('quick_settings')} aria-label="Bluetooth"><Icon name="bluetooth"/></button>}
    {config.show_clipboard&&<button title="Histórico da área de transferência" onClick={()=>shortcut('clipboard')} aria-label="Área de transferência"><Icon name="clipboard"/></button>}
    {config.show_clock&&<button className="taskbar-clock" title={now.toLocaleString('pt-BR')} onClick={()=>shortcut('notifications')}><strong>{time}</strong><small>{day}</small></button>}
    <button title="Notificações" aria-label="Notificações" onClick={()=>shortcut('notifications')}><Icon name="bell"/></button>
    <button title="Control Center" aria-label="Abrir Control Center" onClick={()=>void invoke('toggle_center').catch(e=>setError(String(e)))}><Icon name="settings"/></button>
   </section>
  </div>
  {menu&&<div className="taskbar-menu" role="menu" style={{left:Math.max(8,Math.min(menu.x-70,innerWidth-210)),bottom:Math.max(88,innerHeight-menu.y+12)}}>
   <strong>{menu.app.label}</strong>
   <button role="menuitem" onClick={()=>act(menu.app,'launch')}>Abrir nova janela</button>
   {menu.app.running&&<><button role="menuitem" onClick={()=>act(menu.app,'focus')}>Mostrar janela</button><button role="menuitem" onClick={()=>act(menu.app,'minimize')}>Minimizar</button><button role="menuitem" onClick={()=>act(menu.app,'close')}>Fechar janela</button></>}
   {menu.app.windows.length>1&&menu.app.windows.map((hwnd,i)=><button role="menuitem" key={hwnd} title={menu.app.window_titles[i]} onClick={()=>{setMenu(null);void invoke('taskbar_window_focus',{id:menu.app.id,hwnd}).then(refresh).catch(e=>setError(String(e)))}}>Janela {i+1}: {menu.app.window_titles[i]}</button>)}
   <button role="menuitem" onClick={()=>pin(menu.app,!menu.app.pinned)}>{menu.app.pinned?'Desafixar':'Fixar na barra'}</button>
  </div>}
  {error&&<button className="taskbar-error" title={error} onClick={()=>setError('')}>!</button>}
 </div>
}
