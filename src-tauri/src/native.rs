//! Shell attachment is experimental. Never changes Explorer styles or files.
use std::ffi::c_void;
type Hwnd = *mut c_void;
type SubclassProc = unsafe extern "system" fn(Hwnd, u32, usize, isize, usize, usize) -> isize;
const BORDERLESS_SUBCLASS: usize = 0x4c474c53;
#[link(name = "comctl32")]
unsafe extern "system" {
    fn SetWindowSubclass(hwnd: Hwnd, callback: SubclassProc, id: usize, data: usize) -> i32;
    fn RemoveWindowSubclass(hwnd: Hwnd, callback: SubclassProc, id: usize) -> i32;
    fn DefSubclassProc(hwnd: Hwnd, message: u32, wparam: usize, lparam: isize) -> isize;
}

unsafe extern "system" fn borderless_proc(
    hwnd: Hwnd,
    message: u32,
    wparam: usize,
    lparam: isize,
    id: usize,
    _: usize,
) -> isize {
    match message {
        0x0083 => 0, // WM_NCCALCSIZE: the entire window is client area, both forms.
        0x0085 => 0, // WM_NCPAINT: no OS caption to draw.
        // Let Tao update focus, but tell DefWindowProc not to repaint the caption.
        0x0086 => DefSubclassProc(hwnd, message, wparam, -1),
        0x0082 => {
            // WM_NCDESTROY: release hook, preserve the original teardown.
            RemoveWindowSubclass(hwnd, borderless_proc, id);
            DefSubclassProc(hwnd, message, wparam, lparam)
        }
        _ => DefSubclassProc(hwnd, message, wparam, lparam),
    }
}

