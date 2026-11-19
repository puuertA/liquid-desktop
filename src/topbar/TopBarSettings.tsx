import {useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';

import type {TopBarConfig} from './types';
import './topbar.css';
export function TopBarSettings({config,onError}:{config:TopBarConfig;onError:(value:string)=>void}){
  const [busy,setBusy]=useState(false),[monitors,setMonitors]=useState<{index:number;name:string}[]>([]);

  useEffect(()=>{let disposed=false;void invoke<{index:number;name:string}[]>('island_monitors').then(value=>{if(!disposed)setMonitors(value)}).catch(e=>onError(String(e)));return()=>{disposed=true}},[onError]);
  async function change(patch:Partial<TopBarConfig>){setBusy(true);try{await invoke('update_topbar',{config:{...config,...patch}});onError('')}catch(e){onError(String(e))}finally{setBusy(false)}}
  const flags:[keyof TopBarConfig,string][]=[['show_date','Data'],['show_seconds','Segundos'],['show_cpu','CPU'],['show_memory','RAM'],['show_network','Rede'],['show_volume','Volume']];
  return <section className="island-settings topbar-settings"><div className="section-title"><div><h2>Top Bar</h2><p className="mode-help">Hora e informações do computador, sempre à mão.</p></div><label className="toggle"><input type="checkbox" disabled={busy} checked={config.enabled} onChange={e=>void change({enabled:e.target.checked})}/> Ativar barra</label></div>
    <div className="island-settings-grid"><label>Monitor<select disabled={busy||!monitors.length} value={config.monitor} onChange={e=>void change({monitor:+e.target.value})}>{!monitors.some(m=>m.index===config.monitor)&&<option value={config.monitor}>Monitor salvo indisponível</option>}{monitors.map(m=><option key={m.index} value={m.index}>{m.name}</option>)}</select></label>
    <label className="toggle"><input type="checkbox" checked={config.above_apps} disabled={busy} onChange={e=>void change({above_apps:e.target.checked})}/> Acima de aplicativos</label>
    <label className="toggle"><input type="checkbox" checked={config.hide_fullscreen} disabled={busy} onChange={e=>void change({hide_fullscreen:e.target.checked})}/> Ocultar em tela cheia</label></div>
    <div className="bar-module-options">{flags.map(([key,label])=><label className="toggle" key={key}><input type="checkbox" disabled={busy} checked={Boolean(config[key])} onChange={e=>void change({[key]:e.target.checked})}/>{label}</label>)}</div>
    <p className="mode-help">O vidro segue o material dos widgets: cor, transparência, refração, dispersão e fonte do fundo. Ajuste no painel Material.</p>
    <p className="mode-help">A ilha ganha espaço livre no centro quando usa o mesmo monitor. Em telas menores, detalhes se ocultam para caber. Clique no ícone à esquerda para abrir o painel. CPU, RAM e rede atualizam a cada 2 segundos. A barra reserva uma faixa no topo para os aplicativos maximizados. A ilha compartilha essa faixa; expandi-la não redimensiona os aplicativos. A barra de tarefas do Windows é preservada.</p>
  </section>;
}
