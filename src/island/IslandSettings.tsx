import {useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import type {IslandConfig} from './types';
export function IslandSettings({config,onError}:{config:IslandConfig;onError:(message:string)=>void}){
  const [busy,setBusy]=useState(false);
  const [monitors,setMonitors]=useState<{index:number;name:string}[]>([]);
  useEffect(()=>{void invoke<{index:number;name:string}[]>('island_monitors').then(setMonitors).catch(e=>onError(String(e)))},[onError]);
  async function change(patch:Partial<IslandConfig>){setBusy(true);try{await invoke('update_island',{config:{...config,...patch}});onError('')}catch(e){onError(String(e))}finally{setBusy(false)}}
  return <section className="island-settings"><div className="section-title"><div><h2>Dynamic Island</h2><p className="mode-help">Música, volume e temporizador no centro superior da tela.</p></div><label className="toggle"><input type="checkbox" disabled={busy} checked={config.enabled} onChange={e=>void change({enabled:e.target.checked})}/> Ativar ilha</label></div>
    <div className="island-settings-grid"><label>Formato<select disabled={busy} value={config.form} onChange={e=>void change({form:e.target.value as IslandConfig['form']})}><option value="floating">Floating Island</option><option value="notch">Notch</option></select></label>
    <label>Aparência<select disabled={busy} value={config.appearance} onChange={e=>void change({appearance:e.target.value as IslandConfig['appearance']})}><option value="black">Black</option><option value="graphite">Graphite</option><option value="liquid">Liquid Glass</option><option value="adaptive">Adaptive · contraste seguro</option></select></label>
    <label>Monitor<select disabled={busy||!monitors.length} value={config.monitor} onChange={e=>void change({monitor:+e.target.value})}>{!monitors.some(m=>m.index===config.monitor)&&<option value={config.monitor}>Monitor salvo indisponível</option>}{monitors.map(m=><option key={m.index} value={m.index}>{m.name}</option>)}</select></label>
    <label className="toggle"><input type="checkbox" disabled={busy} checked={config.above_apps} onChange={e=>void change({above_apps:e.target.checked})}/> Acima de aplicativos</label>
    <label className="toggle"><input type="checkbox" disabled={busy} checked={config.motion} onChange={e=>void change({motion:e.target.checked})}/> Expansão elástica</label></div>
    <p className="mode-help">Liquid Glass e Adaptive seguem o material dos widgets, incluindo o fundo animado. Black e Graphite mantêm suas aparências opacas.</p>
    <p className="mode-help">Passe o cursor para expandir; Esc recolhe. A ilha reserva espaço no topo para os aplicativos maximizados; a expansão abre sobre eles. Oculta automaticamente em tela cheia. Usa a saída de áudio padrão e sessões de mídia oferecidas pelo Windows. Desativar a ilha cancela seu temporizador. O monitor volta ao primeiro disponível se o selecionado for desconectado.</p>
  </section>;
}
