import {useCallback,useEffect,useRef,useState} from 'react';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {widgetIds,type Layout,type Settings,type WidgetId} from '../types';
import {catalog,WidgetIcon} from '../widgets/catalog';
import {materials,materialPatch} from '../glass/materials';
import {MaterialPreview} from '../glass/MaterialPreview';
import {IslandSettings} from '../island/IslandSettings';
import {TopBarSettings} from '../topbar/TopBarSettings';
import {FusionSettings} from '../fusion/FusionSettings';
import {previewLayout} from './preview';

function NumberField({label,value,min,max,onCommit}:{label:string;value:number;min:number;max:number;onCommit:(v:number)=>void}) {
  const [draft,setDraft]=useState(String(value));
  useEffect(()=>setDraft(String(value)),[value]);
  function commit(){const n=Number(draft);if(Number.isFinite(n)&&n>=min&&n<=max&&Math.round(n)!==value)onCommit(Math.round(n));else setDraft(String(value))}
  return <label className="number-field">{label}<input type="number" min={min} max={max} value={draft} onChange={e=>setDraft(e.target.value)} onBlur={commit} onKeyDown={e=>{if(e.key==='Enter')e.currentTarget.blur()}}/></label>;
}
function Range({label,value,min=0,max=1,step=.01,suffix='%',disabled=false,onChange}:{label:string;value:number;min?:number;max?:number;step?:number;suffix?:string;disabled?:boolean;onChange:(v:number)=>void}) {
  return <label className="slider-label">{label}<output>{suffix==='%'?Math.round(value*100):value.toFixed(0)}{suffix}</output><input type="range" disabled={disabled} min={min} max={max} step={step} value={value} onChange={e=>onChange(+e.target.value)}/></label>;
}
function MaterialStudio({settings,layout,selected,onSelect,patch,applicationAction,appBusy,saving}:{settings:Settings;layout:Layout;selected:WidgetId;onSelect:(id:WidgetId)=>void;patch:(values:Partial<Settings>)=>void;applicationAction:(action:()=>Promise<unknown>)=>Promise<void>;appBusy:boolean;saving:boolean}){
  return <div className="studio-layout"><aside className="studio-library"><label className="studio-target">Editar material de<select value={selected} onChange={e=>onSelect(e.target.value as WidgetId)}>{widgetIds.map(id=><option key={id} value={id}>{catalog[id].name}</option>)}</select></label><h2>Biblioteca</h2><p>Escolha uma base para a lente</p><div className="material-presets">{Object.entries(materials).map(([id,material])=><button key={id} aria-pressed={settings.material===id} title={material.description} onClick={()=>patch(materialPatch(id as keyof typeof materials))}><strong>{material.name}</strong><small>{material.description}</small></button>)}</div></aside><div className="studio-main"><div className="studio-preview"><div className="studio-preview-heading"><div><h2>Prévia do material</h2><p>Shader real sobre textura de teste</p></div><span>{settings.material==='custom'?'Personalizado':materials[settings.material as keyof typeof materials]?.name??settings.material}</span></div><MaterialPreview settings={settings}/></div><div className="studio-controls"><section className="inspector-section material-controls"><h3>Ajustes do material</h3>
          <label className="toggle"><input type="checkbox" checked={layout?.sync_theme??false} disabled={appBusy||saving} onChange={e=>{const enabled=e.target.checked;void applicationAction(()=>invoke('set_theme_sync',{id:selected,enabled}))}}/> Sincronizar vidro dos widgets, barra e ilha</label>
          <p className="mode-help">Ao ativar, todos usam este material. Barra e ilha seguem o último material ajustado; posições e conteúdo continuam individuais.</p>
          <div className="theme-presets">{[
            {name:'Cristal',tint:'#b9d5e8',opacity:.12,shine:.65,shade:.42},
            {name:'Grafite',tint:'#627080',opacity:.18,shine:.45,shade:.65},
            {name:'Esmeralda',tint:'#98c7b4',opacity:.25,shine:.7,shade:.45},
            {name:'Âmbar',tint:'#e8baa5',opacity:.25,shine:.8,shade:.42}
          ].map(({name,...material})=><button className="widget-toggle" key={name} onClick={()=>patch(material)}>{name}</button>)}</div><div className="color-control"><label htmlFor="glass-color">Cor</label><input id="glass-color" type="color" value={settings.tint} onChange={e=>patch({tint:e.target.value})}/><span>{settings.tint.toUpperCase()}</span><div className="swatches">{['#b9d5e8','#ffffff','#98c7b4','#e8baa5','#a5a8e8','#e38585'].map(tint=><button key={tint} aria-label={`Cor ${tint}`} title={tint} style={{background:tint}} onClick={()=>patch({tint})}/>)}</div></div>
          <div className="studio-group"><h4>Aparência</h4><Range label="Intensidade da cor" value={settings.opacity} max={.6} onChange={opacity=>patch({opacity})}/><Range label="Contraste de leitura" value={settings.shade} min={.2} max={.75} onChange={shade=>patch({shade})}/><Range label="Difusão da textura" value={settings.frost} max={8} step={.25} suffix=" px" onChange={frost=>patch({frost})}/></div>
          <div className="studio-group"><h4>Óptica</h4>
          <Range label="Espessura óptica" value={settings.thickness} min={4} max={40} step={1} suffix=" px" onChange={thickness=>patch({thickness,material:'custom'})}/>
          <Range label="Largura da borda convexa" value={settings.bezel} min={8} max={48} step={1} suffix=" px" onChange={bezel=>patch({bezel,material:'custom'})}/>
          <Range label="Separação cromática" value={settings.dispersion} max={.5} onChange={dispersion=>patch({dispersion})}/>
          <Range label="Refração" value={settings.refraction} max={40} step={1} suffix=" px" onChange={refraction=>patch({refraction})}/></div>
          <div className="studio-group"><h4>Iluminação</h4><Range label="Reflexos especulares" value={settings.shine} onChange={shine=>patch({shine})}/></div>
          <div className="studio-group"><h4>Comportamento</h4>
          <label className="quality-control">Qualidade óptica<select value={settings.quality} onChange={e=>patch({quality:e.target.value as Settings['quality']})}><option value="economy">Econômica · sem dispersão</option><option value="balanced">Equilibrada</option><option value="high">Alta</option></select></label>
          <label className="toggle"><input type="checkbox" checked={settings.motion} onChange={e=>patch({motion:e.target.checked})}/> Resposta elástica ao toque</label>
          <label className="toggle wallpaper-mode"><input type="checkbox" checked={!settings.static_wallpaper} onChange={e=>patch({static_wallpaper:!e.target.checked})}/> Fundo animado / Wallpaper Engine</label>
          <p className="mode-help">{settings.static_wallpaper?'A lente usa a imagem estática configurada no Windows.':'Refração dos frames do Wallpaper Engine em tempo real. Os recortes ficam em memória; áreas de aplicativos são mascaradas.'}</p>
          {settings.material==='adaptive'&&<p className="mode-help">{settings.static_wallpaper?'O contraste acompanha a luminosidade da região atrás deste widget.':'No fundo animado, ajuste Contraste para manter o texto legível.'}</p>}</div>
        </section></div></div></div>;
}
export function Panel() {
  const [layout,setLayout]=useState<Layout|null>(null),[selected,setSelected]=useState<WidgetId>('clock'),[error,setError]=useState(''),[busy,setBusy]=useState<WidgetId|null>(null),[saving,setSaving]=useState(false);
  const [startup,setStartup]=useState<boolean|null>(null),[appBusy,setAppBusy]=useState(false);
  const [page,setPage]=useState<'overview'|'widgets'|'island'|'topbar'|'materials'|'appearance'|'app'>(()=>{const value=new URLSearchParams(location.search).get('page');return ['overview','widgets','island','topbar','materials','appearance','app'].includes(value??'')?value as 'overview'|'widgets'|'island'|'topbar'|'materials'|'appearance'|'app':'overview'});
  const queue=useRef(Promise.resolve()),pending=useRef(0);
  const reload=useCallback(()=>invoke<Layout>('get_layout').then(setLayout),[]);
  useEffect(()=>{
    if(!isTauri()){if(import.meta.env.DEV&&new URLSearchParams(location.search).has('preview-panel')){setLayout(previewLayout);return}setError('Abra pelo aplicativo Tauri para configurar os widgets nativos.');return}
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
  const pages=[['overview','Visão geral'],['widgets','Widgets'],['island','Dynamic Island'],['topbar','Top Bar'],['materials','Materiais'],['appearance','Aparência'],['app','Aplicativo']] as const;
  const pageInfo={overview:['Visão geral','O que está ativo no seu desktop'],widgets:['Widgets','Organize e configure cada elemento'],island:['Dynamic Island','Mídia e controles no topo da tela'],topbar:['Top Bar','Informações essenciais sempre visíveis'],materials:['Material Studio','Cor, refração e comportamento da lente'],appearance:['Aparência','Como os widgets se combinam'],app:['Aplicativo','Inicialização e acesso pela bandeja']} as const;
  return <main className="library-panel"><aside className="panel-sidebar" aria-label="Navegação"><div className="panel-brand"><span className="brandmark" aria-hidden="true">◌</span><span>Liquid Desktop</span></div><nav>{pages.map(([id,name])=><button key={id} className={page===id?'active':''} aria-current={page===id?'page':undefined} onClick={()=>setPage(id)}>{name}</button>)}</nav><div className="sidebar-foot"><span className="status-dot"/> {layout?Object.values(layout.widgets).filter(w=>w.enabled).length+' widgets ativos':'Carregando'}<small>v0.9.0</small></div></aside><div className="panel-main"><header className="panel-page-header"><div><p className="eyebrow">Liquid Desktop / {pageInfo[page][0]}</p><h1>{pageInfo[page][0]}</h1><p className="intro">{pageInfo[page][1]}</p></div><span className="save-status"><span className="status-dot"/>{saving?'Salvando…':'Atualizado'}</span></header>
    {error&&<div className="error" role="alert">{error}<button className="dismiss-error" aria-label="Fechar aviso" onClick={()=>setError('')}>×</button></div>}
    {page==='overview'&&<div className="overview-layout"><section className="overview-lead"><div><p className="eyebrow">Desktop em execução</p><h2>{layout?Object.values(layout.widgets).filter(w=>w.enabled).length:'–'} widgets ativos</h2><p>Controle os elementos do seu espaço em um só lugar.</p><div className="overview-actions"><button onClick={()=>setPage('widgets')}>Gerenciar widgets</button><button className="quiet" onClick={()=>setPage('materials')}>Editar material</button></div></div>{settings&&<MaterialPreview settings={settings}/>}</section><div className="overview-list"><button onClick={()=>setPage('island')}><span>Dynamic Island</span><strong>{layout?.island.enabled?'Ativa':'Desativada'}</strong></button><button onClick={()=>setPage('topbar')}><span>Top Bar</span><strong>{layout?.topbar.enabled?'Ativa':'Desativada'}</strong></button><button onClick={()=>setPage('materials')}><span>Material</span><strong>{settings?.material==='custom'?'Personalizado':settings?.material??'—'}</strong></button><button onClick={()=>setPage('materials')}><span>Fundo</span><strong>{settings?.static_wallpaper?'Estático':'Wallpaper Engine'}</strong></button></div></div>}
    {page==='app'&&<section className="app-options" aria-label="Configurações gerais">
      <div><strong>Aplicativo</strong><small>Fechar ou minimizar mantém os widgets ativos. Use a bandeja para abrir ou sair.</small></div>
      <label className="toggle"><input type="checkbox" checked={startup===true} disabled={startup===null||appBusy} onChange={e=>{const enabled=e.target.checked;void applicationAction(async()=>setStartup(await invoke<boolean>('set_startup',{enabled})))}}/> Iniciar com o Windows</label>
      <button className="widget-toggle" disabled={appBusy||saving} onClick={()=>void applicationAction(()=>invoke('hide_panel'))}>Minimizar para bandeja</button>
    </section>}
    {page==='island'&&layout&&<IslandSettings config={layout.island} onError={setError}/>}
    {page==='topbar'&&layout&&<TopBarSettings config={layout.topbar} onError={setError}/>}
    {page==='appearance'&&layout&&<FusionSettings config={layout.fusion} selected={selected} onError={setError}/>}
    {page==='materials'&&settings&&layout&&<MaterialStudio settings={settings} layout={layout} selected={selected} onSelect={setSelected} patch={patch} applicationAction={applicationAction} appBusy={appBusy} saving={saving}/>}
    {page==='widgets'&&    <div className="library-layout"><aside className="widget-library"><div className="library-label">Sua coleção <span>{layout?Object.values(layout.widgets).filter(s=>s.enabled).length:0} no desktop</span></div>
      {widgetIds.map(id=><article key={id} className={`widget-card ${selected===id?'selected':''}`}><button className="widget-pick" aria-pressed={selected===id} onClick={()=>setSelected(id)}><span className="catalog-icon"><WidgetIcon id={id}/></span><span><strong>{catalog[id].name}</strong><small>{catalog[id].description}</small></span></button><div className="widget-card-bottom"><span>{layout?.widgets[id].enabled?'No desktop':catalog[id].tag}</span><button className="widget-toggle" disabled={!layout||busy!==null} onClick={()=>toggle(id)}>{busy===id?'Aguarde…':layout?.widgets[id].enabled?'Remover':'+ Adicionar'}</button></div></article>)}
      <p className="library-footnote">Cada widget guarda suas próprias configurações. Os que estiverem ativos serão reabertos ao iniciar o app.</p>
    </aside><section className="widget-inspector" aria-label="Configurações do widget selecionado">
      <div className="inspector-heading"><div><span className="small">Personalizar</span><h2>{meta.name}</h2></div><span className={`widget-state ${settings?.enabled?'active':''}`}>{settings?.enabled?'Ativo':'Oculto'}</span></div>
      {!settings?<p className="mode-help">{isTauri()?'Carregando preferências…':'O painel precisa da conexão nativa.'}</p>:<>
        <MaterialPreview settings={settings}/>
        <section className="inspector-section"><div className="section-title"><h3>Posição e tamanho</h3><label className="toggle"><input type="checkbox" checked={settings.edit} onChange={e=>patch({edit:e.target.checked})}/> Permitir arrastar</label></div><p className="mode-help">Arraste pelo fundo ou pelo título do widget. Botões e campos continuam interativos.</p><div className="dimension-grid">
          <NumberField label="X · pixels" value={settings.x} min={-1000000} max={1000000} onCommit={x=>patch({x})}/><NumberField label="Y · pixels" value={settings.y} min={-1000000} max={1000000} onCommit={y=>patch({y})}/>
          <NumberField label="Largura" value={settings.width} min={200} max={640} onCommit={width=>patch({width})}/><NumberField label="Altura" value={settings.height} min={180} max={600} onCommit={height=>patch({height})}/>
        </div><button className="text-action" onClick={()=>patch({width:meta.size[0],height:meta.size[1]})}>Usar tamanho sugerido</button></section>
        {(selected==='clock'||selected==='analog')&&<section className="inspector-section"><h3>Relógio</h3><div className="option-row"><label className="toggle"><input type="checkbox" checked={settings.show_seconds} onChange={e=>patch({show_seconds:e.target.checked})}/> Mostrar segundos</label>{selected==='clock'&&<label className="toggle"><input type="checkbox" checked={settings.hour24} onChange={e=>patch({hour24:e.target.checked})}/> Formato 24 horas</label>}</div></section>}
        {selected==='notes'&&<p className="mode-help contextual-note">Digite na nota diretamente no desktop. O texto é salvo localmente a cada alteração. Limite: 8.000 caracteres.</p>}
        {selected==='system'&&<p className="mode-help contextual-note">CPU, RAM e rede são lidas no Windows a cada 2 segundos. Os gráficos começam vazios e guardam até 30 amostras reais. A rede soma interfaces, incluindo VPNs.</p>}
<footer className="inspector-footer"><span className="status-dot"/>{saving?'Salvando…':'Preferências salvas neste computador'}</footer>
      </>}
    </section></div>}
  </div></main>;
}
