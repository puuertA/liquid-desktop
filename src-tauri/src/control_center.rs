use crate::{handle,media,native};use serde::Serialize;use tauri::{Manager,Emitter};use std::sync::Mutex;
use windows::{core::PWSTR,Win32::{Media::Audio::{*,Endpoints::*},System::Com::{*,StructuredStorage::*},Devices::FunctionDiscovery::PKEY_Device_FriendlyName},Networking::Connectivity::NetworkInformation};
#[derive(Default)]pub struct Runtime {pub lifecycle:Mutex<()>}
#[derive(Serialize)]pub struct Device {id:String,name:String,is_default:bool}
unsafe fn owned_string(value:PWSTR)->String{let text=value.to_string().unwrap_or_default();CoTaskMemFree(Some(value.0.cast()));text}
fn audio_devices()->Result<(Vec<Device>,Vec<IMMDevice>),String>{unsafe{
    let enumerator:IMMDeviceEnumerator=CoCreateInstance(&MMDeviceEnumerator,None,CLSCTX_ALL).map_err(|e|e.to_string())?;
    let default=enumerator.GetDefaultAudioEndpoint(eRender,eMultimedia).and_then(|d|d.GetId()).map(|v|owned_string(v)).unwrap_or_default();
    let collection=enumerator.EnumAudioEndpoints(eRender,DEVICE_STATE_ACTIVE).map_err(|e|e.to_string())?;
    let mut devices=Vec::new();let mut interfaces=Vec::new();
    for i in 0..collection.GetCount().map_err(|e|e.to_string())?.min(64){let device=collection.Item(i).map_err(|e|e.to_string())?;let id=owned_string(device.GetId().map_err(|e|e.to_string())?);
        let name=(||->windows::core::Result<String>{let store=device.OpenPropertyStore(STGM_READ)?;let mut property=store.GetValue(&PKEY_Device_FriendlyName)?;let result=PropVariantToStringAlloc(&property).map(|v|owned_string(v));let _=PropVariantClear(&mut property);result})().unwrap_or_else(|_|format!("Saída {}",i+1));
        devices.push(Device{is_default:id==default,id,name});interfaces.push(device);
    }Ok((devices,interfaces))
}}
fn endpoint_volume(endpoint:&IAudioEndpointVolume,level:Option<f32>,muted:Option<bool>)->Result<media::Volume,String>{unsafe{
    if let Some(value)=level{if !value.is_finite()||!(0.0..=1.0).contains(&value){return Err("Volume inválido".into())}endpoint.SetMasterVolumeLevelScalar(value,std::ptr::null()).map_err(|e|e.to_string())?;}
    if let Some(value)=muted{endpoint.SetMute(value,std::ptr::null()).map_err(|e|e.to_string())?;}
    Ok(media::Volume{level:endpoint.GetMasterVolumeLevelScalar().map_err(|e|e.to_string())?,muted:endpoint.GetMute().map_err(|e|e.to_string())?.as_bool()})
}}
fn allowed(w:&tauri::WebviewWindow)->Result<(),String>{if w.label()=="control-center"{Ok(())}else{Err("Use o Control Center".into())}}
#[repr(C)]#[derive(Default)]struct PowerRaw {ac:u8,flags:u8,percent:u8,saver:u8,remaining:u32,total:u32}
#[link(name="kernel32")]unsafe extern "system"{fn GetSystemPowerStatus(status:*mut PowerRaw)->i32;}
#[link(name="shell32")]unsafe extern "system"{fn ShellExecuteW(hwnd:isize,operation:*const u16,file:*const u16,parameters:*const u16,directory:*const u16,show:i32)->isize;}
#[derive(Serialize)]pub struct Power {ac:Option<bool>,percent:Option<u8>,present:Option<bool>,charging:bool,saver:bool}
fn power()->Option<Power>{let mut p=PowerRaw::default();if unsafe{GetSystemPowerStatus(&mut p)}==0{return None}
    Some(Power{ac:match p.ac{0=>Some(false),1=>Some(true),_=>None},percent:if p.percent<=100{Some(p.percent)}else{None},present:if p.flags==255{None}else{Some(p.flags&128==0)},charging:p.flags!=255&&p.flags&8!=0,saver:p.saver==1})}
