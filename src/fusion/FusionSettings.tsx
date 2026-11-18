import {useState,useEffect} from 'react';import {invoke} from '@tauri-apps/api/core';
import type {FusionConfig} from './types';import type {WidgetId} from '../types';import {materialPatch} from '../glass/materials';import {catalog} from '../widgets/catalog';
export function FusionSettings({config,selected,onError}:{config:FusionConfig;selected:WidgetId;onError:(e:string)=>void}){
 const [busy,setBusy]=useState(false);
 const [monitors,setMonitors]=useState<{index:number;name:string}[]>([]);
 useEffect(()=>{void invoke<{index:number;name:string}[]>('island_monitors').then(setMonitors).catch(e=>onError(String(e)))},[onError]);
 async function monitor(index:number){setBusy(true);try{await invoke('update_fusion',{enabled:config.enabled,source:config.source,monitor:index});onError('')}catch(e){onError(String(e))}finally{setBusy(false)}}
 async function apply(enabled:boolean,prism=false){setBusy(true);try{const source=prism?selected:config.source;if(prism)await invoke('update_widget',{id:source,patch:materialPatch('liquid')});await invoke('update_fusion',{enabled,source,monitor:config.monitor});onError('')}catch(e){onError(String(e))}finally{setBusy(false)}}
 return <section className="island-settings fusion-settings"><div className="section-title"><div><h2>Widgets unidos</h2><p className="mode-help">Uma lente contínua: aproxime os widgets para fundir as bordas.</p></div><label className="toggle"><input type="checkbox" checked={config.enabled} disabled={busy} onChange={e=>void apply(e.target.checked)}/> Ativar união</label></div>
 <button className="widget-toggle" disabled={busy} onClick={()=>void apply(true,true)}>Usar vidro líquido de {catalog[selected].name}</button>
 <label style={{marginLeft:16,fontSize:12}}>Monitor <select value={config.monitor} disabled={busy||!monitors.length} onChange={e=>void monitor(+e.target.value)}>{!monitors.some(m=>m.index===config.monitor)&&<option value={config.monitor}>Monitor salvo indisponível</option>}{monitors.map(m=><option key={m.index} value={m.index}>{m.name}</option>)}</select></label>
 <p className="mode-help">O grupo usa o material de {catalog[config.source].name}, com posições próprias no monitor {config.monitor+1}. Ative “Permitir arrastar” nos widgets. Ao desativar, as janelas individuais voltam às posições anteriores. No material, desative “Fundo animado” para refratar o wallpaper estático. Com Wallpaper Engine há união e transparência, sem refração do vídeo. Não usa Acrylic.</p></section>
}
