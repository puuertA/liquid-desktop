//! One invisible AppBar per occupied monitor. Never writes SPI_SETWORKAREA.
//! All functions and owned HWNDs are confined to the application's UI thread.
use std::{cell::RefCell, ffi::c_void, sync::atomic::{AtomicBool, Ordering}};
use tauri::{Emitter, Manager};
type Hwnd=*mut c_void;
#[repr(C)]
#[derive(Clone,Copy,Default,PartialEq,Debug)]
struct Rect {left:i32,top:i32,right:i32,bottom:i32}
#[repr(C)]
struct AppbarData {size:u32,hwnd:Hwnd,callback:u32,edge:u32,rect:Rect,param:isize}
type Subclass=unsafe extern "system" fn(Hwnd,u32,usize,isize,usize,usize)->isize;
#[link(name="shell32")]
unsafe extern "system" {fn SHAppBarMessage(message:u32,data:*mut AppbarData)->usize;}
#[link(name="user32")]
unsafe extern "system" {
    fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:Hwnd,menu:Hwnd,instance:Hwnd,param:Hwnd)->Hwnd;
    fn DestroyWindow(hwnd:Hwnd)->i32;
    fn SetWindowPos(hwnd:Hwnd,after:Hwnd,x:i32,y:i32,w:i32,h:i32,flags:u32)->i32;
    fn RegisterWindowMessageW(name:*const u16)->u32;
}
#[link(name="comctl32")]
unsafe extern "system" {
    fn SetWindowSubclass(hwnd:Hwnd,proc:Subclass,id:usize,data:usize)->i32;
    fn RemoveWindowSubclass(hwnd:Hwnd,proc:Subclass,id:usize)->i32;
    fn DefSubclassProc(hwnd:Hwnd,message:u32,w:usize,l:isize)->isize;
}
const CALLBACK:u32=0x805b;
const HOOK:usize=0x4c475741;
static DIRTY:AtomicBool=AtomicBool::new(false);
static RESTARTED:AtomicBool=AtomicBool::new(false);
fn wide(value:&str)->Vec<u16>{value.encode_utf16().chain(Some(0)).collect()}
fn data(hwnd:Hwnd)->AppbarData {AppbarData{size:std::mem::size_of::<AppbarData>() as u32,hwnd,callback:CALLBACK,edge:1,rect:Rect::default(),param:0}}
unsafe extern "system" fn callback(hwnd:Hwnd,msg:u32,w:usize,l:isize,_id:usize,restart:usize)->isize {
    // Do not negotiate here: Explorer can call synchronously during ABM_SETPOS.
    if msg==CALLBACK&&w==1 {DIRTY.store(true,Ordering::Relaxed);}
    if msg==restart as u32 {RESTARTED.store(true,Ordering::Relaxed);}
    if msg==0x0082 {SHAppBarMessage(1,&mut data(hwnd));RemoveWindowSubclass(hwnd,callback,HOOK);}
    DefSubclassProc(hwnd,msg,w,l)
}
#[derive(Clone,Copy,PartialEq,Debug)]
struct Request {monitor:Rect,height:i32}
struct Bar {hwnd:Hwnd,request:Request,allocated:Rect,registered:bool}
impl Bar {
    fn new(request:Request)->Result<Self,String>{
        unsafe {
            let null=std::ptr::null_mut();
            let hwnd=CreateWindowExW(0x80|0x08000000,wide("STATIC").as_ptr(),wide("Liquid Desktop reserved space").as_ptr(),0x80000000,request.monitor.left,request.monitor.top,1,1,null,null,null,null);
            if hwnd.is_null(){return Err("Não foi possível criar a reserva de espaço".into())}
            let mut bar=Self{hwnd,request,allocated:Rect::default(),registered:false};
            let restart=RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
            if restart==0||SetWindowSubclass(hwnd,callback,HOOK,restart as usize)==0{return Err("Não foi possível acompanhar a barra do Windows".into())}
            bar.register()?;bar.negotiate(request,true)?;Ok(bar)
        }
    }
    fn register(&mut self)->Result<(),String>{
        if unsafe{SHAppBarMessage(0,&mut data(self.hwnd))}==0{return Err("O Windows recusou a reserva de espaço AppBar".into())}
        self.registered=true;Ok(())
    }
    fn negotiate(&mut self,request:Request,force:bool)->Result<(),String>{
        let mut info=data(self.hwnd);
        info.rect=request.monitor;info.rect.bottom=info.rect.top+request.height;
        unsafe {SHAppBarMessage(2,&mut info);}
        info.rect.bottom=info.rect.top+request.height;
        if force||info.rect!=self.allocated||self.request!=request {
            unsafe {
                SHAppBarMessage(3,&mut info);
                if SetWindowPos(self.hwnd,std::ptr::null_mut(),info.rect.left,info.rect.top,info.rect.right-info.rect.left,info.rect.bottom-info.rect.top,0x4|0x10|0x200)==0 {
                    return Err("Não foi possível posicionar a reserva AppBar".into())
                }
                SHAppBarMessage(9,&mut info);
            }
            self.allocated=info.rect;
        }
        self.request=request;Ok(())
    }
}
impl Drop for Bar {
    fn drop(&mut self){unsafe {
        if self.registered{SHAppBarMessage(1,&mut data(self.hwnd));}
        RemoveWindowSubclass(self.hwnd,callback,HOOK);
        DestroyWindow(self.hwnd);
    }}
}
thread_local! {
    static BARS:RefCell<Vec<Bar>>=const{RefCell::new(Vec::new())};
    static ERROR:RefCell<Option<String>>=const{RefCell::new(None)};
}
fn effective(index:usize,count:usize)->usize{if index<count{index}else{0}}
fn height(layout:&crate::Layout,index:usize,count:usize,bar_exists:bool,island_exists:bool)->f64 {
    // Two logical pixels below the persistent surface (bar 36+8, island up to 56).
    // Transient notices and expansion overlay apps rather than moving the work area.
    let bar=if layout.topbar.enabled&&bar_exists&&effective(layout.topbar.monitor,count)==index {46.0_f64}else{0.0};
    let island=if layout.island.enabled&&island_exists&&effective(layout.island.monitor,count)==index {
        if layout.island.form=="notch"{58.0}else{70.0}
    }else{0.0};
    bar.max(island)
}
fn sync(app:&tauri::AppHandle)->Result<(),String>{
    let window=app.get_webview_window("main").ok_or("Painel indisponível para ler monitores")?;
    let monitors=window.available_monitors().map_err(|e|e.to_string())?;
    let layout=app.state::<crate::State>().layout.lock().unwrap().clone();
    let bar_exists=app.get_webview_window("topbar").is_some();let island_exists=app.get_webview_window("island").is_some();
    let requests:Vec<Request>=monitors.iter().enumerate().filter_map(|(i,m)|{
        let h=height(&layout,i,monitors.len(),bar_exists,island_exists);
        let p=m.position();let s=m.size();
        (h>0.0).then_some(Request{monitor:Rect{left:p.x,top:p.y,right:p.x+s.width as i32,bottom:p.y+s.height as i32},height:(h*m.scale_factor()).round() as i32})
    }).collect();
    let dirty=DIRTY.swap(false,Ordering::Relaxed);let restarted=RESTARTED.swap(false,Ordering::Relaxed);
    BARS.with(|bars|{
        let mut bars=bars.borrow_mut();
        bars.retain(|bar|requests.iter().any(|r|r.monitor==bar.request.monitor));
        for request in requests {
            if let Some(bar)=bars.iter_mut().find(|b|b.request.monitor==request.monitor) {
                if restarted {bar.registered=false;}
                let register=!bar.registered;
                if register {bar.register()?;}
                if register||dirty||request!=bar.request {bar.negotiate(request,register)?;}
            }else{bars.push(Bar::new(request)?);}
        }
        Ok(())
    })
}
pub fn refresh(app:&tauri::AppHandle){
    if let Err(error)=sync(app){
        DIRTY.store(true,Ordering::Relaxed);
        ERROR.with(|previous|{if previous.borrow().as_ref()!=Some(&error){let _=app.emit_to("main","widget-error",&error);*previous.borrow_mut()=Some(error);}});
    }else{ERROR.with(|previous|*previous.borrow_mut()=None);}
}
/// Offset negotiated with other AppBars/taskbars occupying the same edge.
pub fn offset(x:i32,y:i32)->i32 {
    BARS.with(|bars|bars.borrow().iter().find(|b|b.request.monitor.left==x&&b.request.monitor.top==y).map_or(0,|b|b.allocated.top-y))
}
pub fn clear(){BARS.with(|bars|bars.borrow_mut().clear());}

