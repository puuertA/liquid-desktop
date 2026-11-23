//! Reversible visibility control for Explorer's taskbar windows.
//! A small second process watches the app and restores them after a crash.
use std::{
    cell::RefCell,
    ffi::c_void,
    process::{Child, Command},
    time::{SystemTime, UNIX_EPOCH},
};
type Hwnd = isize;
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(callback: unsafe extern "system" fn(Hwnd, isize) -> i32, data: isize) -> i32;
    fn GetClassNameW(hwnd: *mut c_void, text: *mut u16, len: i32) -> i32;
    fn GetWindowThreadProcessId(hwnd: *mut c_void, pid: *mut u32) -> u32;
    fn IsWindow(hwnd: *mut c_void) -> i32;
    fn IsWindowVisible(hwnd: *mut c_void) -> i32;
    fn ShowWindow(hwnd: *mut c_void, command: i32) -> i32;
    fn MonitorFromPoint(point: Point, flags: u32) -> *mut c_void;
    fn MonitorFromWindow(hwnd: *mut c_void, flags: u32) -> *mut c_void;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateEventW(attributes: *const c_void, manual: i32, initial: i32, name: *const u16)
        -> Hwnd;
    fn OpenEventW(access: u32, inherit: i32, name: *const u16) -> Hwnd;
    fn SetEvent(event: Hwnd) -> i32;
    fn WaitForSingleObject(handle: Hwnd, timeout: u32) -> u32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Hwnd;
    fn CloseHandle(handle: Hwnd) -> i32;
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn shell_window(hwnd: Hwnd) -> bool {
    unsafe {
        if IsWindow(hwnd as *mut c_void) == 0 {
            return false;
        }
        let mut buffer = [0u16; 64];
        let len = GetClassNameW(hwnd as *mut c_void, buffer.as_mut_ptr(), 64);
        if len <= 0 {
            return false;
        }
        let class = String::from_utf16_lossy(&buffer[..len as usize]);
        if class != "Shell_TrayWnd" && class != "Shell_SecondaryTrayWnd" {
            return false;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd as *mut c_void, &mut pid);
        crate::taskbar::process_path(pid)
            .to_ascii_lowercase()
            .ends_with("\\explorer.exe")
    }
}
unsafe extern "system" fn enumerate(hwnd: Hwnd, data: isize) -> i32 {
    if shell_window(hwnd) {
        (*(data as *mut Vec<Hwnd>)).push(hwnd)
    }
    1
}
fn windows() -> Vec<Hwnd> {
    let mut result = Vec::new();
    unsafe { EnumWindows(enumerate, &mut result as *mut _ as isize) };
    result
}
fn show(handles: &[Hwnd]) {
    for &h in handles {
        if shell_window(h) {
            unsafe {
                ShowWindow(h as *mut c_void, 5);
            }
        }
    }
}
fn hide(handles: &[Hwnd]) {
    for &h in handles {
        if shell_window(h) {
            unsafe {
                ShowWindow(h as *mut c_void, 0);
            }
        }
    }
}
struct Guardian {
    child: Child,
    handles: Vec<Hwnd>,
    monitor: Hwnd,
}
impl Drop for Guardian {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.try_wait();
    }
}
thread_local! {static GUARD:RefCell<Option<Guardian>>=const{RefCell::new(None)};}

fn spawn_guard(handles: Vec<Hwnd>, monitor: Hwnd) -> Result<Guardian, String> {
    let pid = std::process::id();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let name = format!("Local\\LiquidDesktopTaskbar-{pid}-{nonce}");
    let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, wide(&name).as_ptr()) };
    if event == 0 {
        return Err("Não foi possível criar a proteção da barra do Windows".into());
    }
    let result = (|| {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut child = Command::new(exe)
            .arg("--taskbar-guardian")
            .arg(pid.to_string())
            .arg(&name)
            .args(handles.iter().map(ToString::to_string))
            .spawn()
            .map_err(|e| e.to_string())?;
        if unsafe { WaitForSingleObject(event, 3000) } != 0 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("A proteção da barra do Windows não iniciou".into());
        }
        Ok(Guardian {
            child,
            handles,
            monitor,
        })
    })();
    unsafe { CloseHandle(event) };
    result
}

/// Called only from the Tauri UI thread. Hiding begins after the guardian
/// has opened a handle to this process and acknowledged readiness.
pub fn sync(target_center: Option<[i32; 2]>) -> Result<(), String> {
    GUARD.with(|cell| {
        let mut state = cell.borrow_mut();
        let Some(center) = target_center else {
            if let Some(guard) = state.take() {
                show(&guard.handles)
            }
            return Ok(());
        };
        let target = unsafe {
            MonitorFromPoint(
                Point {
                    x: center[0],
                    y: center[1],
                },
                2,
            ) as Hwnd
        };
        if target == 0 {
            return Err("Monitor da barra indisponível".into());
        }
        if state.as_ref().is_some_and(|g| g.monitor != target) {
            if let Some(previous) = state.take() {
                show(&previous.handles);
            }
        }
        if state
            .as_mut()
            .is_some_and(|g| g.child.try_wait().ok().flatten().is_some())
        {
            if let Some(dead) = state.take() {
                show(&dead.handles)
            }
        }
        let current: Vec<_> = windows()
            .into_iter()
            .filter(|h| unsafe { MonitorFromWindow(*h as *mut c_void, 2) as Hwnd == target })
            .collect();
        let visible: Vec<_> = current
            .iter()
            .copied()
            .filter(|h| unsafe { IsWindowVisible(*h as *mut c_void) != 0 })
            .collect();
        let new: Vec<_> = visible
            .iter()
            .copied()
            .filter(|h| state.as_ref().is_none_or(|g| !g.handles.contains(h)))
            .collect();
        if state.is_none() && visible.is_empty() {
            return Ok(());
        }
        if !new.is_empty() || state.is_none() {
            let mut handles = state.as_ref().map_or_else(Vec::new, |g| {
                g.handles
                    .iter()
                    .copied()
                    .filter(|h| shell_window(*h))
                    .collect()
            });
            for h in &new {
                if !handles.contains(h) {
                    handles.push(*h)
                }
            }
            let guardian = spawn_guard(handles, target)?;
            hide(&visible);
            *state = Some(guardian);
        } else {
            hide(&visible)
        }
        Ok(())
    })
}

/// Entry point for the helper process. Returns true only for its private mode.
pub fn guardian_process(args: &[String]) -> bool {
    if args.get(1).is_none_or(|a| a != "--taskbar-guardian") {
        return false;
    }
    let pid = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let Some(name) = args.get(3) else { return true };
    let handles: Vec<Hwnd> = args
        .iter()
        .skip(4)
        .filter_map(|s| s.parse().ok())
        .take(32)
        .collect();
    if pid == 0 || handles.is_empty() {
        return true;
    }
    unsafe {
        let process = OpenProcess(0x00100000, 0, pid);
        let event = OpenEventW(0x2, 0, wide(name).as_ptr());
        if process == 0 || event == 0 {
            if process != 0 {
                CloseHandle(process);
            }
            if event != 0 {
                CloseHandle(event);
            }
            return true;
        }
        SetEvent(event);
        CloseHandle(event);
        WaitForSingleObject(process, u32::MAX);
        CloseHandle(process);
    }
    show(&handles);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helper_mode_does_not_intercept_normal_launch() {
        assert!(!guardian_process(&["liquid-desktop".into()]));
    }
    #[test]
    fn finder_never_includes_the_current_test_process() {
        for h in windows() {
            assert!(shell_window(h));
        }
    }
}
