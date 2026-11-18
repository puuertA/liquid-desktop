use crate::{handle,native,persist,settings::{Settings,IDS},State};
use serde::{Serialize,Deserialize};
use std::{collections::BTreeMap,sync::Mutex,time::Instant};
use tauri::{Manager,Emitter};
#[derive(Clone,Serialize,Deserialize)]
#[serde(default)]
pub struct Config {pub enabled:bool,pub monitor:usize,pub source:String,pub positions:BTreeMap<String,[f64;2]>}
impl Default for Config {fn default()->Self{Self{enabled:false,monitor:0,source:"clock".into(),positions:BTreeMap::new()}}}
impl Config {pub fn validate(&self)->Result<(),String>{
    crate::settings::valid_id(&self.source)?;
    if self.monitor>15||self.positions.len()>5{return Err("Grupo inválido".into())}
    for(id,p)in &self.positions{crate::settings::valid_id(id)?;if p.iter().any(|v|!v.is_finite()||!(0.0..=16000.0).contains(v)){return Err("Posição do grupo inválida".into())}}
    Ok(())
}}
#[derive(Default)]
pub struct Runtime {pub lifecycle:Mutex<()>,preview:Mutex<Option<(Vec<Shape>,Instant)>>}
#[derive(Clone,Serialize,Deserialize)]
pub struct Shape {pub id:String,pub x:f64,pub y:f64,pub width:f64,pub height:f64}
#[derive(Serialize)]
pub struct Snapshot {pub config:Config,pub widgets:BTreeMap<String,Settings>,pub shapes:Vec<Shape>,pub screen:[i32;4],pub scale:f64,pub width:f64,pub height:f64}
fn allowed(w:&tauri::WebviewWindow)->Result<(),String>{if w.label()=="fusion"{Ok(())}else{Err("Use o grupo líquido".into())}}
fn snapshot(app:&tauri::AppHandle)->Result<Snapshot,String>{
    let layout=app.state::<State>().layout.lock().unwrap().clone();
    let w=app.get_webview_window("fusion").or_else(||app.get_webview_window("main")).ok_or("Janela indisponível")?;
    let monitors=w.available_monitors().map_err(|e|e.to_string())?;
    let m=monitors.get(layout.fusion.monitor).or_else(||monitors.first()).ok_or("Monitor indisponível")?;
    let scale=m.scale_factor();let width=m.size().width as f64/scale;let height=m.size().height as f64/scale;
    let mut shapes=Vec::new();let mut row_y=100.0;let mut row_height=0.0_f64;let mut left_width=0.0;
    for id in IDS{let s=&layout.widgets[id];if !s.enabled{continue}
        let i=shapes.len();if i>0&&i%2==0{row_y+=row_height+16.0;row_height=0.0;}
        let p=layout.fusion.positions.get(id).copied().unwrap_or([if i%2==0{24.0}else{24.0+left_width+16.0},row_y]);
        if i%2==0{left_width=s.width as f64;}row_height=row_height.max(s.height as f64);
        shapes.push(Shape{id:id.into(),x:p[0].clamp(0.0,(width-s.width as f64).max(0.0)),y:p[1].clamp(0.0,(height-s.height as f64).max(0.0)),width:s.width as f64,height:s.height as f64});
    }
    Ok(Snapshot{config:layout.fusion,widgets:layout.widgets,shapes,screen:[m.position().x,m.position().y,m.size().width as i32,m.size().height as i32],scale,width,height})
}
#[tauri::command]
pub fn fusion_snapshot(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Snapshot,String>{allowed(&window)?;snapshot(&app)}
pub fn distance(x:f64,y:f64,shapes:&[Shape])->f64 {
    let mut d=1e9_f64;
    for s in shapes {
        let r=32.0_f64.min(s.width/2.0).min(s.height/2.0);
        let qx=(x-s.x-s.width/2.0).abs()-(s.width/2.0-r);let qy=(y-s.y-s.height/2.0).abs()-(s.height/2.0-r);
        let next=qx.max(0.0).hypot(qy.max(0.0))+qx.max(qy).min(0.0)-r;
        let h=(1.0-(d-next).abs()/48.0).max(0.0);d=d.min(next)-h*h*12.0;
    }d
}
fn region(w:&tauri::WebviewWindow,s:&Snapshot,shapes:&[Shape])->Result<(),String>{
    let mut rows=Vec::new();
    if !shapes.is_empty(){
        let maxx=(shapes.iter().map(|r|r.x+r.width+16.0).fold(0.0,f64::max)*s.scale).ceil().min(s.screen[2] as f64) as i32;
        let minx=(shapes.iter().map(|r|r.x-16.0).fold(s.width,f64::min)*s.scale).floor().max(0.0) as i32;
        let miny=(shapes.iter().map(|r|r.y-16.0).fold(s.height,f64::min)*s.scale).floor().max(0.0) as i32;
        let maxy=(shapes.iter().map(|r|r.y+r.height+16.0).fold(0.0,f64::max)*s.scale).ceil().min(s.screen[3] as f64) as i32;
        // Conservative one-physical-pixel scanlines, only inside the group's bounds.
        for y in (miny..maxy).step_by(1){let mut start=None;
            for x in (minx..=maxx).step_by(1){let inside=x<maxx&&distance((x as f64+0.5)/s.scale,(y as f64+0.5)/s.scale,shapes)<=1.0/s.scale;
                if inside&&start.is_none(){start=Some(x)}
                if !inside{if let Some(left)=start.take(){rows.push([left,y,x,(y+1).min(s.screen[3])]);}}
            }
            if let Some(left)=start{rows.push([left,y,maxx,(y+1).min(s.screen[3])]);}
        }
    }
    native::scanline_region(handle(w)?,&rows)
}
#[tauri::command]
pub fn fusion_positions(app:tauri::AppHandle,window:tauri::WebviewWindow,positions:BTreeMap<String,[f64;2]>,commit:bool)->Result<(),String>{
    allowed(&window)?;let mut state=snapshot(&app)?;
    if positions.len()>5{return Err("Muitos widgets".into())}
    for(id,p)in &positions{
        let shape=state.shapes.iter_mut().find(|s|s.id==*id).ok_or("Widget inativo")?;
        if !state.widgets[id].edit{return Err("Ative Permitir arrastar no painel".into())}
        if p.iter().any(|v|!v.is_finite()){return Err("Posição inválida".into())}
        shape.x=p[0].clamp(0.0,(state.width-shape.width).max(0.0));shape.y=p[1].clamp(0.0,(state.height-shape.height).max(0.0));
    }
    if !commit&&!positions.is_empty(){
        // Pointer capture owns the gesture. Keep the canvas unclipped during movement;
        // restore its exact hit region on release or after an idle maintenance tick.
        native::scanline_region(handle(&window)?,&[[0,0,state.screen[2],state.screen[3]]])?;
        *app.state::<Runtime>().preview.lock().unwrap()=Some((state.shapes.clone(),Instant::now()));
    }else{
        *app.state::<Runtime>().preview.lock().unwrap()=None;
        region(&window,&state,&state.shapes)?;
    }
    if commit{
        let shared=app.state::<State>();let mut layout=shared.layout.lock().unwrap();let mut next=layout.clone();
        for shape in &state.shapes{if positions.contains_key(&shape.id){next.fusion.positions.insert(shape.id.clone(),[shape.x,shape.y]);}}
        persist(&app,&next)?;*layout=next.clone();drop(layout);app.emit_to("main","layout",next).map_err(|e|e.to_string())?;
    }Ok(())
}
pub fn maintain(app:&tauri::AppHandle)->Result<(),String>{
    let enabled=app.state::<State>().layout.lock().unwrap().fusion.enabled;
    let Some(w)=app.get_webview_window("fusion")else{
        // Fail open if the group window was closed: never leave all originals hidden.
        let layout=app.state::<State>().layout.lock().unwrap().clone();
        if enabled{for id in IDS{if layout.widgets[id].enabled{if let Some(w)=app.get_webview_window(id){if !w.is_visible().unwrap_or(true){let _=w.show();}}}}}
        return Ok(())
    };
    if !enabled{return Ok(())}
    let s=snapshot(app)?;
    let preview={let runtime=app.state::<Runtime>();let mut preview=runtime.preview.lock().unwrap();
        if preview.as_ref().is_some_and(|(_,at)|at.elapsed().as_millis()>200){preview.take().map(|(shapes,_)|shapes)}else{None}};
    if let Some(shapes)=preview{region(&w,&s,&shapes)?;}
    if native::rect(handle(&w)?)?!=s.screen{
        native::position_surface(handle(&w)?,s.screen[0],s.screen[1],s.screen[2],s.screen[3],false)?;
        region(&w,&s,&s.shapes)?;let _=app.emit_to("fusion","fusion-refresh",());
    }
    for id in IDS{if let Some(original)=app.get_webview_window(id){if original.is_visible().unwrap_or(false){let _=original.hide();}}}
    native::maintain_desktop_layer(handle(&w)?)?;Ok(())
}
pub fn create(app:&tauri::AppHandle)->Result<(),String>{
    if app.get_webview_window("fusion").is_some()||!app.state::<State>().layout.lock().unwrap().fusion.enabled{return Ok(())}
    let w=tauri::WebviewWindowBuilder::new(app,"fusion",tauri::WebviewUrl::App("index.html?surface=fusion".into())).title("Liquid widgets unidos")
        .inner_size(800.0,600.0).transparent(true).decorations(false).shadow(false).resizable(false).skip_taskbar(true).visible(false).build().map_err(|e|e.to_string())?;
    let copy=app.clone();let window=w.clone();let(tx,rx)=std::sync::mpsc::channel();
    app.run_on_main_thread(move||{let result=(||{native::configure_desktop_window(handle(&window)?)?;maintain(&copy)?;window.show().map_err(|e|e.to_string())})();let _=tx.send(result);}).map_err(|e|e.to_string())?;
    if let Err(e)=rx.recv().map_err(|e|e.to_string())?{let _=w.close();return Err(e)}Ok(())
}
#[tauri::command]
pub async fn update_fusion(app:tauri::AppHandle,window:tauri::WebviewWindow,enabled:bool,monitor:usize,source:String)->Result<(),String>{
    if window.label()!="main"{return Err("Use o painel".into())}
    tauri::async_runtime::spawn_blocking(move||{
        let runtime=app.state::<Runtime>();let _guard=runtime.lifecycle.lock().unwrap();let shared=app.state::<State>();
        let old={let mut layout=shared.layout.lock().unwrap();let old=layout.fusion.clone();let mut next=layout.clone();
            next.fusion.enabled=enabled;next.fusion.monitor=monitor;next.fusion.source=source;next.fusion.validate()?;persist(&app,&next)?;*layout=next;old};
        if let Err(e)=create(&app){let mut layout=shared.layout.lock().unwrap();layout.fusion=old;persist(&app,&layout)?;drop(layout);
            let copy=app.clone();let _=app.run_on_main_thread(move||{let layout=copy.state::<State>().layout.lock().unwrap().clone();for id in IDS{if layout.widgets[id].enabled{if let Some(w)=copy.get_webview_window(id){let _=w.show();}}}});return Err(e)}
        let copy=app.clone();let(tx,rx)=std::sync::mpsc::channel();
        app.run_on_main_thread(move||{let result=(||{
            if enabled{maintain(&copy)?;if let Some(w)=copy.get_webview_window("fusion"){let s=snapshot(&copy)?;region(&w,&s,&s.shapes)?;}}
            else{if let Some(w)=copy.get_webview_window("fusion"){w.close().map_err(|e|e.to_string())?;}
                let layout=copy.state::<State>().layout.lock().unwrap().clone();for id in IDS{if layout.widgets[id].enabled{if let Some(w)=copy.get_webview_window(id){let _=w.show();}}}}
            Ok::<(),String>(())})();let _=tx.send(result);}).map_err(|e|e.to_string())?;
        if let Err(error)=rx.recv().map_err(|e|e.to_string())?{
            let mut layout=shared.layout.lock().unwrap();layout.fusion=old;persist(&app,&layout)?;let restored=layout.clone();drop(layout);
            let copy=app.clone();let _=app.run_on_main_thread(move||{if restored.fusion.enabled{let _=maintain(&copy);}else{if let Some(w)=copy.get_webview_window("fusion"){let _=w.close();}for id in IDS{if restored.widgets[id].enabled{if let Some(w)=copy.get_webview_window(id){let _=w.show();}}}}});return Err(error)
        }
        app.emit_to("fusion","fusion-refresh",()).map_err(|e|e.to_string())?;
        let layout=shared.layout.lock().unwrap().clone();app.emit_to("main","layout",layout).map_err(|e|e.to_string())
    }).await.map_err(|e|e.to_string())?
}
#[cfg(test)]mod tests{use super::*;
    #[test]fn fusion_layout_migrates_without_replacing_individual_positions(){
        let mut layout=crate::settings::Layout::load(br#"{"version":2,"widgets":{"notes":{"note":"keep","x":230}}}"#).unwrap();
        assert!(!layout.fusion.enabled);layout.fusion.enabled=true;layout.fusion.positions.insert("notes".into(),[15.0,220.0]);
        let saved=crate::settings::Layout::load(&serde_json::to_vec(&layout).unwrap()).unwrap();
        assert_eq!(saved.widgets["notes"].x,230);assert_eq!(saved.widgets["notes"].note,"keep");assert_eq!(saved.fusion.positions["notes"],[15.0,220.0]);
        layout.fusion.positions.insert("not-a-widget".into(),[0.0,0.0]);assert!(layout.fusion.validate().is_err());
    }
    #[test]fn nearby_widgets_merge_but_distant_widgets_leave_a_real_gap(){
        let a=Shape{id:"clock".into(),x:0.0,y:0.0,width:200.0,height:200.0};let mut b=Shape{id:"notes".into(),x:216.0,..a.clone()};
        assert!(distance(208.0,100.0,&[a.clone(),b.clone()])<0.0);
        b.x=280.0;assert!(distance(240.0,100.0,&[a.clone(),b])>0.0);assert!(distance(-20.0,0.0,&[a])>0.0);
    }
}
