use crate::{handle, media, monitor, native, persist, State};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub static_wallpaper: bool,
    pub monitor: usize,
    pub tint: String,
    pub opacity: f64,
    pub above_apps: bool,
    pub hide_fullscreen: bool,
    pub show_seconds: bool,
    pub show_date: bool,
    pub show_cpu: bool,
    pub show_memory: bool,
    pub show_network: bool,
    pub show_volume: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self { static_wallpaper:false, enabled:false, monitor:0, tint:"#18232d".into(), opacity:0.82,
            above_apps:true, hide_fullscreen:true, show_seconds:false, show_date:true,
            show_cpu:true, show_memory:true, show_network:true, show_volume:true }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(),String> {
        if self.monitor > 15 || !self.opacity.is_finite() || !(0.35..=1.0).contains(&self.opacity)
            || self.tint.len()!=7 || !self.tint.starts_with('#') || !self.tint[1..].bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Configuração da Top Bar inválida".into());
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct Runtime {
    pub lifecycle: Mutex<()>,
    placement: Mutex<Option<[i32;6]>>,
}
fn allowed(window:&tauri::WebviewWindow) -> Result<(),String> {
    if window.label()=="topbar" { Ok(()) } else { Err("Comando reservado à Top Bar".into()) }
}
fn effective(index:usize,count:usize)->usize { if index<count {index} else {0} }
fn shared_monitor(layout:&crate::Layout,count:usize)->bool {
    layout.island.enabled && effective(layout.island.monitor,count)==effective(layout.topbar.monitor,count)
}
// Physical coordinates, including negative monitor origins and mixed DPI.
fn geometry(origin:[i32;2],width:u32,scale:f64,shared:bool,above:bool)->[i32;6] {
    let margin=(8.0*scale).round() as i32;
    let w=(width as i32-2*margin).max(1);
    [origin[0]+margin,origin[1]+margin,w,(36.0*scale).round() as i32,
        if shared {(480.0*scale).round() as i32} else {0},i32::from(above)]
}
#[derive(Serialize)]
pub struct Snapshot {
    config:Config,
    shared_island:bool,
    system:Option<monitor::Snapshot>,
    volume:Option<media::Volume>,
    volume_error:Option<String>,
}
#[tauri::command]
pub async fn topbar_snapshot(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Snapshot,String> {
    allowed(&window)?;
    let count=window.available_monitors().map_err(|e|e.to_string())?.len();
    tauri::async_runtime::spawn_blocking(move || {
        let state=app.state::<State>();
        let layout=state.layout.lock().unwrap().clone();
        let shared_island=shared_monitor(&layout,count);
        let config=layout.topbar;
        let system=if config.show_cpu||config.show_memory||config.show_network {
            Some(state.monitor.lock().unwrap().get_or_insert_with(monitor::Monitor::new).sample())
        } else {None};
        let volume=if config.show_volume {
            media::Apartment::new().and_then(|_apartment|media::volume(None,None)).map(Some)
        } else {Ok(None)};
        Ok(Snapshot{config,shared_island,system,volume:volume.as_ref().ok().cloned().flatten(),volume_error:volume.err()})
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn topbar_volume(window:tauri::WebviewWindow,level:Option<f32>,muted:Option<bool>)->Result<media::Volume,String> {
    allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _apartment=media::Apartment::new()?;
        media::volume(level,muted)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn topbar_open_panel(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<(),String> {
    allowed(&window)?;
    crate::show_panel(&app);
    Ok(())
}

// UI-thread only. An actual hole in the HWND leaves the island and desktop clickable.
pub fn maintain(app:&tauri::AppHandle)->Result<(),String> {
    crate::workarea::refresh(app);
    let Some(window)=app.get_webview_window("topbar") else {return Ok(())};
    let layout=app.state::<State>().layout.lock().unwrap().clone();
    let config=&layout.topbar;
    let monitors=window.available_monitors().map_err(|e|e.to_string())?;
    let m=monitors.get(effective(config.monitor,monitors.len())).ok_or("Nenhum monitor disponível")?;
    let p=m.position();let size=m.size();let scale=m.scale_factor();
    let desired=geometry([p.x,p.y+crate::workarea::offset(p.x,p.y)],size.width,scale,shared_monitor(&layout,monitors.len()),config.above_apps);
    let runtime=app.state::<Runtime>();
    let mut previous=runtime.placement.lock().unwrap();
    let hwnd=handle(&window)?;
    if previous.as_ref()!=Some(&desired)||native::rect(hwnd)?!=desired[..4] {
        native::position_surface(hwnd,desired[0],desired[1],desired[2],desired[3],config.above_apps)?;
        native::bar_region(hwnd,desired[2],desired[3],desired[4],(14.0*scale).round() as i32)?;
        *previous=Some(desired);
    }
    let hidden=config.hide_fullscreen&&native::foreground_fullscreen([p.x,p.y,size.width as i32,size.height as i32]);
    if hidden { if window.is_visible().unwrap_or(false){window.hide().map_err(|e|e.to_string())?;} }
    else if !window.is_visible().unwrap_or(true){window.show().map_err(|e|e.to_string())?;}
    Ok(())
}
// Window creation runs on a worker, with all native setup dispatched to its UI thread.
pub fn create(app:&tauri::AppHandle)->Result<(),String> {
    if app.get_webview_window("topbar").is_some()||!app.state::<State>().layout.lock().unwrap().topbar.enabled{return Ok(())}
    let window=tauri::WebviewWindowBuilder::new(app,"topbar",tauri::WebviewUrl::App("index.html?surface=topbar".into()))
        .title("Liquid Top Bar").inner_size(800.0,36.0).transparent(true).decorations(false)
        .shadow(false).resizable(false).skip_taskbar(true).visible(false).focused(false).build().map_err(|e|e.to_string())?;
    let copy=app.clone();let w=window.clone();let(tx,rx)=std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let result=(||{
            native::configure_desktop_window(handle(&w)?)?;
            *copy.state::<Runtime>().placement.lock().unwrap()=None;
            maintain(&copy)
        })();
        let _=tx.send(result);
    }).map_err(|e|e.to_string())?;
    if let Err(error)=rx.recv().map_err(|e|e.to_string())? {let _=window.close();return Err(error)}
    Ok(())
}
#[tauri::command]
pub async fn update_topbar(app:tauri::AppHandle,window:tauri::WebviewWindow,config:Config)->Result<(),String> {
    if window.label()!="main"{return Err("Use o painel".into())}
    config.validate()?;
    tauri::async_runtime::spawn_blocking(move || {
        let runtime=app.state::<Runtime>();let _lock=runtime.lifecycle.lock().unwrap();
        let state=app.state::<State>();
        let old={let mut layout=state.layout.lock().unwrap();let old=layout.topbar.clone();
            let mut next=layout.clone();next.topbar=config.clone();persist(&app,&next)?;*layout=next;old};
        let result=(||{
            if config.enabled {
                create(&app)?;
                let copy=app.clone();let(tx,rx)=std::sync::mpsc::channel();
                app.run_on_main_thread(move ||{let _=tx.send(maintain(&copy));}).map_err(|e|e.to_string())?;
                rx.recv().map_err(|e|e.to_string())??;
            } else if let Some(w)=app.get_webview_window("topbar"){w.close().map_err(|e|e.to_string())?;}
            Ok::<(),String>(())
        })();
        if let Err(error)=result {
            let mut layout=state.layout.lock().unwrap();layout.topbar=old;persist(&app,&layout)?;
            return Err(error);
        }
        let copy=app.clone();
        app.run_on_main_thread(move ||crate::workarea::refresh(&copy)).map_err(|e|e.to_string())?;
        app.emit_to("topbar","topbar-refresh",()).map_err(|e|e.to_string())?;
        let layout=state.layout.lock().unwrap().clone();
        app.emit_to("main","layout",layout).map_err(|e|e.to_string())
    }).await.map_err(|e|e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrates_layout_and_rejects_invalid_material() {
        let mut layout=crate::Layout::load(br##"{"version":2,"widgets":{"notes":{"note":"keep","x":-200}},"island":{"enabled":true}}"##).unwrap();
        assert!(!layout.topbar.enabled);assert!(layout.island.enabled);
        layout.topbar.enabled=true;layout.topbar.tint="#aabbcc".into();layout.topbar.opacity=0.55;
        let saved=crate::Layout::load(&serde_json::to_vec(&layout).unwrap()).unwrap();
        assert_eq!(saved.widgets["notes"].note,"keep");assert_eq!(saved.widgets["notes"].x,-200);
        assert_eq!(saved.topbar.opacity,0.55);assert!(saved.topbar.enabled);
        for color in ["red","#gggggg","#abc","é12345"] {layout.topbar.tint=color.into();assert!(layout.topbar.validate().is_err());}
        layout.topbar=Config::default();layout.topbar.opacity=f64::NAN;assert!(layout.topbar.validate().is_err());
        layout.topbar=Config::default();layout.topbar.monitor=16;assert!(layout.topbar.validate().is_err());
    }
    #[test]
    fn gap_tracks_effective_monitors_and_geometry_uses_physical_coordinates() {
        let mut layout=crate::Layout::default();layout.island.enabled=true;
        assert!(shared_monitor(&layout,2));layout.island.monitor=1;assert!(!shared_monitor(&layout,2));
        assert!(shared_monitor(&layout,1));
        assert_eq!(geometry([-1920,-200],1920,1.5,true,true),[-1908,-188,1896,54,720,1]);
        assert_eq!(geometry([0,0],1920,1.0,false,false),[8,8,1904,36,0,0]);
    }
}
