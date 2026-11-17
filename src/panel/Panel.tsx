import {useCallback,useEffect,useRef,useState} from 'react';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {widgetIds,type Layout,type Settings,type WidgetId} from '../types';
import {catalog,WidgetIcon} from '../widgets/catalog';

function NumberField({label,value,min,max,onCommit}:{label:string;value:number;min:number;max:number;onCommit:(v:number)=>void}) {
  const [draft,setDraft]=useState(String(value));
  useEffect(()=>setDraft(String(value)),[value]);
  function commit(){const n=Number(draft);if(Number.isFinite(n)&&n>=min&&n<=max&&Math.round(n)!==value)onCommit(Math.round(n));else setDraft(String(value))}
  return <label className="number-field">{label}<input type="number" min={min} max={max} value={draft} onChange={e=>setDraft(e.target.value)} onBlur={commit} onKeyDown={e=>{if(e.key==='Enter')e.currentTarget.blur()}}/></label>;
}
function Range({label,value,min=0,max=1,step=.01,suffix='%',disabled=false,onChange}:{label:string;value:number;min?:number;max?:number;step?:number;suffix?:string;disabled?:boolean;onChange:(v:number)=>void}) {
  return <label className="slider-label">{label}<output>{suffix==='%'?Math.round(value*100):value.toFixed(0)}{suffix}</output><input type="range" disabled={disabled} min={min} max={max} step={step} value={value} onChange={e=>onChange(+e.target.value)}/></label>;
}
export function Panel() {
  const [layout,setLayout]=useState<Layout|null>(null),[selected,setSelected]=useState<WidgetId>('clock'),[error,setError]=useState(''),[busy,setBusy]=useState<WidgetId|null>(null),[saving,setSaving]=useState(false);
  const [startup,setStartup]=useState<boolean|null>(null),[appBusy,setAppBusy]=useState(false);
  const queue=useRef(Promise.resolve()),pending=useRef(0);
  const reload=useCallback(()=>invoke<Layout>('get_layout').then(setLayout),[]);
  useEffect(()=>{
    if(!isTauri()){setError('Abra pelo aplicativo Tauri para configurar os widgets nativos.');return}
    const off=listen<Layout>('layout',e=>setLayout(e.payload));
    const errors=listen<string>('widget-error',e=>setError(e.payload));
    void invoke<boolean>('startup_enabled').then(setStartup).catch(e=>setError(String(e)));
    void reload().catch(e=>setError(String(e)));
    return()=>{void off.then(f=>f());void errors.then(f=>f())};
  },[reload]);
  function patch(values:Partial<Settings>){
    const id=selected;setSaving(true);pending.current++;
    queue.current=queue.current.catch(()=>{}).then(()=>invoke('update_widget',{id,patch:values})).then(()=>setError('')).catch(e=>setError(String(e))).finally(()=>{pending.current--;if(!pending.current)setSaving(false)});
  }
  async function toggle(id:WidgetId){if(!layout)return;setBusy(id);setError('');try{await queue.current;await invoke(layout.widgets[id].enabled?'remove_widget':'create_widget',{id});await reload()}catch(e){setError(String(e))}finally{setBusy(null)}}
  async function applicationAction(action:()=>Promise<unknown>){setAppBusy(true);try{await queue.current;await action();setError('')}catch(e){setError(String(e))}finally{setAppBusy(false)}}
  const settings=layout?.widgets[selected],meta=catalog[selected];
  return <main className="library-panel"><header><div className="brandmark">◌</div><div><p className="eyebrow">Liquid Desktop</p><h1>Seu espaço, do seu jeito.</h1><p className="intro">Widgets independentes. Um mesmo material.</p></div><span className="phase">v0.3.0</span></header>
    {error&&<div className="error" role="alert">{error}<button className="dismiss-error" aria-label="Fechar aviso" onClick={()=>setError('')}>×</button></div>}
    <section className="app-options" aria-label="Configurações gerais">
      <div><strong>Aplicativo</strong><small>Fechar ou minimizar mantém os widgets ativos. Use a bandeja para abrir ou sair.</small></div>
      <label className="toggle"><input type="checkbox" checked={startup===true} disabled={startup===null||appBusy} onChange={e=>{const enabled=e.target.checked;void applicationAction(async()=>setStartup(await invoke<boolean>('set_startup',{enabled})))}}/> Iniciar com o Windows</label>
      <button className="widget-toggle" disabled={appBusy||saving} onClick={()=>void applicationAction(()=>invoke('hide_panel'))}>Minimizar para bandeja</button>
    </section>
    <div className="library-layout"><aside className="widget-library"><div className="library-label">Sua coleção <span>{layout?Object.values(layout.widgets).filter(s=>s.enabled).length:0} no desktop</span></div>
      {widgetIds.map(id=><article key={id} className={`widget-card ${selected===id?'selected':''}`}><button className="widget-pick" aria-pressed={selected===id} onClick={()=>setSelected(id)}><span className="catalog-icon"><WidgetIcon id={id}/></span><span><strong>{catalog[id].name}</strong><small>{catalog[id].description}</small></span></button><div className="widget-card-bottom"><span>{layout?.widgets[id].enabled?'No desktop':catalog[id].tag}</span><button className="widget-toggle" disabled={!layout||busy!==null} onClick={()=>toggle(id)}>{busy===id?'Aguarde…':layout?.widgets[id].enabled?'Remover':'+ Adicionar'}</button></div></article>)}
      <p className="library-footnote">Cada widget guarda suas próprias configurações. Os que estiverem ativos serão reabertos ao iniciar o app.</p>
    </aside><section className="widget-inspector" aria-label="Configurações do widget selecionado">
      <div className="inspector-heading"><div><span className="small">Personalizar</span><h2>{meta.name}</h2></div><span className={`widget-state ${settings?.enabled?'active':''}`}>{settings?.enabled?'Ativo':'Oculto'}</span></div>
      {!settings?<p className="mode-help">{isTauri()?'Carregando preferências…':'O painel precisa da conexão nativa.'}</p>:<>
        <section className="inspector-section"><div className="section-title"><h3>Posição e tamanho</h3><label className="toggle"><input type="checkbox" checked={settings.edit} onChange={e=>patch({edit:e.target.checked})}/> Permitir arrastar</label></div><p className="mode-help">Arraste pelo fundo ou pelo título do widget. Botões e campos continuam interativos.</p><div className="dimension-grid">
          <NumberField label="X · pixels" value={settings.x} min={-1000000} max={1000000} onCommit={x=>patch({x})}/><NumberField label="Y · pixels" value={settings.y} min={-1000000} max={1000000} onCommit={y=>patch({y})}/>
          <NumberField label="Largura" value={settings.width} min={200} max={640} onCommit={width=>patch({width})}/><NumberField label="Altura" value={settings.height} min={180} max={600} onCommit={height=>patch({height})}/>
        </div><button className="text-action" onClick={()=>patch({width:meta.size[0],height:meta.size[1]})}>Usar tamanho sugerido</button></section>
        {(selected==='clock'||selected==='analog')&&<section className="inspector-section"><h3>Relógio</h3><div className="option-row"><label className="toggle"><input type="checkbox" checked={settings.show_seconds} onChange={e=>patch({show_seconds:e.target.checked})}/> Mostrar segundos</label>{selected==='clock'&&<label className="toggle"><input type="checkbox" checked={settings.hour24} onChange={e=>patch({hour24:e.target.checked})}/> Formato 24 horas</label>}</div></section>}
        {selected==='notes'&&<p className="mode-help contextual-note">Digite na nota diretamente no desktop. O texto é salvo localmente a cada alteração. Limite: 8.000 caracteres.</p>}
        {selected==='system'&&<p className="mode-help contextual-note">CPU, RAM e rede são lidas no Windows a cada 2 segundos. Os gráficos começam vazios e guardam até 30 amostras reais. A rede soma interfaces, incluindo VPNs.</p>}
        <section className="inspector-section"><h3>Material</h3>
          <label className="toggle"><input type="checkbox" checked={layout?.sync_theme??false} disabled={appBusy||saving} onChange={e=>{const enabled=e.target.checked;void applicationAction(()=>invoke('set_theme_sync',{id:selected,enabled}))}}/> Sincronizar tema de todos os widgets</label>
          <p className="mode-help">Ao ativar, o material deste widget vale para todos, inclusive os ocultos. Depois, qualquer mudança de cor ou efeito será compartilhada.</p>
          <div className="theme-presets">{[
            {name:'Cristal',tint:'#b9d5e8',opacity:.12,shine:.65,shade:.42},
            {name:'Grafite',tint:'#627080',opacity:.18,shine:.45,shade:.65},
            {name:'Esmeralda',tint:'#98c7b4',opacity:.25,shine:.7,shade:.45},
            {name:'Âmbar',tint:'#e8baa5',opacity:.25,shine:.8,shade:.42}
          ].map(({name,...material})=><button className="widget-toggle" key={name} onClick={()=>patch(material)}>{name}</button>)}</div><div className="color-control"><label htmlFor="glass-color">Cor</label><input id="glass-color" type="color" value={settings.tint} onChange={e=>patch({tint:e.target.value})}/><span>{settings.tint.toUpperCase()}</span><div className="swatches">{['#b9d5e8','#ffffff','#98c7b4','#e8baa5','#a5a8e8','#e38585'].map(tint=><button key={tint} aria-label={`Cor ${tint}`} title={tint} style={{background:tint}} onClick={()=>patch({tint})}/>)}</div></div>
          <Range label="Intensidade da cor" value={settings.opacity} max={.6} onChange={opacity=>patch({opacity})}/><Range label="Reflexos e espessura" value={settings.shine} onChange={shine=>patch({shine})}/><Range label="Contraste de leitura" value={settings.shade} min={.2} max={.75} onChange={shade=>patch({shade})}/>
          <label className="toggle wallpaper-mode"><input type="checkbox" checked={!settings.static_wallpaper} onChange={e=>patch({static_wallpaper:!e.target.checked})}/> Fundo animado / Wallpaper Engine</label>
          <Range label="Refração estática" value={settings.refraction} max={40} step={1} suffix=" px" disabled={!settings.static_wallpaper} onChange={refraction=>patch({refraction})}/>
          <p className="mode-help">{settings.static_wallpaper?'A lente usa a imagem estática configurada no Windows.':'Transparência sobre o fundo animado, sem copiar uma imagem antiga.'}</p>
        </section><footer className="inspector-footer"><span className="status-dot"/>{saving?'Salvando…':'Preferências salvas neste computador'}</footer>
      </>}
    </section></div>
  </main>;
}