#[derive(Serialize)]pub struct Snapshot {devices:Vec<Device>,selected:Option<String>,volume:Option<media::Volume>,audio_error:Option<String>,power:Option<Power>,network:String}
#[tauri::command]pub async fn center_snapshot(window:tauri::WebviewWindow,selected:Option<String>)->Result<Snapshot,String>{allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move||{
        let _apartment=media::Apartment::new()?;
        let network=NetworkInformation::GetInternetConnectionProfile().and_then(|p|Ok((p.ProfileName()?.to_string(),p.GetNetworkConnectivityLevel()?.0))).map(|(name,level)|format!("{} · {}",name,match level{3=>"Internet",2=>"Acesso limitado",1=>"Rede local",_=>"Sem conexão"})).unwrap_or_else(|_|"Sem conexão informada pelo Windows".into());
        let mut result=Snapshot{devices:Vec::new(),selected:None,volume:None,audio_error:None,power:power(),network};
        match audio_devices(){Ok((devices,interfaces))=>{
            let index=selected.as_ref().and_then(|id|devices.iter().position(|d|&d.id==id)).or_else(||devices.iter().position(|d|d.is_default)).or(if devices.is_empty(){None}else{Some(0)});
            if let Some(i)=index{result.selected=Some(devices[i].id.clone());match unsafe{interfaces[i].Activate::<IAudioEndpointVolume>(CLSCTX_ALL,None)}.map_err(|e|e.to_string()).and_then(|e|endpoint_volume(&e,None,None)){Ok(v)=>result.volume=Some(v),Err(e)=>result.audio_error=Some(e)}}
            result.devices=devices;
        },Err(e)=>result.audio_error=Some(e)}Ok(result)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]pub async fn center_volume(window:tauri::WebviewWindow,id:String,level:Option<f32>,muted:Option<bool>)->Result<media::Volume,String>{allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move||{let _apartment=media::Apartment::new()?;let(devices,interfaces)=audio_devices()?;let index=devices.iter().position(|d|d.id==id).ok_or("Saída desconectada")?;
        let endpoint=unsafe{interfaces[index].Activate::<IAudioEndpointVolume>(CLSCTX_ALL,None)}.map_err(|e|e.to_string())?;endpoint_volume(&endpoint,level,muted)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]pub fn center_hide(window:tauri::WebviewWindow)->Result<(),String>{allowed(&window)?;window.hide().map_err(|e|e.to_string())}
#[tauri::command]pub fn center_settings(window:tauri::WebviewWindow,page:String)->Result<(),String>{allowed(&window)?;
    let uri=match page.as_str(){"sound"=>"ms-settings:sound","network"=>"ms-settings:network-status","power"=>"ms-settings:powersleep",_=>return Err("Atalho desconhecido".into())};
    let file:Vec<u16>=uri.encode_utf16().chain(Some(0)).collect();let operation:Vec<u16>="open".encode_utf16().chain(Some(0)).collect();
    if unsafe{ShellExecuteW(handle(&window)?,operation.as_ptr(),file.as_ptr(),std::ptr::null(),std::ptr::null(),1)}<=32{return Err("Não foi possível abrir as Configurações do Windows".into())}Ok(())
}
#[tauri::command]pub async fn toggle_center(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<(),String>{
    if !["topbar","taskbar"].contains(&window.label()){return Err("Abra pela barra".into())}
    tauri::async_runtime::spawn_blocking(move||{
        let runtime=app.state::<Runtime>();let _lock=runtime.lifecycle.lock().unwrap();
        let center=if let Some(w)=app.get_webview_window("control-center"){w}else{tauri::WebviewWindowBuilder::new(&app,"control-center",tauri::WebviewUrl::App("index.html?surface=control-center".into())).title("Liquid Control Center").inner_size(360.0,440.0).transparent(true).decorations(false).shadow(false).skip_taskbar(true).resizable(false).visible(false).focused(false).build().map_err(|e|e.to_string())?};
        let copy=app.clone();let(tx,rx)=std::sync::mpsc::channel();
        app.run_on_main_thread(move||{let result=(||{
            if center.is_visible().unwrap_or(false){return center.hide().map_err(|e|e.to_string())}
            let scale=window.scale_factor().map_err(|e|e.to_string())?;let bar=native::rect(handle(&window)?)?;
            let monitor=window.current_monitor().map_err(|e|e.to_string())?.ok_or("Monitor indisponível")?;let p=monitor.position();let size=monitor.size();
            let width=((360.0*scale).round()as i32).min((size.width as i32-16).max(100));let height=((440.0*scale).round()as i32).min((size.height as i32-16).max(100));
            native::configure_desktop_window(handle(&center)?)?;
            let y=if window.label()=="taskbar"{let bar_height=copy.state::<crate::State>().layout.lock().unwrap().taskbar.height as f64;bar[1]+bar[3]-height-((bar_height+18.0)*scale)as i32}else{bar[1]+bar[3]+(8.0*scale)as i32};
            let x=if window.label()=="taskbar"{native::cursor().map(|c|c[0]-width/2).unwrap_or(bar[0]+bar[2]-width)}else{bar[0]+bar[2]-width};
            native::position_surface(handle(&center)?,x.min(p.x+size.width as i32-width).max(p.x),y.min(p.y+size.height as i32-height).max(p.y),width,height,true)?;
            native::surface_region(handle(&center)?,0,0,width,height,(26.0*scale)as i32,false)?;
            copy.emit_to("control-center","center-open",()).map_err(|e|e.to_string())?;center.show().map_err(|e|e.to_string())?;center.set_focus().map_err(|e|e.to_string())
        })();let _=tx.send(result);}).map_err(|e|e.to_string())?;rx.recv().map_err(|e|e.to_string())?
    }).await.map_err(|e|e.to_string())?
}
#[cfg(test)]mod tests{use super::*;#[test]#[ignore="Reads active audio endpoints and power without changing the system"]fn enumerates_real_outputs(){let _apartment=media::Apartment::new().unwrap();let(devices,interfaces)=audio_devices().unwrap();assert_eq!(devices.len(),interfaces.len());for d in devices{assert!(!d.id.is_empty());assert!(!d.name.is_empty())}let _=power();}}