#[cfg(test)]
mod tests {
    use super::*;
    #[repr(C)]
    struct Point{x:i32,y:i32}
    #[repr(C)]
    struct MonitorInfo{size:u32,monitor:Rect,work:Rect,flags:u32}
    #[link(name="user32")]
    unsafe extern "system" {
        fn MonitorFromPoint(point:Point,flags:u32)->Hwnd;
        fn GetMonitorInfoW(monitor:Hwnd,info:*mut MonitorInfo)->i32;
    }
    fn monitor_info(monitor:Hwnd)->MonitorInfo {
        let mut info=MonitorInfo{size:std::mem::size_of::<MonitorInfo>() as u32,monitor:Rect::default(),work:Rect::default(),flags:0};
        assert_ne!(unsafe{GetMonitorInfoW(monitor,&mut info)},0);info
    }
    #[test]
    #[ignore="Temporarily reserves 8-16 physical pixels on the primary monitor and restores them with RAII"]
    fn shell_reserves_space_and_restores_it_after_remove(){
        let monitor=unsafe{MonitorFromPoint(Point{x:0,y:0},1)};
        assert!(!monitor.is_null());
        let before=monitor_info(monitor);
        {
            let mut bar=Bar::new(Request{monitor:before.monitor,height:8}).expect("Register temporary AppBar");
            assert!(bar.allocated.top>=before.work.top,"Respect existing top AppBars");
            let after=monitor_info(monitor);
            assert!(after.work.top>=before.work.top+8,"Windows must actually reduce the work area");
            assert_eq!(after.work.bottom,before.work.bottom,"Preserve bottom taskbar");
            bar.negotiate(Request{monitor:before.monitor,height:16},false).unwrap();
            assert_eq!(monitor_info(monitor).work.top,before.work.top+16,"Resize, never stack a second reservation");
            bar.negotiate(Request{monitor:before.monitor,height:16},false).unwrap();
            assert_eq!(monitor_info(monitor).work.top,before.work.top+16,"Repeated maintenance must not drift");
            bar.negotiate(Request{monitor:before.monitor,height:8},false).unwrap();
            assert_eq!(monitor_info(monitor).work.top,before.work.top+8,"Shrinking a reservation returns the difference");
        }
        for _ in 0..20 {
            if monitor_info(monitor).work==before.work{return}
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(monitor_info(monitor).work,before.work,"ABM_REMOVE must release the reserved strip");
    }
    #[test]
    fn one_reservation_per_monitor_never_adds_bar_and_island_heights(){
        let mut layout=crate::Layout::default();
        assert_eq!(height(&layout,0,2,true,true),0.0);
        layout.topbar.enabled=true;assert_eq!(height(&layout,0,2,true,false),46.0);
        layout.island.enabled=true;assert_eq!(height(&layout,0,2,true,true),70.0);
        layout.island.form="notch".into();assert_eq!(height(&layout,0,2,true,true),58.0);
        layout.island.monitor=1;assert_eq!(height(&layout,0,2,true,true),46.0);assert_eq!(height(&layout,1,2,true,true),58.0);
        assert_eq!(height(&layout,0,1,true,true),58.0);
        layout.topbar.enabled=false;layout.island.enabled=false;assert_eq!(height(&layout,0,1,true,true),0.0);
    }
    #[test]
    fn a_failed_or_closed_surface_does_not_reserve_space(){
        let mut layout=crate::Layout::default();layout.topbar.enabled=true;layout.island.enabled=true;
        assert_eq!(height(&layout,0,1,false,false),0.0);
        assert_eq!(height(&layout,0,1,true,false),46.0);
        assert_eq!((height(&layout,0,1,false,true)*1.5).round() as i32,105);
    }
}
