import {useState} from 'react';
import type {Settings} from '../types';
import {MaterialPreview} from './MaterialPreview';
import {materials,materialPatch} from './materials';
import {widget} from '../panel/preview';
const base:Settings={...widget,x:0,y:0,width:360,height:220,enabled:true,refraction:18,opacity:.12,edit:false,static_wallpaper:true,tint:'#b9d5e8',shine:.65,shade:.42,show_seconds:false,hour24:true,note:'',material:'regular',thickness:18,bezel:26,dispersion:.1,frost:.35,quality:'balanced',motion:true};
export function MaterialLab(){const [settings,setSettings]=useState(base);return <main className="material-lab"><h1>Estúdio óptico</h1><p>Ambiente de validação. Textura ilustrativa, renderer real.</p><MaterialPreview settings={settings}/><div className="theme-presets">{Object.keys(materials).map(id=><button key={id} onClick={()=>setSettings({...settings,...materialPatch(id as keyof typeof materials)})}>{id}</button>)}</div><label>Refração<input aria-label="Refração" type="range" min="0" max="40" value={settings.refraction} onChange={e=>setSettings({...settings,refraction:+e.target.value})}/></label><label>Separação cromática<input aria-label="Separação cromática" type="range" min="0" max="0.5" step="0.01" value={settings.dispersion} onChange={e=>setSettings({...settings,dispersion:+e.target.value})}/></label></main>}
