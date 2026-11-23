//! A separate, reversible taskbar surface. Explorer remains the shell.
use crate::{handle, media, native, persist, State};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::c_void,
    path::Path,
    sync::Mutex,
};
use tauri::{Emitter, Manager};

type Hwnd = *mut c_void;
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct MonitorInfo {
    size: u32,
    monitor: Rect,
    work: Rect,
    flags: u32,
}
#[repr(C)]
#[derive(Default)]
struct PowerRaw {
    ac: u8,
    flags: u8,
    percent: u8,
    saver: u8,
    remaining: u32,
    total: u32,
}
#[repr(C)]
struct ShellFileInfo {
    icon: Hwnd,
    index: i32,
    attributes: u32,
    name: [u16; 260],
    kind: [u16; 80],
}
impl Default for ShellFileInfo {
    fn default() -> Self {
        Self {
            icon: std::ptr::null_mut(),
            index: 0,
            attributes: 0,
            name: [0; 260],
            kind: [0; 80],
        }
    }
}
#[repr(C)]
struct BitmapHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits: u16,
    compression: u32,
    size_image: u32,
    xppm: i32,
    yppm: i32,
    used: u32,
    important: u32,
}
#[repr(C)]
struct BitmapInfo {
    header: BitmapHeader,
    colors: [u32; 1],
}
#[repr(C)]
struct WindowsVersion {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform: u32,
    csd: [u16; 128],
    sp_major: u16,
    sp_minor: u16,
    suite: u16,
    product: u8,
    reserved: u8,
}
#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetVersion(version: *mut WindowsVersion) -> i32;
}
#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(callback: unsafe extern "system" fn(isize, isize) -> i32, data: isize) -> i32;
    fn IsWindowVisible(hwnd: Hwnd) -> i32;
    fn IsWindow(hwnd: Hwnd) -> i32;
    fn IsIconic(hwnd: Hwnd) -> i32;
    fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
    fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, len: i32) -> i32;
    fn GetClassNameW(hwnd: Hwnd, text: *mut u16, len: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn GetWindow(hwnd: Hwnd, command: u32) -> Hwnd;
    fn GetWindowThreadProcessId(hwnd: Hwnd, pid: *mut u32) -> u32;
    fn GetForegroundWindow() -> Hwnd;
    fn ShowWindowAsync(hwnd: Hwnd, command: i32) -> i32;
    fn SetForegroundWindow(hwnd: Hwnd) -> i32;
    fn PostMessageW(hwnd: Hwnd, message: u32, w: usize, l: isize) -> i32;
    fn MonitorFromPoint(point: Point, flags: u32) -> Hwnd;
    fn GetMonitorInfoW(monitor: Hwnd, info: *mut MonitorInfo) -> i32;
    fn keybd_event(key: u8, scan: u8, flags: u32, extra: usize);
    fn DrawIconEx(
        dc: Hwnd,
        x: i32,
        y: i32,
        icon: Hwnd,
        w: i32,
        h: i32,
        step: u32,
        brush: Hwnd,
        flags: u32,
    ) -> i32;
    fn DestroyIcon(icon: Hwnd) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
    fn QueryFullProcessImageNameW(process: isize, flags: u32, name: *mut u16, len: *mut u32)
        -> i32;
    fn CloseHandle(handle: isize) -> i32;
    fn GetSystemPowerStatus(status: *mut PowerRaw) -> i32;
}
#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmGetWindowAttribute(hwnd: Hwnd, attribute: u32, value: *mut c_void, size: u32) -> i32;
}
#[link(name = "shell32")]
unsafe extern "system" {
    fn SHGetFileInfoW(
        path: *const u16,
        attributes: u32,
        info: *mut ShellFileInfo,
        size: u32,
        flags: u32,
    ) -> usize;
    fn ShellExecuteW(
        hwnd: isize,
        operation: *const u16,
        file: *const u16,
        args: *const u16,
        directory: *const u16,
        show: i32,
    ) -> isize;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: Hwnd) -> Hwnd;
    fn DeleteDC(dc: Hwnd) -> i32;
    fn CreateDIBSection(
        dc: Hwnd,
        info: *const BitmapInfo,
        usage: u32,
        bits: *mut *mut c_void,
        section: Hwnd,
        offset: u32,
    ) -> Hwnd;
    fn SelectObject(dc: Hwnd, object: Hwnd) -> Hwnd;
    fn DeleteObject(object: Hwnd) -> i32;
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn windows_11() -> bool {
    unsafe {
        let mut v: WindowsVersion = std::mem::zeroed();
        v.size = std::mem::size_of::<WindowsVersion>() as u32;
        RtlGetVersion(&mut v) == 0 && v.build >= 22000
    }
}
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub mode: String,
    #[serde(default)]
    pub mode_migrated: bool,
    pub monitor: usize,
    pub width: String,
    pub height: u32,
    pub radius: u32,
    pub material: String,
    pub opacity: f64,
    pub refraction: f64,
    pub icon_size: u32,
    pub magnification: f64,
    pub auto_hide: bool,
    pub hide_fullscreen: bool,
    pub show_labels: bool,
    pub show_search: bool,
    pub show_task_view: bool,
    pub show_weather: bool,
    pub show_system: bool,
    pub show_clock: bool,
    pub show_bluetooth: bool,
    pub show_clipboard: bool,
    pub show_media: bool,
    pub windows_auto_hide: bool,
    pub segment_order: Vec<String>,
    pub pinned: Vec<String>,
    pub hidden_pins: Vec<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "experimental".into(),
            mode_migrated: true,
            monitor: 0,
            width: "adaptive".into(),
            height: 56,
            radius: 10,
            material: "shared".into(),
            opacity: 0.18,
            refraction: 1.0,
            icon_size: 30,
            magnification: 0.26,
            auto_hide: false,
            hide_fullscreen: true,
            show_labels: false,
            show_search: true,
            show_task_view: true,
            show_weather: false,
            show_system: true,
            show_clock: true,
            show_bluetooth: false,
            show_clipboard: false,
            show_media: false,
            windows_auto_hide: false,
            segment_order: vec!["start".into(), "apps".into(), "system".into()],
            pinned: Vec::new(),
            hidden_pins: Vec::new(),
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !["coexist", "overlay", "experimental"].contains(&self.mode.as_str())
            || !["compact", "adaptive", "full"].contains(&self.width.as_str())
            || !["shared", "clear", "regular", "frosted", "dark", "adaptive"]
                .contains(&self.material.as_str())
            || self.monitor > 15
            || !(40..=96).contains(&self.height)
            || !(8..=48).contains(&self.radius)
            || !(20..=60).contains(&self.icon_size)
            || !self.opacity.is_finite()
            || !(0.0..=0.7).contains(&self.opacity)
            || !self.refraction.is_finite()
            || !(0.0..=2.0).contains(&self.refraction)
            || !self.magnification.is_finite()
            || !(0.0..=0.7).contains(&self.magnification)
        {
            return Err("Configuração da barra de tarefas inválida".into());
        }
        let mut order = self.segment_order.clone();
        order.sort();
        if order != ["apps", "start", "system"] {
            return Err("Ordem dos segmentos inválida".into());
        }
        if self.pinned.len() > 40
            || self
                .pinned
                .iter()
                .any(|p| p.len() > 32768 || !valid_launch_path(p))
            || self.hidden_pins.len() > 40
            || self
                .hidden_pins
                .iter()
                .any(|p| p.len() > 32768 || !valid_launch_path(p))
        {
            return Err("Aplicativo fixado inválido".into());
        }
        Ok(())
    }
}
fn valid_launch_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    Path::new(path).is_absolute() && (lower.ends_with(".exe") || lower.ends_with(".lnk"))
}
#[derive(Default)]
pub struct Runtime {
    pub lifecycle: Mutex<()>,
    placement: Mutex<Option<[i32; 5]>>,
    icons: Mutex<HashMap<String, Option<String>>>,
}
#[derive(Clone, Serialize)]
pub struct App {
    pub id: String,
    pub label: String,
    pub path: String,
    pub icon: Option<String>,
    pub pinned: bool,
    pub running: bool,
    pub active: bool,
    pub minimized: bool,
    pub windows: Vec<usize>,
    pub window_titles: Vec<String>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub config: Config,
    pub apps: Vec<App>,
    pub volume: Option<media::Volume>,
    pub battery: Option<u8>,
    pub charging: bool,
    pub network: bool,
    pub media: Option<String>,
}
fn allowed(w: &tauri::WebviewWindow) -> Result<(), String> {
    if w.label() == "taskbar" {
        Ok(())
    } else {
        Err("Comando reservado à barra de tarefas".into())
    }
}
pub(crate) fn process_path(pid: u32) -> String {
    unsafe {
        let h = OpenProcess(0x1000, 0, pid);
        if h == 0 {
            return String::new();
        }
        let mut data = [0u16; 32768];
        let mut len = data.len() as u32;
        let ok = QueryFullProcessImageNameW(h, 0, data.as_mut_ptr(), &mut len);
        CloseHandle(h);
        if ok == 0 {
            String::new()
        } else {
            String::from_utf16_lossy(&data[..len as usize])
        }
    }
}
unsafe extern "system" fn enumerate(raw: isize, data: isize) -> i32 {
    let hwnd = raw as Hwnd;
    if IsWindowVisible(hwnd) == 0 || GetWindowTextLengthW(hwnd) <= 0 {
        return 1;
    }
    let ex = GetWindowLongPtrW(hwnd, -20);
    if ex & 0x80 != 0 && ex & 0x40000 == 0 {
        return 1;
    }
    if !GetWindow(hwnd, 4).is_null() && ex & 0x40000 == 0 {
        return 1;
    }
    let mut cloak = 0u32;
    if DwmGetWindowAttribute(hwnd, 14, (&mut cloak as *mut u32).cast(), 4) == 0 && cloak != 0 {
        return 1;
    }
    let mut class = [0u16; 256];
    let n = GetClassNameW(hwnd, class.as_mut_ptr(), 256).max(0) as usize;
    if [
        "Progman",
        "WorkerW",
        "Shell_TrayWnd",
        "Shell_SecondaryTrayWnd",
        "Windows.UI.Core.CoreWindow",
    ]
    .contains(&String::from_utf16_lossy(&class[..n]).as_str())
    {
        return 1;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == 0 || pid == std::process::id() {
        return 1;
    }
    let mut title = [0u16; 512];
    let n = GetWindowTextW(hwnd, title.as_mut_ptr(), 512).max(0) as usize;
    let path = process_path(pid);
    if path.is_empty() {
        return 1;
    }
    let list = &mut *(data as *mut Vec<(usize, String, String, bool)>);
    list.push((
        hwnd as usize,
        path,
        String::from_utf16_lossy(&title[..n]),
        IsIconic(hwnd) != 0,
    ));
    1
}
fn windows() -> Vec<(usize, String, String, bool)> {
    let mut result = Vec::new();
    unsafe { EnumWindows(enumerate, &mut result as *mut _ as isize) };
    result
}
fn imported_pins() -> Vec<String> {
    let Some(base) = std::env::var_os("APPDATA") else {
        return Vec::new();
    };
    let path =
        Path::new(&base).join(r"Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar");
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("lnk"))
        })
        .map(|e| e.path().to_string_lossy().into_owned())
        .take(40)
        .collect()
}
fn label(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Aplicativo".into())
}
fn icon(path: &str) -> Option<String> {
    unsafe {
        let mut info = ShellFileInfo::default();
        let found = SHGetFileInfoW(
            wide(path).as_ptr(),
            0,
            &mut info,
            std::mem::size_of::<ShellFileInfo>() as u32,
            0x100 | 0x0,
        );
        if found == 0 || info.icon.is_null() {
            return None;
        }
        let dc = CreateCompatibleDC(std::ptr::null_mut());
        if dc.is_null() {
            DestroyIcon(info.icon);
            return None;
        }
        let bitmap = BitmapInfo {
            header: BitmapHeader {
                size: 40,
                width: 40,
                height: -40,
                planes: 1,
                bits: 32,
                compression: 0,
                size_image: 6400,
                xppm: 0,
                yppm: 0,
                used: 0,
                important: 0,
            },
            colors: [0],
        };
        let mut pixels = std::ptr::null_mut();
        let bmp = CreateDIBSection(dc, &bitmap, 0, &mut pixels, std::ptr::null_mut(), 0);
        if bmp.is_null() {
            DeleteDC(dc);
            DestroyIcon(info.icon);
            return None;
        }
        std::ptr::write_bytes(pixels, 0, 6400);
        let old = SelectObject(dc, bmp);
        let ok = DrawIconEx(dc, 0, 0, info.icon, 40, 40, 0, std::ptr::null_mut(), 3);
        let mut rgba = vec![0u8; 6400];
        if ok != 0 && !pixels.is_null() {
            let raw = std::slice::from_raw_parts(pixels as *const u8, 6400);
            for (dst, src) in rgba.chunks_exact_mut(4).zip(raw.chunks_exact(4)) {
                dst.copy_from_slice(&[src[2], src[1], src[0], src[3]])
            }
        }
        SelectObject(dc, old);
        DeleteObject(bmp);
        DeleteDC(dc);
        DestroyIcon(info.icon);
        if ok == 0 {
            return None;
        }
        if rgba.chunks_exact(4).all(|p| p[3] == 0) {
            for p in rgba.chunks_exact_mut(4) {
                if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                    p[3] = 255
                }
            }
        }
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, 40, 40);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().ok()?.write_image_data(&rgba).ok()?;
        }
        Some(format!("data:image/png;base64,{}", STANDARD.encode(png)))
    }
}
fn collect(config: &Config, runtime: &Runtime) -> Vec<App> {
    let mut apps = BTreeMap::<String, App>::new();
    let pins = config
        .pinned
        .iter()
        .chain(imported_pins().iter())
        .filter(|p| !config.hidden_pins.iter().any(|h| h.eq_ignore_ascii_case(p)))
        .cloned()
        .collect::<Vec<_>>();
    for path in pins {
        let id = path.to_lowercase();
        apps.entry(id.clone()).or_insert_with(|| App {
            id,
            label: label(&path),
            path,
            icon: None,
            pinned: true,
            running: false,
            active: false,
            minimized: false,
            windows: Vec::new(),
            window_titles: Vec::new(),
        });
    }
    let foreground = unsafe { GetForegroundWindow() } as usize;
    for (hwnd, path, title, minimized) in windows() {
        let id = path.to_lowercase();
        let app = apps.entry(id.clone()).or_insert_with(|| App {
            id,
            label: label(&path),
            path,
            icon: None,
            pinned: false,
            running: false,
            active: false,
            minimized: false,
            windows: Vec::new(),
            window_titles: Vec::new(),
        });
        if app.label == label(&app.path) && !title.is_empty() {
            app.label = title.clone()
        }
        app.running = true;
        app.active |= hwnd == foreground;
        app.minimized |= minimized;
        app.windows.push(hwnd);
        app.window_titles.push(title);
    }
    let mut list = apps.into_values().collect::<Vec<_>>();
    let mut icons = runtime.icons.lock().unwrap();
    for app in &mut list {
        app.icon = icons
            .entry(app.path.clone())
            .or_insert_with(|| icon(&app.path))
            .clone()
    }
    list.sort_by_key(|a| {
        let p = config
            .pinned
            .iter()
            .position(|x| x.eq_ignore_ascii_case(&a.path))
            .or_else(|| {
                imported_pins()
                    .iter()
                    .position(|x| x.eq_ignore_ascii_case(&a.path))
            });
        (p.is_none(), p.unwrap_or(999), a.label.to_lowercase())
    });
    list
}
#[tauri::command]
pub async fn taskbar_snapshot(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<Snapshot, String> {
    allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<State>();
        let config = state.layout.lock().unwrap().taskbar.clone();
        let apps = collect(&config, &app.state::<Runtime>());
        let apartment = media::Apartment::new().ok();
        let volume = apartment
            .as_ref()
            .and_then(|_| media::volume(None, None).ok());
        let mut power = PowerRaw::default();
        let battery = if unsafe { GetSystemPowerStatus(&mut power) } != 0 && power.percent <= 100 {
            Some(power.percent)
        } else {
            None
        };
        let network =
            windows::Networking::Connectivity::NetworkInformation::GetInternetConnectionProfile()
                .and_then(|p| p.GetNetworkConnectivityLevel())
                .is_ok_and(|v| v.0 >= 3);
        let media = apartment
            .and_then(|_| {
                app.state::<crate::island::Runtime>()
                    .provider
                    .lock()
                    .unwrap()
                    .snapshot(None)
                    .ok()
            })
            .and_then(|s| s.track.map(|t| t.title));
        Ok(Snapshot {
            config,
            apps,
            volume,
            battery,
            charging: power.flags != 255 && power.flags & 8 != 0,
            network,
            media,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
fn launch(path: &str) -> Result<(), String> {
    if !valid_launch_path(path) || !Path::new(path).is_file() {
        return Err("Aplicativo indisponível".into());
    }
    let result = unsafe {
        ShellExecuteW(
            0,
            wide("open").as_ptr(),
            wide(path).as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    };
    if result <= 32 {
        Err("Não foi possível abrir o aplicativo".into())
    } else {
        Ok(())
    }
}
#[tauri::command]
pub fn taskbar_app_action(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
    action: String,
) -> Result<(), String> {
    allowed(&window)?;
    let config = app.state::<State>().layout.lock().unwrap().taskbar.clone();
    let found = collect(&config, &app.state::<Runtime>())
        .into_iter()
        .find(|a| a.id == id)
        .ok_or("Aplicativo não encontrado")?;
    match action.as_str() {
        "launch" => launch(&found.path),
        "focus" | "minimize" | "close" => {
            let hwnd = found
                .windows
                .iter()
                .copied()
                .find(|h| *h == unsafe { GetForegroundWindow() } as usize)
                .or_else(|| found.windows.first().copied())
                .ok_or("Janela não está aberta")? as Hwnd;
            if unsafe { IsWindow(hwnd) } == 0 {
                return Err("Janela fechada".into());
            }
            unsafe {
                match action.as_str() {
                    "minimize" => {
                        ShowWindowAsync(hwnd, 6);
                    }
                    "close" => {
                        PostMessageW(hwnd, 0x10, 0, 0);
                    }
                    _ => {
                        if IsIconic(hwnd) != 0 {
                            ShowWindowAsync(hwnd, 9);
                        }
                        SetForegroundWindow(hwnd);
                    }
                }
            }
            Ok(())
        }
        _ => Err("Ação inválida".into()),
    }
}
#[tauri::command]
pub fn taskbar_window_focus(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
    hwnd: usize,
) -> Result<(), String> {
    allowed(&window)?;
    let config = app.state::<State>().layout.lock().unwrap().taskbar.clone();
    let found = collect(&config, &app.state::<Runtime>())
        .into_iter()
        .find(|a| a.id == id)
        .ok_or("Aplicativo não encontrado")?;
    if !found.windows.contains(&hwnd) || unsafe { IsWindow(hwnd as Hwnd) } == 0 {
        return Err("Janela não está aberta".into());
    }
    unsafe {
        if IsIconic(hwnd as Hwnd) != 0 {
            ShowWindowAsync(hwnd as Hwnd, 9);
        }
        SetForegroundWindow(hwnd as Hwnd);
    }
    Ok(())
}
#[tauri::command]
pub fn taskbar_shortcut(window: tauri::WebviewWindow, action: String) -> Result<(), String> {
    allowed(&window)?;
    let keys: &[u8] = match action.as_str() {
        "start" => &[0x11, 0x1b],
        "search" => &[0x5b, 0x53],
        "task_view" => &[0x5b, 0x09],
        "notifications" => {
            if windows_11() {
                &[0x5b, 0x4e]
            } else {
                &[0x5b, 0x41]
            }
        }
        "quick_settings" => &[0x5b, 0x41],
        "clipboard" => &[0x5b, 0x56],
        _ => return Err("Atalho inválido".into()),
    };
    unsafe {
        for k in keys {
            keybd_event(*k, 0, 0, 0)
        }
        for k in keys.iter().rev() {
            keybd_event(*k, 0, 2, 0)
        }
    }
    Ok(())
}
#[tauri::command]
pub fn taskbar_pin(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    path: String,
    pinned: bool,
) -> Result<(), String> {
    allowed(&window)?;
    if !valid_launch_path(&path) || !Path::new(&path).is_file() {
        return Err("Selecione um arquivo .exe ou .lnk válido".into());
    }
    let state = app.state::<State>();
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    next.taskbar
        .pinned
        .retain(|p| !p.eq_ignore_ascii_case(&path));
    next.taskbar
        .hidden_pins
        .retain(|p| !p.eq_ignore_ascii_case(&path));
    if pinned {
        next.taskbar.pinned.push(path)
    } else {
        next.taskbar.hidden_pins.push(path)
    }
    next.taskbar.validate()?;
    persist(&app, &next)?;
    *layout = next;
    app.emit_to("main", "layout", layout.clone())
        .map_err(|e| e.to_string())?;
    app.emit_to("taskbar", "taskbar-refresh", ())
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn taskbar_panel_pin(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    path: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Use o painel".into());
    }
    if !valid_launch_path(&path) || !Path::new(&path).is_file() {
        return Err("Selecione um arquivo .exe ou .lnk válido".into());
    }
    let state = app.state::<State>();
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    next.taskbar
        .pinned
        .retain(|p| !p.eq_ignore_ascii_case(&path));
    next.taskbar
        .hidden_pins
        .retain(|p| !p.eq_ignore_ascii_case(&path));
    next.taskbar.pinned.push(path);
    next.taskbar.validate()?;
    persist(&app, &next)?;
    *layout = next;
    app.emit_to("main", "layout", layout.clone())
        .map_err(|e| e.to_string())?;
    app.emit_to("taskbar", "taskbar-refresh", ())
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn taskbar_reorder(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    paths: Vec<String>,
) -> Result<(), String> {
    allowed(&window)?;
    if paths.len() > 40
        || paths
            .iter()
            .any(|p| !valid_launch_path(p) || !Path::new(p).is_file())
    {
        return Err("Ordem inválida".into());
    }
    let state = app.state::<State>();
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    next.taskbar.pinned = paths;
    next.taskbar.validate()?;
    persist(&app, &next)?;
    *layout = next;
    app.emit_to("main", "layout", layout.clone())
        .map_err(|e| e.to_string())?;
    app.emit_to("taskbar", "taskbar-refresh", ())
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn update_taskbar(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    config: Config,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Use o painel".into());
    }
    config.validate()?;
    tauri::async_runtime::spawn_blocking(move || {
        let runtime = app.state::<Runtime>();
        let _lock = runtime.lifecycle.lock().unwrap();
        let state = app.state::<State>();
        let old = {
            let mut layout = state.layout.lock().unwrap();
            let old = layout.taskbar.clone();
            let mut next = layout.clone();
            next.taskbar = config.clone();
            persist(&app, &next)?;
            *layout = next;
            old
        };
        let result = (|| {
            if config.enabled {
                create(&app)?;
                let copy = app.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                app.run_on_main_thread(move || {
                    let _ = tx.send(maintain(&copy));
                })
                .map_err(|e| e.to_string())?;
                rx.recv().map_err(|e| e.to_string())??
            } else {
                if let Some(w) = app.get_webview_window("taskbar") {
                    w.close().map_err(|e| e.to_string())?
                }
                restore_shell(&app)?
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            let save_error = {
                let mut layout = state.layout.lock().unwrap();
                layout.taskbar = old.clone();
                persist(&app, &layout).err()
            };
            let recovery = if old.enabled {
                create(&app).and_then(|_| {
                    let copy = app.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    app.run_on_main_thread(move || {
                        let _ = tx.send(maintain(&copy));
                    })
                    .map_err(|e| e.to_string())?;
                    rx.recv().map_err(|e| e.to_string())?
                })
            } else {
                if let Some(w) = app.get_webview_window("taskbar") {
                    let _ = w.close();
                }
                restore_shell(&app)
            };
            let mut failure = match recovery {
                Ok(()) => error,
                Err(e) => format!("{error}; recuperação: {e}"),
            };
            if let Some(e) = save_error {
                failure.push_str(&format!("; preferências: {e}"));
            }
            return Err(failure);
        }
        app.emit_to("taskbar", "taskbar-refresh", ())
            .map_err(|e| e.to_string())?;
        let saved = state.layout.lock().unwrap().clone();
        app.emit_to("main", "layout", saved)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
fn monitor_work(x: i32, y: i32) -> Option<Rect> {
    unsafe {
        let m = MonitorFromPoint(Point { x, y }, 2);
        let mut info = MonitorInfo {
            size: std::mem::size_of::<MonitorInfo>() as u32,
            monitor: Rect::default(),
            work: Rect::default(),
            flags: 0,
        };
        (GetMonitorInfoW(m, &mut info) != 0).then_some(info.work)
    }
}
fn placement(
    config: &Config,
    origin: [i32; 2],
    size: [i32; 2],
    work_bottom: i32,
    scale: f64,
) -> [i32; 5] {
    let h = ((config.height + 270) as f64 * scale).round() as i32;
    let margin = (8.0 * scale).round() as i32;
    let bottom = if config.mode == "coexist" {
        work_bottom
    } else {
        origin[1] + size[1]
    };
    [origin[0], bottom - h - margin, size[0], h, 1]
}
fn restore_shell(app: &tauri::AppHandle) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let result = crate::shell_taskbar::sync(None);
        crate::workarea::shell_auto_hide(0, false);
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}
pub fn maintain(app: &tauri::AppHandle) -> Result<(), String> {
    let Some(w) = app.get_webview_window("taskbar") else {
        crate::shell_taskbar::sync(None)?;
        return Ok(());
    };
    let config = app.state::<State>().layout.lock().unwrap().taskbar.clone();
    let monitors = w.available_monitors().map_err(|e| e.to_string())?;
    let m = monitors
        .get(config.monitor)
        .or_else(|| monitors.first())
        .ok_or("Monitor indisponível")?;
    let pos = m.position();
    let size = m.size();
    if config.enabled && config.mode == "experimental" {
        crate::workarea::shell_auto_hide(0, false);
        crate::shell_taskbar::sync(Some([pos.x + size.width as i32 / 2, pos.y + size.height as i32 / 2]))?;
    } else {
        crate::shell_taskbar::sync(None)?;
        crate::workarea::shell_auto_hide(
            handle(&w)?,
            config.enabled && config.mode == "overlay" && config.windows_auto_hide,
        );
    }
    let scale = m.scale_factor();
    let work_bottom =
        monitor_work(pos.x + 2, pos.y + 2).map_or(pos.y + size.height as i32, |r| r.bottom);
    let desired = placement(
        &config,
        [pos.x, pos.y],
        [size.width as i32, size.height as i32],
        work_bottom,
        scale,
    );
    let runtime = app.state::<Runtime>();
    let mut previous = runtime.placement.lock().unwrap();
    let hwnd = handle(&w)?;
    if previous.as_ref() != Some(&desired) || native::rect(hwnd)? != desired[..4] {
        native::position_surface(
            hwnd,
            desired[0],
            desired[1],
            desired[2],
            desired[3],
            desired[4] == 1,
        )?;
        *previous = Some(desired);
        let _ = app.emit_to("taskbar", "taskbar-placement", ());
    } else {
        native::keep_surface_above(hwnd)?
    }
    let hidden = config.hide_fullscreen
        && native::foreground_fullscreen([pos.x, pos.y, size.width as i32, size.height as i32]);
    if hidden {
        if w.is_visible().unwrap_or(false) {
            w.hide().map_err(|e| e.to_string())?
        }
    } else {
        if w.is_minimized().unwrap_or(false) {
            w.unminimize().map_err(|e| e.to_string())?
        }
        if !w.is_visible().unwrap_or(true) {
            w.show().map_err(|e| e.to_string())?
        }
    }
    Ok(())
}
#[tauri::command]
pub fn taskbar_region(window: tauri::WebviewWindow, rects: Vec<[f64; 4]>) -> Result<(), String> {
    allowed(&window)?;
    if rects.len() > 8
        || rects
            .iter()
            .flat_map(|r| r.iter())
            .any(|v| !v.is_finite() || v.abs() > 32768.0)
    {
        return Err("Região inválida".into());
    }
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let hwnd = handle(&window)?;
    let bounds = native::rect(hwnd)?;
    let mut rows = Vec::new();
    for rect in rects {
        let x = (rect[0] * scale).round() as i32;
        let y = (rect[1] * scale).round() as i32;
        let w = (rect[2] * scale).round() as i32;
        let h = (rect[3] * scale).round() as i32;
        if x < 0 || y < 0 || w < 0 || h < 0 || x + w > bounds[2] + 1 || y + h > bounds[3] + 1 {
            return Err("Região fora da barra".into());
        }
        rows.push([x, y, x + w, y + h]);
    }
    let radius = window
        .app_handle()
        .state::<State>()
        .layout
        .lock()
        .unwrap()
        .taskbar
        .radius;
    native::segmented_region(hwnd, &rows, (radius as f64 * scale).round() as i32)
}
pub fn create(app: &tauri::AppHandle) -> Result<(), String> {
    if app.get_webview_window("taskbar").is_some()
        || !app.state::<State>().layout.lock().unwrap().taskbar.enabled
    {
        return Ok(());
    }
    let w = tauri::WebviewWindowBuilder::new(
        app,
        "taskbar",
        tauri::WebviewUrl::App("index.html?surface=taskbar".into()),
    )
    .title("Liquid Taskbar")
    .inner_size(1200.0, 80.0)
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .visible(false)
    .focused(false)
    .build()
    .map_err(|e| e.to_string())?;
    let copy = app.clone();
    let win = w.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let posted = app.run_on_main_thread(move || {
        let result = (|| {
            native::configure_desktop_window(handle(&win)?)?;
            native::segmented_region(handle(&win)?, &[], 24)?;
            *copy.state::<Runtime>().placement.lock().unwrap() = None;
            maintain(&copy)
        })();
        let _ = tx.send(result);
    });
    let result = posted
        .map_err(|e| e.to_string())
        .and_then(|_| rx.recv().map_err(|e| e.to_string()))
        .and_then(|r| r);
    if result.is_err() {
        let _ = w.close();
        let _ = restore_shell(app);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_validation() {
        let mut c = Config::default();
        assert!(!c.enabled);
        assert_eq!(c.mode, "experimental");
        assert!(c.validate().is_ok());
        c.mode = "replace-shell".into();
        assert!(c.validate().is_err());
        c = Config::default();
        c.segment_order.swap(0, 1);
        assert!(c.validate().is_ok());
        c.segment_order.pop();
        assert!(c.validate().is_err());
    }
    #[test]
    fn migration_preserves_other_surfaces() {
        let layout=crate::Layout::load(br##"{"version":2,"widgets":{"notes":{"note":"preserved"}},"topbar":{"enabled":true}}"##).unwrap();
        assert!(!layout.taskbar.enabled);
        assert!(layout.topbar.enabled);
        assert_eq!(layout.widgets["notes"].note, "preserved");
        let restored = crate::Layout::load(&serde_json::to_vec(&layout).unwrap()).unwrap();
        assert_eq!(restored.taskbar.mode, "experimental");
    }
    #[test]
    fn monitor_placement_handles_dpi_and_negative_origins() {
        let mut c = Config::default();
        c.mode = "coexist".into();
        assert_eq!(
            placement(&c, [-1920, -100], [1920, 1080], 930, 1.5),
            [-1920, 429, 1920, 489, 1]
        );
        c.mode = "overlay".into();
        assert_eq!(
            placement(&c, [-1920, -100], [1920, 1080], 930, 1.5),
            [-1920, 479, 1920, 489, 1]
        );
    }
    #[test]
    fn enumeration_has_no_internal_windows() {
        for (_, path, _, _) in windows() {
            assert!(!path.is_empty())
        }
    }
    #[test]
    fn shell_icon_is_encoded_as_png() {
        let exe = std::env::current_exe().unwrap();
        let image = icon(exe.to_str().unwrap()).expect("Windows shell icon");
        let data = STANDARD
            .decode(image.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        assert_eq!(&data[..8], b"\x89PNG\r\n\x1a\n");
    }
}