/// Must run on the HWND's owning UI thread, never an async worker.
fn install_borderless_frame(hwnd: isize) -> Result<(), String> {
    unsafe {
        let h = hwnd as Hwnd;
        if SetWindowSubclass(h, borderless_proc, BORDERLESS_SUBCLASS, 0) == 0 {
            return Err(
                "Não foi possível instalar o tratamento da moldura na thread da janela".into(),
            );
        }
        if SetWindowPos(
            h,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            0x1 | 0x2 | 0x4 | 0x10 | 0x20,
        ) == 0
        {
            return Err("Não foi possível recalcular a área sem título".into());
        }
        Ok(())
    }
}
#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(class: *const u16, name: *const u16) -> Hwnd;
    fn FindWindowExW(parent: Hwnd, after: Hwnd, class: *const u16, name: *const u16) -> Hwnd;

    #[cfg(test)]
    fn GetParent(child: Hwnd) -> Hwnd;
    fn GetTopWindow(parent: Hwnd) -> Hwnd;
    fn GetWindow(hwnd: Hwnd, command: u32) -> Hwnd;
    fn GetClassNameW(hwnd: Hwnd, buffer: *mut u16, length: i32) -> i32;
    fn IsWindowVisible(hwnd: Hwnd) -> i32;
    fn IsWindow(hwnd: Hwnd) -> i32;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn SetWindowPos(hwnd: Hwnd, after: Hwnd, x: i32, y: i32, w: i32, h: i32, flags: u32) -> i32;
    fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> i32;

    fn SetWindowRgn(hwnd: Hwnd, region: Hwnd, redraw: i32) -> i32;
    fn GetPhysicalCursorPos(point: *mut Point) -> i32;
    fn GetForegroundWindow() -> Hwnd;
    fn GetWindowThreadProcessId(hwnd: Hwnd, process: *mut u32) -> u32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateRoundRectRgn(l: i32, t: i32, r: i32, b: i32, w: i32, h: i32) -> Hwnd;
    fn DeleteObject(object: Hwnd) -> i32;
    fn CreateRectRgn(l: i32, t: i32, r: i32, b: i32) -> Hwnd;
    fn CombineRgn(dest: Hwnd, a: Hwnd, b: Hwnd, mode: i32) -> i32;
}
/// Floating surfaces have their own layer; never feed them into widget ordering.
pub fn position_surface(
    hwnd: isize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    above: bool,
) -> Result<(), String> {
    unsafe {
        remove_frame(hwnd)?;
        if SetWindowPos(
            hwnd as Hwnd,
            if above {
                -1isize as Hwnd
            } else {
                -2isize as Hwnd
            },
            x,
            y,
            w,
            h,
            0x10 | 0x200,
        ) == 0
        {
            return Err("Falha ao posicionar superfície".into());
        }
        Ok(())
    }
}
/// The OS clips painting AND hit-testing to this region, including transparent margins.
pub fn surface_region(
    hwnd: isize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    radius: i32,
    notch: bool,
) -> Result<(), String> {
    unsafe {
        let region = CreateRoundRectRgn(x, y, x + w + 1, y + h + 1, radius * 2, radius * 2);
        if region.is_null() {
            return Err("Falha ao criar região da ilha".into());
        }
        if notch {
            let top = CreateRectRgn(x, y, x + w + 1, y + radius + 1);
            if top.is_null() {
                DeleteObject(region);
                return Err("Falha ao criar notch".into());
            }
            let result = CombineRgn(region, region, top, 2);
            DeleteObject(top);
            if result == 0 {
                DeleteObject(region);
                return Err("Falha ao compor notch".into());
            }
        }
        if SetWindowRgn(hwnd as Hwnd, region, 1) == 0 {
            DeleteObject(region);
            return Err("Falha ao atualizar região da ilha".into());
        }
        Ok(())
    }
}
/// The central gap is absent from the HWND region, not merely transparent CSS.
pub fn bar_region(hwnd:isize,width:i32,height:i32,gap:i32,radius:i32)->Result<(),String> {
    unsafe {
        let region=CreateRectRgn(0,0,0,0);
        if region.is_null(){return Err("Falha ao criar região da barra".into())}
        let spans=if gap>0 {vec![(0,(width-gap).max(0)/2),((width+gap).min(width*2)/2,width)]} else {vec![(0,width)]};
        for (left,right) in spans {
            if right<=left{continue}
            let part=CreateRoundRectRgn(left,0,right+1,height+1,2*radius,2*radius);
            if part.is_null(){DeleteObject(region);return Err("Falha ao criar segmento da barra".into())}
            let result=CombineRgn(region,region,part,2);DeleteObject(part);
            if result==0{DeleteObject(region);return Err("Falha ao unir segmentos da barra".into())}
        }
        if SetWindowRgn(hwnd as Hwnd,region,1)==0{DeleteObject(region);return Err("Falha ao aplicar região da barra".into())}
        Ok(())
    }
}
pub fn scanline_region(hwnd:isize,rows:&[[i32;4]])->Result<(),String>{
    #[link(name="gdi32")]
    unsafe extern "system"{fn ExtCreateRegion(transform:*const c_void,count:u32,data:*const c_void)->Hwnd;}
    // RGNDATAHEADER followed by ordered, non-overlapping RECT scanlines.
    let mut words=Vec::<i32>::with_capacity(8+rows.len()*4);
    let bounds=[rows.iter().map(|r|r[0]).min().unwrap_or(0),rows.iter().map(|r|r[1]).min().unwrap_or(0),rows.iter().map(|r|r[2]).max().unwrap_or(0),rows.iter().map(|r|r[3]).max().unwrap_or(0)];
    words.extend([32,1,rows.len() as i32,(rows.len()*16)as i32]);words.extend(bounds);
    for row in rows{words.extend(row)}
    unsafe{
        let region=if rows.is_empty(){CreateRectRgn(0,0,0,0)}else{ExtCreateRegion(std::ptr::null(),(words.len()*4)as u32,words.as_ptr().cast())};
        if region.is_null(){return Err("Falha ao criar contorno líquido".into())}
        if SetWindowRgn(hwnd as Hwnd,region,1)==0{DeleteObject(region);return Err("Falha ao aplicar contorno líquido".into())}
    }Ok(())
}
pub fn foreground_fullscreen(monitor: [i32; 4]) -> bool {
    unsafe {
        let h = GetForegroundWindow();
        if h.is_null() {
            return false;
        }
        let mut process = 0;
        GetWindowThreadProcessId(h, &mut process);
        if process == std::process::id() {
            return false;
        }
        let mut name = [0u16; 256];
        let len = GetClassNameW(h, name.as_mut_ptr(), 256);
        let class = String::from_utf16_lossy(&name[..len.max(0) as usize]);
        if ["Progman", "WorkerW", "Shell_TrayWnd"].contains(&class.as_str()) {
            return false;
        }
        let mut r = Rect::default();
        if GetWindowRect(h, &mut r) == 0 {
            return false;
        }
        r.left <= monitor[0]
            && r.top <= monitor[1]
            && r.right >= monitor[0] + monitor[2]
            && r.bottom >= monitor[1] + monitor[3]
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn probe() -> bool {
    unsafe { !desktop_host().is_null() }
}
unsafe fn desktop_host() -> Hwnd {
    let null = std::ptr::null_mut();
    let has_icons = |p: Hwnd| {
        !p.is_null()
            && !FindWindowExW(p, null, wide("SHELLDLL_DefView").as_ptr(), std::ptr::null())
                .is_null()
    };
    let progman = FindWindowW(wide("Progman").as_ptr(), std::ptr::null());
    if has_icons(progman) {
        return progman;
    }
    // Only the existing icon-bearing WorkerW is eligible, never the wallpaper layer.
    let mut worker = null;
    loop {
        worker = FindWindowExW(null, worker, wide("WorkerW").as_ptr(), std::ptr::null());
        if worker.is_null() || has_icons(worker) {
            return worker;
        }
    }
}
/// A top-level tool window survives changes to Wallpaper Engine's WorkerW tree.
/// Never reparent the Tauri HWND: its runtime expects a top-level window.
pub fn configure_desktop_window(hwnd: isize) -> Result<String, String> {
    unsafe {
        let h = hwnd as Hwnd;
        if IsWindow(h) == 0 {
            return Err("HWND do widget inválido".into());
        }
        let style = GetWindowLongPtrW(h, -16);
        if style & 0x40000000 != 0 {
            return Err("O widget deve ser uma janela independente".into());
        }
        let ex = GetWindowLongPtrW(h, -20);
        SetWindowLongPtrW(h, -20, (ex | 0x80) & !0x40000);
        install_borderless_frame(hwnd)?;
        remove_frame(hwnd)?;
        // Explicitly non-topmost: desktop widgets must remain below regular apps.
        if SetWindowPos(h, -2isize as Hwnd, 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0010) == 0 {
            return Err("Não foi possível configurar a camada do widget".into());
        }
        Ok("Janela independente · compatibilidade com fundos animados".into())
    }
}

/// Clear native caption/buttons and request a non-client recalculation.
/// Some shell/window managers restore these styles after the window is shown.
pub fn remove_frame(hwnd: isize) -> Result<(), String> {
    unsafe {
        let h = hwnd as Hwnd;
        if IsWindow(h) == 0 {
            return Err("Widget indisponível".into());
        }
        let style = GetWindowLongPtrW(h, -16);
        let ex = GetWindowLongPtrW(h, -20);
        let borderless = style & !(0x00c00000 | 0x00040000 | 0x00080000 | 0x00030000);
        let clean_ex = (ex | 0x80) & !(0x00040000 | 0x00000100 | 0x00000200 | 0x00020000 | 0x1);
        if borderless != style || clean_ex != ex {
            SetWindowLongPtrW(h, -16, borderless);
            SetWindowLongPtrW(h, -20, clean_ex);
            if SetWindowPos(
                h,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                0x1 | 0x2 | 0x4 | 0x10 | 0x20,
            ) == 0
            {
                return Err("Não foi possível remover a moldura nativa".into());
            }
        }
        Ok(())
    }
}

/// Insert immediately above the first visible shell surface, below normal apps.
/// Only the widget is repositioned; no Explorer/Wallpaper Engine HWND is mutated.
pub fn maintain_desktop_layer(hwnd: isize) -> Result<(), String> {
    maintain_desktop_layers(&[hwnd])
}
/// Order the widget group together so multiple widgets do not cyclically raise
/// one another on every maintenance tick.
pub fn maintain_desktop_layers(handles: &[isize]) -> Result<(), String> {
    if handles.is_empty() {
        return Ok(());
    }
    for &handle in handles {
        remove_frame(handle)?;
    }
    unsafe {
        let mut current = GetTopWindow(std::ptr::null_mut());
        let mut previous: Hwnd = std::ptr::null_mut();
        let mut count = 0;
        while !current.is_null() && count < 4096 {
            count += 1;
            if !handles.contains(&(current as isize)) && IsWindowVisible(current) != 0 {
                let mut class = [0u16; 256];
                let length = GetClassNameW(current, class.as_mut_ptr(), class.len() as i32);
                let name = String::from_utf16_lossy(&class[..length.max(0) as usize]);
                if name == "Progman" || name == "WorkerW" {
                    let mut after =
                        if previous.is_null() || GetWindowLongPtrW(previous, -20) & 0x8 != 0 {
                            -2isize as Hwnd
                        } else {
                            previous
                        };
                    for &handle in handles {
                        let h = handle as Hwnd;
                        if SetWindowPos(h, after, 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0010 | 0x0200)
                            == 0
                        {
                            return Err("Falha ao ordenar widgets sobre o desktop".into());
                        }
                        after = h;
                    }
                    return Ok(());
                }
                previous = current;
            }
            current = GetWindow(current, 2);
        }
        Err("Superfície do desktop não encontrada; widgets mantidos como janelas normais".into())
    }
}
pub fn place(hwnd: isize, x: i32, y: i32, w: i32, h: i32, radius: i32) -> Result<(), String> {
    unsafe {
        let hwnd = hwnd as Hwnd;
        if IsWindow(hwnd) == 0 {
            return Err("HWND do widget inválido".into());
        }
        if SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            w,
            h,
            0x0004 | 0x0010 | 0x0020,
        ) == 0
        {
            return Err("SetWindowPos falhou".into());
        }
        let region = CreateRoundRectRgn(0, 0, w + 1, h + 1, radius * 2, radius * 2);
        if region.is_null() {
            return Err("CreateRoundRectRgn falhou".into());
        }
        if SetWindowRgn(hwnd, region, 1) == 0 {
            DeleteObject(region);
            return Err("SetWindowRgn falhou".into());
        }
        Ok(())
    }
}
pub fn rect(hwnd: isize) -> Result<[i32; 4], String> {
    unsafe {
        let mut r = Rect::default();
        if IsWindow(hwnd as Hwnd) == 0 || GetWindowRect(hwnd as Hwnd, &mut r) == 0 {
            return Err("Widget indisponível".into());
        }
        Ok([r.left, r.top, r.right - r.left, r.bottom - r.top])
    }
}

/// Top-level HWNDs use screen coordinates. A null parent is valid and expected.
pub fn move_to(hwnd: isize, x: i32, y: i32) -> Result<(), String> {
    unsafe {
        let h = hwnd as Hwnd;
        if IsWindow(h) == 0 {
            return Err("HWND do widget foi destruído".into());
        }
        if SetWindowPos(
            h,
            std::ptr::null_mut(),
            x,
            y,
            0,
            0,
            0x0001 | 0x0004 | 0x0010,
        ) == 0
        {
            return Err(format!(
                "Falha ao mover widget: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
}
pub fn cursor() -> Result<[i32; 2], String> {
    unsafe {
        let mut point = Point { x: 0, y: 0 };
        if GetPhysicalCursorPos(&mut point) == 0 {
            return Err("Não foi possível ler a posição do cursor".into());
        }
        Ok([point.x, point.y])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn CreateWindowExW(
            ex: u32,
            class: *const u16,
            title: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: Hwnd,
            instance: Hwnd,
            parameter: Hwnd,
        ) -> Hwnd;
        fn DestroyWindow(hwnd: Hwnd) -> i32;
        fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
        fn ClientToScreen(hwnd: Hwnd, point: *mut Point) -> i32;
        fn GetWindowRgn(hwnd: Hwnd, region: Hwnd) -> i32;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn PtInRegion(region: Hwnd, x: i32, y: i32) -> i32;
    }
    struct TestWindow(Hwnd);
    impl Drop for TestWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    #[test]
    #[ignore = "Briefly shows a small non-activating color fixture to verify live capture; restores it with RAII"]
    fn live_capture_tracks_controlled_motion(){unsafe{
        #[link(name="user32")]unsafe extern "system"{fn GetDC(hwnd:Hwnd)->Hwnd;fn ReleaseDC(hwnd:Hwnd,dc:Hwnd)->i32;fn FillRect(dc:Hwnd,rect:*const Rect,brush:Hwnd)->i32;fn ShowWindow(hwnd:Hwnd,command:i32)->i32;}
        #[link(name="gdi32")]unsafe extern "system"{fn CreateSolidBrush(color:u32)->Hwnd;}
        let null=std::ptr::null_mut();let h=CreateWindowExW(0x08000088,wide("STATIC").as_ptr(),wide("Liquid capture validation").as_ptr(),0x80000000,40,80,120,80,null,null,null,null);assert!(!h.is_null());let _window=TestWindow(h);
        crate::live_wallpaper::TEST_FIXTURE.store(h as isize,std::sync::atomic::Ordering::Relaxed);
        struct Reset;impl Drop for Reset{fn drop(&mut self){crate::live_wallpaper::TEST_FIXTURE.store(0,std::sync::atomic::Ordering::Relaxed);}}let _reset=Reset;
        ShowWindow(h,4);
        let paint=|color:u32|{let dc=GetDC(h);let brush=CreateSolidBrush(color);let r=Rect{left:0,top:0,right:120,bottom:80};FillRect(dc,&r,brush);DeleteObject(brush);ReleaseDC(h,dc);};
        let sample=|expected:[u8;3]|{for _ in 0..40{paint(u32::from(expected[0])|(u32::from(expected[1])<<8)|(u32::from(expected[2])<<16));std::thread::sleep(std::time::Duration::from_millis(100));if let Ok(frame)=crate::live_wallpaper::test_frame([60,100,20,20]){if frame[20..].chunks_exact(4).any(|p|p[3]==255&&p[..3].iter().zip(expected).all(|(a,b)|a.abs_diff(b)<8)){return true}}}false};
        assert!(sample([230,40,30]),"First live color was not captured");
        let overlay=CreateWindowExW(0x08000088,wide("STATIC").as_ptr(),wide("Excluded glass validation").as_ptr(),0x80000000,40,80,120,80,null,null,null,null);assert!(!overlay.is_null());let _overlay=TestWindow(overlay);
        crate::live_wallpaper::set_excluded(overlay as isize,true).unwrap();ShowWindow(overlay,4);
        let dc=GetDC(overlay);let brush=CreateSolidBrush(0x00ff0000);FillRect(dc,&Rect{left:0,top:0,right:120,bottom:80},brush);DeleteObject(brush);ReleaseDC(overlay,dc);
        assert!(sample([230,40,30]),"Excluded overlay hid the underlying capture");assert!(sample([30,190,90]),"Changed live color was not captured");println!("Live capture observed both controlled colors through the production broker");
    }}
    #[test]
    #[ignore = "Creates a hidden widget HWND and verifies capture exclusion and restoration"]
    fn live_capture_exclusion_roundtrips(){unsafe{
        let null=std::ptr::null_mut();let h=CreateWindowExW(0x80,wide("STATIC").as_ptr(),wide("Liquid capture test").as_ptr(),0x80000000,0,0,200,180,null,null,null,null);assert!(!h.is_null());let _window=TestWindow(h);
        configure_desktop_window(h as isize).unwrap();
        crate::live_wallpaper::set_excluded(h as isize,true).unwrap();
        crate::live_wallpaper::set_excluded(h as isize,false).unwrap();
    }}
    #[test]
    #[ignore = "Creates a hidden HWND and verifies the scanline mask leaves empty gaps clickable"]
    fn scanline_mask_preserves_empty_gaps(){unsafe{
        let null=std::ptr::null_mut();let h=CreateWindowExW(0x80,wide("STATIC").as_ptr(),wide("Liquid mask test").as_ptr(),0x80000000,0,0,120,80,null,null,null,null);assert!(!h.is_null());let _window=TestWindow(h);
        scanline_region(h as isize,&[[10,10,50,12],[70,10,100,12],[10,12,100,14]]).unwrap();
        let region=CreateRectRgn(0,0,0,0);assert_ne!(GetWindowRgn(h,region),0);
        assert_ne!(PtInRegion(region,20,11),0);assert_ne!(PtInRegion(region,80,11),0);assert_eq!(PtInRegion(region,60,11),0);assert_ne!(PtInRegion(region,60,13),0);assert_eq!(PtInRegion(region,0,0),0);DeleteObject(region);
    }}
    #[test]
    #[ignore = "Creates a hidden Top Bar HWND and verifies the real click-through gap"]
    fn topbar_gap_is_absent_from_native_region_and_can_be_closed() {
        unsafe {
            let null=std::ptr::null_mut();
            let h=CreateWindowExW(0x80,wide("STATIC").as_ptr(),wide("Top Bar region test").as_ptr(),0x80000000,0,0,1200,54,null,null,null,null);
            assert!(!h.is_null());let _window=TestWindow(h);
            configure_desktop_window(h as isize).unwrap();
            let region=CreateRectRgn(0,0,0,0);assert!(!region.is_null());
            // 150% DPI: 480 logical units of reserved island space = 720 physical pixels.
            bar_region(h as isize,1200,54,720,21).unwrap();
            assert_ne!(GetWindowRgn(h,region),0);
            assert_ne!(PtInRegion(region,100,27),0);
            assert_ne!(PtInRegion(region,1100,27),0);
            for x in [241,400,600,800,959] {assert_eq!(PtInRegion(region,x,27),0);}
            assert_eq!(PtInRegion(region,0,0),0);
            bar_region(h as isize,1200,54,0,21).unwrap();
            assert_ne!(GetWindowRgn(h,region),0);assert_ne!(PtInRegion(region,600,27),0);
            // A very narrow screen never produces an inverted GDI region.
            bar_region(h as isize,300,36,480,14).unwrap();
            assert_ne!(GetWindowRgn(h,region),0);assert_eq!(PtInRegion(region,150,18),0);
            DeleteObject(region);
        }
    }
    #[test]
    #[ignore = "Creates a hidden island HWND and verifies its native clipping/hit region"]
    fn island_region_excludes_transparent_margins_and_tracks_expansion() {
        unsafe {
            let null = std::ptr::null_mut();
            let h = CreateWindowExW(
                0x80,
                wide("STATIC").as_ptr(),
                wide("Island region test").as_ptr(),
                0x80000000,
                0,
                0,
                480,
                340,
                null,
                null,
                null,
                null,
            );
            assert!(!h.is_null());
            let _window = TestWindow(h);
            configure_desktop_window(h as isize).unwrap();
            let region = CreateRectRgn(0, 0, 0, 0);
            assert!(!region.is_null());
            surface_region(h as isize, 170, 0, 140, 36, 18, false).unwrap();
            assert_ne!(GetWindowRgn(h, region), 0);
            assert_eq!(PtInRegion(region, 10, 10), 0);
            assert_ne!(PtInRegion(region, 240, 18), 0);
            assert_eq!(PtInRegion(region, 240, 100), 0);
            assert_eq!(PtInRegion(region, 170, 0), 0);
            surface_region(h as isize, 20, 0, 440, 306, 32, true).unwrap();
            assert_ne!(GetWindowRgn(h, region), 0);
            assert_ne!(PtInRegion(region, 21, 1), 0);
            assert_ne!(PtInRegion(region, 240, 290), 0);
            assert_eq!(PtInRegion(region, 10, 200), 0);
            surface_region(h as isize, 170, 0, 140, 36, 18, false).unwrap();
            GetWindowRgn(h, region);
            assert_eq!(PtInRegion(region, 240, 100), 0);
            DeleteObject(region);
        }
    }

    #[test]
    #[ignore = "Creates only a hidden independent test HWND"]
    fn independent_widget_moves_without_explorer_parent() {
        let null = std::ptr::null_mut();
        let h = unsafe {
            CreateWindowExW(
                0x80,
                wide("STATIC").as_ptr(),
                wide("Liquid Desktop hidden test").as_ptr(),
                0x80000000,
                100,
                100,
                360,
                220,
                null,
                null,
                null,
                null,
            )
        };
        assert!(!h.is_null(), "Could not create hidden test HWND");
        let _cleanup = TestWindow(h);
        let hwnd = h as isize;
        configure_desktop_window(hwnd).expect("configure independent window");
        unsafe {
            SetWindowLongPtrW(h, -16, GetWindowLongPtrW(h, -16) | 0x00cf0000);
            // Even if an external theme restores caption bits, reserve NO title strip.
            SetWindowPos(h, null, 0, 0, 0, 0, 0x1 | 0x2 | 0x4 | 0x10 | 0x20);
        }
        let outer = rect(hwnd).unwrap();
        let mut client = Rect::default();
        let mut origin = Point { x: 0, y: 0 };
        assert_ne!(unsafe { GetClientRect(h, &mut client) }, 0);
        assert_ne!(unsafe { ClientToScreen(h, &mut origin) }, 0);
        assert_eq!(
            [origin.x, origin.y, client.right, client.bottom],
            outer,
            "regression: caption must consume zero pixels, even when style flags return"
        );
        remove_frame(hwnd).expect("remove restored native caption");
        assert_eq!(unsafe { GetWindowLongPtrW(h, -16) } & 0x00cf0000, 0);
        assert!(
            unsafe { GetParent(h) }.is_null(),
            "regression: a null parent must be supported"
        );
        place(hwnd, 100, 100, 360, 220, 32).expect("initial placement");
        let initial = rect(hwnd).unwrap();
        cursor().expect("physical cursor read");
        move_to(hwnd, initial[0] + 140, initial[1] + 80).expect("drag movement");
        assert_eq!(
            rect(hwnd).unwrap(),
            [initial[0] + 140, initial[1] + 80, initial[2], initial[3]]
        );
        move_to(hwnd, initial[0], initial[1]).expect("cancel movement");
        assert_eq!(rect(hwnd).unwrap(), initial);
    }

    #[test]
    #[ignore = "Requires desktop shell; creates two hidden test HWNDs only"]
    fn multiple_widgets_keep_the_same_relative_order() {
        let null = std::ptr::null_mut();
        let make = || {
            let h = unsafe {
                CreateWindowExW(
                    0x80,
                    wide("STATIC").as_ptr(),
                    wide("Liquid group test").as_ptr(),
                    0x80000000,
                    100,
                    100,
                    240,
                    220,
                    null,
                    null,
                    null,
                    null,
                )
            };
            assert!(!h.is_null());
            configure_desktop_window(h as isize).unwrap();
            TestWindow(h)
        };
        let first = make();
        let second = make();
        let handles = [first.0 as isize, second.0 as isize];
        maintain_desktop_layers(&handles).unwrap();
        assert_eq!(unsafe { GetWindow(first.0, 2) }, second.0);
        maintain_desktop_layers(&handles).unwrap();
        assert_eq!(unsafe { GetWindow(first.0, 2) }, second.0);
        assert_eq!(
            unsafe { GetWindowLongPtrW(first.0, -20) } & 0x8,
            0,
            "must never become topmost"
        );
    }
}
