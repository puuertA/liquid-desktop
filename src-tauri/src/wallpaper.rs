use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::{
    core::PCWSTR,
    Win32::{System::Com::*, UI::Shell::*},
};
#[derive(Clone, Serialize)]
pub struct Monitor {
    pub id: String,
    pub rect: [i32; 4],
    pub image: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct Wallpaper {
    pub mode: i32,
    pub color: u32,
    pub monitors: Vec<Monitor>,
}
static CACHE: OnceLock<Mutex<Option<(Instant, Wallpaper)>>> = OnceLock::new();
/// One COM/file read for all requesting widgets in a 15-second interval.
pub fn read_cached() -> Result<Wallpaper, String> {
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|e| e.to_string())?;
    if let Some((at, wallpaper)) = cache.as_ref() {
        if at.elapsed() < Duration::from_secs(15) {
            return Ok(wallpaper.clone());
        }
    }
    let wallpaper = read()?;
    *cache = Some((Instant::now(), wallpaper.clone()));
    Ok(wallpaper)
}
struct ComGuard;
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}
unsafe fn take_string(p: windows::core::PWSTR) -> String {
    let s = p.to_string().unwrap_or_default();
    CoTaskMemFree(Some(p.0.cast()));
    s
}
pub fn read() -> Result<Wallpaper, String> {
    std::thread::spawn(|| unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        let _guard = ComGuard;
        let desktop: IDesktopWallpaper =
            CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        let mode = desktop.GetPosition().map_err(|e| e.to_string())?.0;
        let color = desktop.GetBackgroundColor().map_err(|e| e.to_string())?.0;
        let count = desktop
            .GetMonitorDevicePathCount()
            .map_err(|e| e.to_string())?;
        let mut monitors = Vec::new();
        for i in 0..count {
            let id = take_string(
                desktop
                    .GetMonitorDevicePathAt(i)
                    .map_err(|e| e.to_string())?,
            );
            let wide: Vec<u16> = id.encode_utf16().chain(Some(0)).collect();
            let ptr = PCWSTR(wide.as_ptr());
            let r = desktop.GetMonitorRECT(ptr).map_err(|e| e.to_string())?;
            let path = take_string(desktop.GetWallpaper(ptr).map_err(|e| e.to_string())?);
            let (image, error) = if path.is_empty() {
                (None, None)
            } else {
                match std::fs::read(&path) {
                    Ok(bytes) if bytes.len() <= 40 * 1024 * 1024 => {
                        let ext = std::path::Path::new(&path)
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        let mime = match ext.as_str() {
                            "png" => "image/png",
                            "bmp" => "image/bmp",
                            "webp" => "image/webp",
                            _ => "image/jpeg",
                        };
                        (
                            Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes))),
                            None,
                        )
                    }
                    Ok(_) => (None, Some("Wallpaper excede 40 MB".into())),
                    Err(e) => (None, Some(e.to_string())),
                }
            };
            monitors.push(Monitor {
                id,
                rect: [r.left, r.top, r.right - r.left, r.bottom - r.top],
                image,
                error,
            });
        }
        Ok(Wallpaper {
            mode,
            color,
            monitors,
        })
    })
    .join()
    .map_err(|_| "Falha na thread COM".to_string())?
}
