use crate::{handle, media, native, persist, State};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub form: String,
    pub appearance: String,
    pub above_apps: bool,
    pub monitor: usize,
    pub motion: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            form: "floating".into(),
            appearance: "black".into(),
            above_apps: true,
            monitor: 0,
            motion: true,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !["floating", "notch"].contains(&self.form.as_str())
            || !["liquid", "black", "graphite", "adaptive"].contains(&self.appearance.as_str())
            || self.monitor > 15
        {
            return Err("Configuração da ilha inválida".into());
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct Runtime {
    pub provider: Mutex<media::Provider>,
    pub timer: Mutex<Option<Instant>>,
    pub lifecycle: Mutex<()>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub media: Option<media::MediaSnapshot>,
    pub media_error: Option<String>,
    pub volume: Option<media::Volume>,
    pub volume_error: Option<String>,
    pub remaining: Option<u64>,
}
pub fn allowed(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "island" {
        Ok(())
    } else {
        Err("Comando reservado à Dynamic Island".into())
    }
}
#[tauri::command]
pub fn get_island_config(app: tauri::AppHandle) -> Config {
    app.state::<State>().layout.lock().unwrap().island.clone()
}
#[derive(Serialize)]
pub struct MonitorChoice {
    index: usize,
    name: String,
}
#[tauri::command]
pub fn island_monitors(window: tauri::WebviewWindow) -> Result<Vec<MonitorChoice>, String> {
    if window.label() != "main" {
        return Err("Use o painel".into());
    }
    Ok(window
        .available_monitors()
        .map_err(|e| e.to_string())?
        .iter()
        .enumerate()
        .map(|(index, m)| MonitorChoice {
            index,
            name: format!(
                "{} · {}×{}",
                m.name()
                    .cloned()
                    .unwrap_or_else(|| format!("Monitor {}", index + 1)),
                m.size().width,
                m.size().height
            ),
        })
        .collect())
}
#[tauri::command]
pub async fn island_snapshot(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    selected: Option<String>,
) -> Result<Snapshot, String> {
    allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _apartment = media::Apartment::new()?;
        let runtime = app.state::<Runtime>();
        let media = runtime
            .provider
            .lock()
            .unwrap()
            .snapshot(selected.as_deref());
        let volume = media::volume(None, None);
        let remaining = runtime.timer.lock().unwrap().as_ref().map(|at| {
            ((at.saturating_duration_since(Instant::now()).as_millis() + 999) / 1000) as u64
        });
        Ok(Snapshot {
            media: media.as_ref().ok().cloned(),
            media_error: media.err(),
            volume: volume.as_ref().ok().cloned(),
            volume_error: volume.err(),
            remaining,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn island_media_action(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
    action: String,
    position: Option<f64>,
) -> Result<(), String> {
    allowed(&window)?;
    if !["toggle", "previous", "next", "seek"].contains(&action.as_str())
        || position.is_some_and(|p| !p.is_finite() || p < 0.0 || p > 604800.0)
    {
        return Err("Comando inválido".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let _apartment = media::Apartment::new()?;
        app.state::<Runtime>()
            .provider
            .lock()
            .unwrap()
            .action(&id, &action, position)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn island_volume(
    window: tauri::WebviewWindow,
    level: Option<f32>,
    muted: Option<bool>,
) -> Result<media::Volume, String> {
    allowed(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _apartment = media::Apartment::new()?;
        media::volume(level, muted)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn island_timer(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    seconds: Option<u64>,
) -> Result<(), String> {
    allowed(&window)?;
    if seconds.is_some_and(|n| n == 0 || n > 86400) {
        return Err("Use um tempo entre 1 segundo e 24 horas".into());
    }
    *app.state::<Runtime>().timer.lock().unwrap() =
        seconds.map(|n| Instant::now() + Duration::from_secs(n));
    Ok(())
}
#[tauri::command]
pub fn island_region(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    width: f64,
    height: f64,
    radius: f64,
) -> Result<(), String> {
    allowed(&window)?;
    if !width.is_finite()
        || !height.is_finite()
        || !radius.is_finite()
        || !(100.0..=460.0).contains(&width)
        || !(28.0..=330.0).contains(&height)
        || !(0.0..=80.0).contains(&radius)
    {
        return Err("Geometria inválida".into());
    }
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let notch = app.state::<State>().layout.lock().unwrap().island.form == "notch";
    native::surface_region(
        handle(&window)?,
        ((480.0 - width) * 0.5 * scale).round() as i32,
        0,
        (width * scale).round() as i32,
        (height * scale).round() as i32,
        (radius * scale).round() as i32,
        notch,
    )
}
pub fn place(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    config: &Config,
) -> Result<(), String> {
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .get(config.monitor)
        .or_else(|| monitors.first())
        .ok_or("Nenhum monitor disponível")?;
    let scale = monitor.scale_factor();
    let pos = monitor.position();
    let size = monitor.size();
    native::position_surface(
        handle(window)?,
        pos.x + (size.width as i32 - (480.0 * scale) as i32) / 2,
        pos.y
            + if config.form == "notch" {
                0
            } else {
                (12.0 * scale) as i32
            },
        (480.0 * scale) as i32,
        (340.0 * scale) as i32,
        config.above_apps,
    )?;
    let _ = app;
    Ok(())
}
pub fn create(app: &tauri::AppHandle) -> Result<(), String> {
    if app.get_webview_window("island").is_some() {
        return Ok(());
    }
    let config = app.state::<State>().layout.lock().unwrap().island.clone();
    if !config.enabled {
        return Ok(());
    }
    let window = tauri::WebviewWindowBuilder::new(
        app,
        "island",
        tauri::WebviewUrl::App("index.html?surface=island".into()),
    )
    .title("Liquid Island")
    .inner_size(480.0, 340.0)
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .visible(false)
    .focused(false)
    .build()
    .map_err(|e| e.to_string())?;
    let w = window.clone();
    let app2 = app.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let result = (|| {
            native::configure_desktop_window(handle(&w)?)?;
            place(&app2, &w, &config)?;
            let scale = w.scale_factor().map_err(|e| e.to_string())?;
            native::surface_region(
                handle(&w)?,
                (170.0 * scale) as i32,
                0,
                (140.0 * scale) as i32,
                (36.0 * scale) as i32,
                (18.0 * scale) as i32,
                config.form == "notch",
            )?;
            maintain(&app2);
            Ok::<(), String>(())
        })();
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;
    if let Err(error) = rx.recv().map_err(|e| e.to_string())? {
        let _ = window.close();
        return Err(error);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn legacy_acrylic_setting_is_discarded_without_losing_layout() {
        let layout = crate::settings::Layout::load(
            br#"{"version":2,"island":{"enabled":true,"appearance":"liquid","native_blur":true},"widgets":{"notes":{"note":"Saved note","x":210}}}"#,
        ).unwrap();
        assert!(layout.island.enabled);
        assert_eq!(layout.island.appearance, "liquid");
        assert_eq!(layout.widgets["notes"].note, "Saved note");
        assert_eq!(layout.widgets["notes"].x, 210);
        let saved = serde_json::to_value(&layout).unwrap();
        assert!(saved["island"].get("native_blur").is_none());
        assert!(crate::settings::Layout::load(&serde_json::to_vec(&saved).unwrap()).is_ok());
    }

    #[test]
    fn migration_preserves_widgets_and_island_is_opt_in() {
        let layout = crate::settings::Layout::load(
            br#"{"version":2,"widgets":{"notes":{"note":"Saved note","x":210}}}"#,
        )
        .unwrap();
        assert!(!layout.island.enabled);
        assert_eq!(layout.widgets["notes"].note, "Saved note");
        assert_eq!(layout.widgets["notes"].x, 210);
        let mut next = layout.clone();
        next.island.enabled = true;
        next.island.form = "notch".into();
        let restored = crate::settings::Layout::load(&serde_json::to_vec(&next).unwrap()).unwrap();
        assert!(restored.island.enabled);
        assert_eq!(restored.island.form, "notch");
        next.island.monitor = 99;
        assert!(crate::settings::Layout::load(&serde_json::to_vec(&next).unwrap()).is_err());
    }
}
pub fn maintain(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("island") else {
        return;
    };
    let config = app.state::<State>().layout.lock().unwrap().island.clone();
    if let Ok(monitors) = window.available_monitors() {
        if let Some(monitor) = monitors.get(config.monitor).or_else(|| monitors.first()) {
            let p = monitor.position();
            let s = monitor.size();
            let scale = monitor.scale_factor();
            let fullscreen =
                native::foreground_fullscreen([p.x, p.y, s.width as i32, s.height as i32]);
            if fullscreen {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                }
                return;
            }
            if !window.is_visible().unwrap_or(true) {
                let _ = window.show();
            }
            let expected = [
                p.x + (s.width as i32 - (480.0 * scale) as i32) / 2,
                p.y + if config.form == "notch" {
                    0
                } else {
                    (12.0 * scale) as i32
                },
                (480.0 * scale) as i32,
                (340.0 * scale) as i32,
            ];
            if handle(&window)
                .ok()
                .and_then(|h| native::rect(h).ok())
                .is_some_and(|rect| rect != expected)
            {
                let _ = place(app, &window, &config);
            }
        }
    }
}
#[tauri::command]
pub async fn update_island(
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
        let _lifecycle = runtime.lifecycle.lock().unwrap();
        let state = app.state::<State>();
        let old;
        {
            let mut layout = state.layout.lock().unwrap();
            old = layout.island.clone();
            let mut next = layout.clone();
            next.island = config.clone();
            persist(&app, &next)?;
            *layout = next;
        }
        let result = if config.enabled {
            create(&app)
        } else {
            if let Some(w) = app.get_webview_window("island") {
                w.close().map_err(|e| e.to_string())?;
            }
            *runtime.timer.lock().unwrap() = None;
            *runtime.provider.lock().unwrap() = media::Provider::default();
            Ok(())
        };
        if let Err(error) = result {
            let mut layout = state.layout.lock().unwrap();
            layout.island = old;
            persist(&app, &layout)?;
            return Err(error);
        }
        if let Some(w) = app.get_webview_window("island") {
            let app2 = app.clone();
            let conf = config.clone();
            app.run_on_main_thread(move || {
                let _ = place(&app2, &w, &conf);
            })
            .map_err(|e| e.to_string())?;
        }
        app.emit_to("island", "island-config", &config)
            .map_err(|e| e.to_string())?;
        let copy=app.clone();
        app.run_on_main_thread(move ||{let _=crate::topbar::maintain(&copy);}).map_err(|e|e.to_string())?;
        app.emit_to("topbar", "topbar-refresh", ()).map_err(|e|e.to_string())?;
        let layout = state.layout.lock().unwrap().clone();
        app.emit_to("main", "layout", &layout)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
