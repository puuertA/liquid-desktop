//! Borderless DXGI Desktop Duplication; application regions are masked.
//! One lazy broker shares DXGI frames across all WebViews; nothing is saved to disk.
use std::{collections::HashMap,ffi::c_void,sync::{mpsc,OnceLock},time::{Duration,Instant}};
use tauri::Manager;

use windows::{core::Interface,Win32::{Foundation::HMODULE,Graphics::{Direct3D::D3D_DRIVER_TYPE_UNKNOWN,Direct3D11::*,Dxgi::*}}};

#[link(name="user32")]
unsafe extern "system" {
 fn EnumWindows(callback:unsafe extern "system" fn(isize,isize)->i32,data:isize)->i32;
 fn EnumChildWindows(hwnd:isize,callback:unsafe extern "system" fn(isize,isize)->i32,data:isize)->i32;
 fn GetClassNameW(hwnd:*mut c_void,text:*mut u16,len:i32)->i32;
 fn GetWindowThreadProcessId(hwnd:*mut c_void,pid:*mut u32)->u32;
 fn GetAncestor(hwnd:isize,flags:u32)->isize;
 fn IsWindow(hwnd:*mut c_void)->i32;
 fn IsWindowVisible(hwnd:*mut c_void)->i32;
 fn IsIconic(hwnd:*mut c_void)->i32;
 fn GetWindowLongPtrW(hwnd:*mut c_void,index:i32)->isize;
 fn GetWindowDisplayAffinity(hwnd:*mut c_void,affinity:*mut u32)->i32;
 fn SetWindowDisplayAffinity(hwnd:*mut c_void,affinity:u32)->i32;
 fn MonitorFromRect(rect:*const NativeRect,flags:u32)->isize;
}
#[repr(C)]#[derive(Default,Clone,Copy)]struct NativeRect{left:i32,top:i32,right:i32,bottom:i32}
#[link(name="dwmapi")]
unsafe extern "system" {fn DwmGetWindowAttribute(hwnd:*mut c_void,attribute:u32,value:*mut c_void,size:u32)->i32;}
#[link(name="kernel32")]
unsafe extern "system" {
 fn OpenProcess(access:u32,inherit:i32,pid:u32)->isize;
 fn QueryFullProcessImageNameW(process:isize,flags:u32,name:*mut u16,len:*mut u32)->i32;
 fn CloseHandle(handle:isize)->i32;
}
#[derive(Clone,Debug)]
struct Target {hwnd:isize,leaf:isize,rect:[i32;4],monitor:isize}
fn class(hwnd:isize)->String {unsafe{let mut name=[0u16;256];let len=GetClassNameW(hwnd as *mut c_void,name.as_mut_ptr(),256);String::from_utf16_lossy(&name[..len.max(0)as usize])}}
fn wallpaper_process(hwnd:isize)->bool {unsafe{
 let mut pid=0;GetWindowThreadProcessId(hwnd as *mut c_void,&mut pid);let process=OpenProcess(0x1000,0,pid);if process==0{return false}
 let mut path=[0u16;32768];let mut len=path.len()as u32;let ok=QueryFullProcessImageNameW(process,0,path.as_mut_ptr(),&mut len);CloseHandle(process);
 if ok==0{return false}let name=String::from_utf16_lossy(&path[..len as usize]).to_lowercase();name.ends_with("\\wallpaper64.exe")||name.ends_with("\\wallpaper32.exe")
}}
unsafe extern "system" fn child(hwnd:isize,data:isize)->i32 {
 if class(hwnd).starts_with("WPE")&&wallpaper_process(hwnd){
  let root=GetAncestor(hwnd,2);if ["WorkerW","Progman"].contains(&class(root).as_str()){
   if let Ok(rect)=crate::native::rect(hwnd){if rect[2]>100&&rect[3]>100{let list=&mut *(data as *mut Vec<Target>);if !list.iter().any(|x|x.leaf==hwnd){list.push(Target{hwnd:root,leaf:hwnd,rect,monitor:0})}}}
  }
 }1
}
unsafe extern "system" fn top(hwnd:isize,data:isize)->i32 {child(hwnd,data);EnumChildWindows(hwnd,child,data);1}
fn targets()->Vec<Target>{let mut out=Vec::<Target>::new();unsafe{EnumWindows(top,&mut out as *mut _ as isize)};out}
fn monitor(rect:[i32;4])->Result<(isize,[i32;4]),String>{unsafe{
 let r=NativeRect{left:rect[0],top:rect[1],right:rect[0]+rect[2],bottom:rect[1]+rect[3]};let handle=MonitorFromRect(&r,2);
 let mut info=windows::Win32::Graphics::Gdi::MONITORINFO{cbSize:std::mem::size_of::<windows::Win32::Graphics::Gdi::MONITORINFO>()as u32,..Default::default()};
 if !windows::Win32::Graphics::Gdi::GetMonitorInfoW(windows::Win32::Graphics::Gdi::HMONITOR(handle as *mut c_void),&mut info).as_bool(){return Err("Monitor indisponível".into())}let m=info.rcMonitor;Ok((handle,[m.left,m.top,m.right-m.left,m.bottom-m.top]))
}}
unsafe extern "system" fn occluder(hwnd:isize,data:isize)->i32{
 #[cfg(test)]if TEST_FIXTURE.load(std::sync::atomic::Ordering::Relaxed)==hwnd{return 1}
 let h=hwnd as *mut c_void;if IsWindowVisible(h)==0||IsIconic(h)!=0{return 1}
 let mut cloaked=0u32;if DwmGetWindowAttribute(h,14,&mut cloaked as *mut _ as *mut c_void,4)==0&&cloaked!=0{return 1}
 let name=class(hwnd);if ["WorkerW","Progman"].contains(&name.as_str()){return 1}
 let mut affinity=0;if GetWindowDisplayAffinity(h,&mut affinity)!=0&&affinity==0x11{return 1}
 // Ignore invisible zero-area/message windows, but mask tool windows too.
 if GetWindowLongPtrW(h,-16)&0x40000000!=0{return 1}
 if let Ok(r)=crate::native::rect(hwnd){if r[2]>0&&r[3]>0{(&mut *(data as *mut Vec<[i32;4]>)).push(r)}}1
}
fn occlusions()->Vec<[i32;4]>{let mut out=Vec::new();unsafe{EnumWindows(occluder,&mut out as *mut _ as isize)};out}
fn mask_applications(data:&mut[u8],rect:[i32;4],covered:&[[i32;4]]){
 for bounds in covered{if let Some(r)=intersection(rect,*bounds){for y in r[1]-rect[1]..r[1]-rect[1]+r[3]{let offset=20+(y as usize*rect[2]as usize+(r[0]-rect[0])as usize)*4;data[offset..offset+r[2]as usize*4].fill(0);}}}
}
// Some Windows 10 drivers expose exclusion placeholders as hard white rectangles
// inside transparent WebView windows. Only remove large, nearly solid white
// components; small highlights and bright wallpaper details stay untouched.
fn mask_capture_placeholders(data:&mut[u8],width:usize,height:usize,over_apps:bool){
 let count=width.saturating_mul(height);if count==0||count>16_000_000{return}
 let mut seen=vec![false;count];
 let white=|i:usize,bytes:&[u8]|{let p=20+i*4;p+3<bytes.len()&&bytes[p+3]==255&&bytes[p]>=248&&bytes[p+1]>=248&&bytes[p+2]>=248};
 for start in 0..count{
  if seen[start]||!white(start,data){continue}
  seen[start]=true;let mut stack=vec![start];let mut pixels=Vec::new();let(mut left,mut right,mut top,mut bottom)=(start%width,start%width,start/width,start/width);
  while let Some(i)=stack.pop(){pixels.push(i);let x=i%width;let y=i/width;left=left.min(x);right=right.max(x);top=top.min(y);bottom=bottom.max(y);
   for next in [x.checked_sub(1).map(|_|i-1), (x+1<width).then_some(i+1),y.checked_sub(1).map(|_|i-width),(y+1<height).then_some(i+width)].into_iter().flatten(){if !seen[next]&&white(next,data){seen[next]=true;stack.push(next)}}
  }
  let area=(right-left+1)*(bottom-top+1);
  let fill=if over_apps{75}else{94};
  if right-left>=23&&bottom-top>=23&&pixels.len()*100>=area*fill&&(over_apps||area<count*4/5){for i in pixels{let p=20+i*4;data[p..p+4].fill(0)}}
 }
}
fn apply_capture_policy(data:&mut[u8],rect:[i32;4],covered:&[[i32;4]],over_apps:bool){
 if !over_apps{mask_applications(data,rect,covered)}
 mask_capture_placeholders(data,rect[2]as usize,rect[3]as usize,over_apps);
}
fn intersection(a:[i32;4],b:[i32;4])->Option<[i32;4]>{
 let x=a[0].max(b[0]);let y=a[1].max(b[1]);let r=(a[0]+a[2]).min(b[0]+b[2]);let d=(a[1]+a[3]).min(b[1]+b[3]);
 (r>x&&d>y).then_some([x,y,r-x,d-y])
}
fn texture_point(x:u32,y:u32,size:(u32,u32),rotation:i32)->(u32,u32){match rotation{
 2=>(y,size.1.saturating_sub(1+x)),3=>(size.0.saturating_sub(1+x),size.1.saturating_sub(1+y)),4=>(size.0.saturating_sub(1+y),x),_=>(x,y)
}}
struct Capture {
 target:Target,device:ID3D11Device,context:ID3D11DeviceContext,duplication:IDXGIOutputDuplication,
 staging:Option<ID3D11Texture2D>,size:(u32,u32),rotation:i32,pixels:Vec<u8>,sequence:u32,
 last_sample:Instant,last_request:Instant,monitor_capture:bool,covered:Vec<[i32;4]>,previous_covered:Vec<[i32;4]>,
}
impl Capture {
 fn new(mut target:Target)->Result<Self,String>{(move||->windows::core::Result<Self>{unsafe{
  let factory:IDXGIFactory1=CreateDXGIFactory1()?;
  for a in 0..32{let Ok(adapter)=factory.EnumAdapters1(a)else{break};
   for o in 0..32{let Ok(output)=adapter.EnumOutputs(o)else{break};let desc=output.GetDesc()?;
    if desc.Monitor.0 as isize!=target.monitor{continue}
    let r=desc.DesktopCoordinates;target.rect=[r.left,r.top,r.right-r.left,r.bottom-r.top];
    let mut device=None;let mut context=None;
    D3D11CreateDevice(&adapter,D3D_DRIVER_TYPE_UNKNOWN,HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,None,D3D11_SDK_VERSION,Some(&mut device),None,Some(&mut context))?;
    let device=device.unwrap();let context=context.unwrap();let output:IDXGIOutput1=output.cast()?;
    let duplication=output.DuplicateOutput(&device)?;
    return Ok(Self{target,device,context,duplication,staging:None,size:(0,0),rotation:desc.Rotation.0,pixels:Vec::new(),sequence:0,last_sample:Instant::now()-Duration::from_secs(1),last_request:Instant::now(),monitor_capture:true,covered:Vec::new(),previous_covered:Vec::new()})
   }
  }
  Err(windows::core::Error::new(windows::core::HRESULT(0x80004005u32 as i32),"Monitor não encontrado no adaptador DXGI"))
 }})().map_err(|e|format!("Captura DXGI indisponível: {e}"))}
 fn sample(&mut self)->Result<(),String>{
  self.last_request=Instant::now();if self.last_sample.elapsed()<Duration::from_millis(32){return Ok(())}self.last_sample=Instant::now();
  if unsafe{IsWindow(self.target.hwnd as *mut c_void)==0||IsWindow(self.target.leaf as *mut c_void)==0}{return Err("Wallpaper Engine reiniciou".into())}
  self.previous_covered=std::mem::replace(&mut self.covered,occlusions());
  let mut info=DXGI_OUTDUPL_FRAME_INFO::default();let mut resource=None;
  match unsafe{self.duplication.AcquireNextFrame(0,&mut info,&mut resource)}{
   Ok(())=>{},Err(e)if e.code()==DXGI_ERROR_WAIT_TIMEOUT=>return Ok(()),Err(e)=>return Err(e.to_string())
  }
  // ReleaseFrame runs on every path after AcquireNextFrame, including copy/map errors.
  let result=(||->windows::core::Result<()>{unsafe{
   let texture:ID3D11Texture2D=resource.ok_or_else(||windows::core::Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32)))?.cast()?;
   let mut desc=D3D11_TEXTURE2D_DESC::default();texture.GetDesc(&mut desc);
   if desc.Width>16384||desc.Height>16384||u64::from(desc.Width)*u64::from(desc.Height)>40_000_000{return Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x80070057u32 as i32)))}
   if self.size!=(desc.Width,desc.Height){
    desc.Usage=D3D11_USAGE_STAGING;desc.BindFlags=0;desc.CPUAccessFlags=D3D11_CPU_ACCESS_READ.0 as u32;desc.MiscFlags=0;
    self.device.CreateTexture2D(&desc,None,Some(&mut self.staging))?;self.size=(desc.Width,desc.Height);self.pixels.resize(desc.Width as usize*desc.Height as usize*4,0);
   }
   let staging=self.staging.as_ref().unwrap();self.context.CopyResource(staging,&texture);
   let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();self.context.Map(staging,0,D3D11_MAP_READ,0,Some(&mut mapped))?;
   let stride=self.size.0 as usize*4;for y in 0..self.size.1 as usize{std::ptr::copy_nonoverlapping((mapped.pData as *const u8).add(y*mapped.RowPitch as usize),self.pixels.as_mut_ptr().add(y*stride),stride)}
   self.context.Unmap(staging,0);self.sequence=self.sequence.wrapping_add(1);Ok(())
  }})();let released=unsafe{self.duplication.ReleaseFrame()};result.and(released).map_err(|e|e.to_string())
 }
 fn crop(&self,request:[i32;4],over_apps:bool)->Result<Vec<u8>,String>{
  if self.pixels.is_empty(){return Err("Aguardando primeiro frame do Wallpaper Engine".into())}
  let bounds=self.target.rect;let r=intersection(request,bounds).ok_or("Wallpaper Engine não cobre esta superfície")?;
  let mut out=Vec::with_capacity(20+r[2]as usize*r[3]as usize*4);
  for value in [r[0],r[1],r[2],r[3],self.sequence as i32]{out.extend_from_slice(&value.to_le_bytes())}
  for y in r[1]-bounds[1]..r[1]-bounds[1]+r[3]{for x in r[0]-bounds[0]..r[0]-bounds[0]+r[2]{
   let (tx,ty)=texture_point(x as u32,y as u32,self.size,self.rotation);
   if tx>=self.size.0||ty>=self.size.1{return Err("Geometria da captura mudou; aguardando novo monitor".into())}
   let offset=(ty as usize*self.size.0 as usize+tx as usize)*4;let p=&self.pixels[offset..offset+4];out.extend_from_slice(&[p[2],p[1],p[0],255]);
  }}if self.monitor_capture&&!fixture_covers(r){
    if !over_apps{mask_applications(&mut out,r,&self.previous_covered);mask_applications(&mut out,r,&self.covered)}
    apply_capture_policy(&mut out,r,&occlusions(),over_apps);
  }Ok(out)
 }
}
struct Request {rect:[i32;4],over_apps:bool,reply:mpsc::SyncSender<Result<Vec<u8>,String>>}
fn broker()->&'static mpsc::SyncSender<Request>{static BROKER:OnceLock<mpsc::SyncSender<Request>>=OnceLock::new();BROKER.get_or_init(||{
 let(tx,rx)=mpsc::sync_channel::<Request>(16);std::thread::spawn(move||{
  let apartment=crate::media::Apartment::new();let mut captures=HashMap::<(isize,isize),Capture>::new();let mut found=Vec::new();let mut discovery=Instant::now()-Duration::from_secs(10);
  loop{
   captures.retain(|_,c|c.last_request.elapsed()<Duration::from_secs(2));
   let request=match rx.recv_timeout(Duration::from_millis(250)){Ok(r)=>r,Err(mpsc::RecvTimeoutError::Timeout)=>continue,Err(_)=>break};
   let answer=(||{
    if let Err(e)=&apartment{return Err(e.clone())}
    if discovery.elapsed()>Duration::from_secs(2){found=targets();discovery=Instant::now();captures.retain(|id,_|found.iter().any(|t|t.leaf==id.0));}
    let target=found.iter().filter_map(|t|intersection(t.rect,request.rect).map(|r|(t,i64::from(r[2])*i64::from(r[3])))).max_by_key(|(_,area)|*area).map(|(t,_)|t.clone()).ok_or("Wallpaper Engine não encontrado neste monitor")?;
    let(m,r)=monitor(request.rect)?;let key=(target.leaf,m);
    if captures.get(&key).is_some_and(|c|c.monitor_capture&&c.target.rect!=r){captures.remove(&key);}
    if !captures.contains_key(&key){let mut source=target.clone();source.monitor=m;source.rect=r;captures.insert(key,Capture::new(source)?);}
    let capture=captures.get_mut(&key).unwrap();
    if let Err(e)=capture.sample(){captures.remove(&key);return Err(e)}capture.crop(request.rect,request.over_apps)
   })();let _=request.reply.send(answer);
  }
 });tx
})}
fn read(rect:[i32;4],over_apps:bool)->Result<Vec<u8>,String>{let(tx,rx)=mpsc::sync_channel(1);broker().try_send(Request{rect,over_apps,reply:tx}).map_err(|_|"Captura ocupada; tentando novamente".to_string())?;rx.recv_timeout(Duration::from_secs(3)).map_err(|_|"Tempo de captura excedido".to_string())?}
#[cfg(test)]pub static TEST_FIXTURE:std::sync::atomic::AtomicIsize=std::sync::atomic::AtomicIsize::new(0);
#[cfg(test)]fn fixture_covers(rect:[i32;4])->bool{let h=TEST_FIXTURE.load(std::sync::atomic::Ordering::Relaxed);h!=0&&crate::native::rect(h).ok().is_some_and(|r|intersection(r,rect)==Some(rect))}
#[cfg(not(test))]fn fixture_covers(_rect:[i32;4])->bool{false}
#[cfg(test)]pub fn test_frame(rect:[i32;4])->Result<Vec<u8>,String>{read(rect,false)}
pub fn set_excluded(hwnd:isize,excluded:bool)->Result<(),String>{unsafe{
 let h=hwnd as *mut c_void;let target=if excluded{0x11}else{0};let mut value=0;
 if GetWindowDisplayAffinity(h,&mut value)==0||value!=target{if SetWindowDisplayAffinity(h,target)==0{return Err("Windows não permitiu configurar a exclusão do vidro da captura".into())}}
 if GetWindowDisplayAffinity(h,&mut value)==0||value!=target{return Err("Exclusão da captura não confirmada pelo Windows".into())}Ok(())
}}
#[tauri::command]
pub async fn live_wallpaper_frame(app:tauri::AppHandle,window:tauri::WebviewWindow,bounds:[f64;4])->Result<tauri::ipc::Response,String>{
 if !["clock","analog","calendar","notes","system","media","weather","gallery","storage","fusion","topbar","island","taskbar"].contains(&window.label()){return Err("Superfície não autorizada".into())}
 if bounds.iter().any(|v|!v.is_finite()||v.abs()>32768.0)||bounds[2]<=0.0||bounds[3]<=0.0{return Err("Recorte inválido".into())}
 for(label,w)in app.webview_windows(){if ["clock","analog","calendar","notes","system","media","weather","gallery","storage","fusion","topbar","island","taskbar","control-center"].contains(&label.as_str()){
  set_excluded(crate::handle(&w)?,true)?;
 }}
 let screen=crate::native::rect(crate::handle(&window)?)?;let scale=window.scale_factor().map_err(|e|e.to_string())?;
 let r=[screen[0]+(bounds[0]*scale).floor()as i32,screen[1]+(bounds[1]*scale).floor()as i32,(bounds[2]*scale).ceil()as i32,(bounds[3]*scale).ceil()as i32];
 let rect=intersection(r,screen).ok_or("Recorte fora da janela")?;
 if i64::from(rect[2])*i64::from(rect[3])>16_000_000{return Err("Superfície grande demais para captura".into())}
 let over_apps=matches!(window.label(),"island"|"topbar"|"taskbar");
 tauri::async_runtime::spawn_blocking(move||read(rect,over_apps).map(tauri::ipc::Response::new)).await.map_err(|e|e.to_string())?
}
pub fn maintain_affinity(app:&tauri::AppHandle){
 let state=app.state::<crate::State>();let layout=state.layout.lock().unwrap();
 let active=layout.widgets.values().any(|s|s.enabled&&!s.static_wallpaper)
  ||((layout.topbar.enabled||layout.island.enabled||layout.taskbar.enabled)&&!layout.widgets[&layout.glass_source].static_wallpaper)
  ||(layout.fusion.enabled&&!layout.widgets[&layout.fusion.source].static_wallpaper);
 drop(layout);if active{return}
 for(_,w)in app.webview_windows(){if let Ok(h)=crate::handle(&w){unsafe{let mut value=0;if GetWindowDisplayAffinity(h as *mut c_void,&mut value)!=0&&value==0x11{SetWindowDisplayAffinity(h as *mut c_void,0);}}}}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn rotated_display_maps_to_texture(){assert_eq!(texture_point(0,0,(1920,1080),2),(0,1079));assert_eq!(texture_point(1079,1919,(1920,1080),2),(1919,0));assert_eq!(texture_point(1919,1079,(1920,1080),3),(0,0));assert_eq!(texture_point(1079,1919,(1920,1080),4),(0,1079));}
 #[test]fn clips_negative_monitor_coordinates(){assert_eq!(intersection([-1920,0,1920,1080],[-2000,30,400,300]),Some([-1920,30,320,300]));assert_eq!(intersection([0,0,10,10],[20,0,10,10]),None);}
 #[test]fn masks_application_pixels_without_touching_header_or_visible_wallpaper(){
  let mut data=vec![255;20+4*3*4];let header=data[..20].to_vec();mask_applications(&mut data,[-4,2,4,3],&[[-3,2,2,2]]);
  assert_eq!(&data[..20],header.as_slice());assert_eq!(&data[20..24],&[255;4]);assert_eq!(&data[24..32],&[0;8]);assert_eq!(&data[20+2*4*4..],&[255;16]);
 }
 #[test]fn masks_large_white_capture_placeholders_but_preserves_small_glints(){
  let(w,h)=(96usize,72usize);let mut frame=vec![0u8;20+w*h*4];
  for p in frame[20..].chunks_exact_mut(4){p.copy_from_slice(&[110,14,22,255])}
  for y in 13..51{for x in 18..61{let p=20+(y*w+x)*4;frame[p..p+4].copy_from_slice(&[255,255,255,255])}}
  for y in 3..8{for x in 3..8{let p=20+(y*w+x)*4;frame[p..p+4].copy_from_slice(&[255,255,255,255])}}
  mask_capture_placeholders(&mut frame,w,h,false);
  assert_eq!(&frame[20+(20*w+30)*4..20+(20*w+30)*4+4],&[0,0,0,0]);
  assert_eq!(&frame[20+(4*w+4)*4..20+(4*w+4)*4+4],&[255,255,255,255]);
  assert_eq!(&frame[20+(20*w+70)*4..20+(20*w+70)*4+4],&[110,14,22,255]);
 }
 #[test]fn island_preserves_dark_application_pixels_but_discards_white_exclusion_block(){
  let rect=[0,0,96,72];let mut frame=vec![0u8;20+96*72*4];
  for p in frame[20..].chunks_exact_mut(4){p.copy_from_slice(&[38,39,45,255])}
  for y in 10..60{for x in 10..80{let p=20+(y*96+x)*4;frame[p..p+4].copy_from_slice(&[255,255,255,255])}}
  apply_capture_policy(&mut frame,rect,&[rect],true);
  assert_eq!(&frame[20+(4*96+4)*4..20+(4*96+4)*4+4],&[38,39,45,255]);
  assert_eq!(&frame[20+(20*96+20)*4..20+(20*96+20)*4+4],&[0,0,0,0]);
  let mut widget=vec![0u8;20+96*72*4];for p in widget[20..].chunks_exact_mut(4){p.copy_from_slice(&[38,39,45,255])}
  apply_capture_policy(&mut widget,rect,&[rect],false);
  assert_eq!(&widget[20+(4*96+4)*4..20+(4*96+4)*4+4],&[0,0,0,0]);
 }
 #[test]#[ignore="Captures live wallpaper with application masking; keeps pixels in memory"]fn reads_real_wallpaper_frames(){
  let found=targets();assert!(!found.is_empty(),"No Wallpaper Engine desktop host found");let r=found[0].rect;println!("Wallpaper host {:?}",found[0]);
  let mut previous:Option<Vec<u8>>=None;let mut count=0;let mut changed=false;let mut visible=0;
  for _ in 0..40{match read(r,false){Ok(data)=>{count+=1;visible=data[20..].chunks_exact(4).filter(|p|p[3]>0).count();if let Some(old)=&previous{changed|=old[20..].chunks_exact(4).zip(data[20..].chunks_exact(4)).any(|(a,b)|a[3]==255&&b[3]==255&&a[..3]!=b[..3])}previous=Some(data)},Err(e)=>println!("Capture pending/error: {e}")}if count>=3&&changed{break}std::thread::sleep(Duration::from_millis(100));}
  assert!(count>=3,"No frames delivered");println!("{count} frames; visible wallpaper pixels={visible}; pixel changes={changed}");assert!(visible>0,"Desktop is fully occluded; show wallpaper to validate movement");assert!(changed,"Wallpaper frame pixels did not change; check Wallpaper Engine playback/pause rules");
 }
}
