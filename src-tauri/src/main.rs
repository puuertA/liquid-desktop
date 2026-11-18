#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod island;
mod topbar;
mod media;
mod monitor;
mod native;
mod settings;
mod startup;
mod wallpaper;
use settings::{valid_id, Layout, Settings, IDS};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

fn show_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
#[tauri::command]
fn hide_panel(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Use o painel".into());
    }
    window.hide().map_err(|e| e.to_string())
}
#[tauri::command]
fn startup_enabled() -> Result<bool, String> {
    startup::enabled()
}
#[tauri::command]
fn set_startup(window: tauri::WebviewWindow, enabled: bool) -> Result<bool, String> {
    if window.label() != "main" {
        return Err("Use o painel".into());
    }
    startup::set(enabled)
}
#[tauri::command]
fn set_theme_sync(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Use o painel".into());
    }
    valid_id(&id)?;
    let state = app.state::<State>();
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    next.sync_theme = enabled;
    if enabled {
        next.sync_material(&id)?;
    }
    persist(&app, &next)?;
    *layout = next.clone();
    drop(layout);
    for id in IDS {
        publish(&app, id, &next)?;
    }
    Ok(())
}

struct State {
    layout: Mutex<Layout>,
    dragging: Mutex<Option<DragSession>>,
    monitor: Mutex<Option<monitor::Monitor>>,
    load_error: Option<String>,
}
struct DragSession {
    id: String,
    hwnd: isize,
    origin: [i32; 4],
    cursor: [i32; 2],
}
fn handle(w: &tauri::WebviewWindow) -> Result<isize, String> {
    Ok(w.hwnd().map_err(|e| e.to_string())?.0 as isize)
}
fn storage_ready(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(error) = &app.state::<State>().load_error {
        return Err(error.clone());
    }
    Ok(())
}
fn persist(app: &tauri::AppHandle, layout: &Layout) -> Result<(), String> {
    storage_ready(app)?;
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join("settings.json");
    if target.exists() && !dir.join("settings-before-widgets.json").exists() {
        std::fs::copy(&target, dir.join("settings-before-widgets.json"))
            .map_err(|e| e.to_string())?;
    }
    let temp = dir.join("settings.tmp");
    std::fs::write(
        &temp,
        serde_json::to_vec_pretty(layout).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&temp, &target).map_err(|e| e.to_string())
}
fn publish(app: &tauri::AppHandle, id: &str, layout: &Layout) -> Result<(), String> {
    app.emit_to(id, "settings", &layout.widgets[id])
        .map_err(|e| e.to_string())?;
    app.emit_to("main", "layout", layout)
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn get_layout(app: tauri::AppHandle) -> Result<Layout, String> {
    storage_ready(&app)?;
    Ok(app.state::<State>().layout.lock().unwrap().clone())
}
#[tauri::command]
fn get_settings(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Settings, String> {
    valid_id(window.label())?;
    Ok(app.state::<State>().layout.lock().unwrap().widgets[window.label()].clone())
}
#[tauri::command]
async fn get_wallpaper() -> Result<wallpaper::Wallpaper, String> {
    tauri::async_runtime::spawn_blocking(wallpaper::read_cached)
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn system_snapshot(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<monitor::Snapshot, String> {
    if window.label() != "system" {
        return Err("Provedor reservado ao monitor do sistema".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<State>();
        let mut monitor = state.monitor.lock().unwrap();
        monitor.get_or_insert_with(monitor::Monitor::new).sample()
    })
    .await
    .map_err(|e| e.to_string())
}
fn place_settings(window: &tauri::WebviewWindow, settings: &Settings) -> Result<(), String> {
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    native::place(
        handle(window)?,
        settings.x,
        settings.y,
        (settings.width as f64 * scale).round() as i32,
        (settings.height as f64 * scale).round() as i32,
        (32.0 * scale).round() as i32,
    )
}
#[tauri::command]
fn update_widget(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
    patch: serde_json::Value,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Configuração disponível no painel".into());
    }
    valid_id(&id)?;
    let state = app.state::<State>();
    if state
        .dragging
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|d| d.id == id)
    {
        return Err("Solte o widget antes de configurar".into());
    }
    let mut layout = state.layout.lock().unwrap();
    let mut value = serde_json::to_value(&layout.widgets[&id]).map_err(|e| e.to_string())?;
    for (key, item) in patch.as_object().ok_or("Configuração inválida")? {
        if ![
            "x",
            "y",
            "width",
            "height",
            "refraction",
            "opacity",
            "edit",
            "static_wallpaper",
            "tint",
            "shine",
            "shade",
            "show_seconds",
            "hour24",
            "material",
            "thickness",
            "bezel",
            "dispersion",
            "frost",
            "quality",
            "motion",
        ]
        .contains(&key.as_str())
        {
            return Err("Campo de configuração desconhecido".into());
        }
        value[key] = item.clone();
    }
    let next: Settings = serde_json::from_value(value).map_err(|e| e.to_string())?;
    next.validate()?;
    let old = &layout.widgets[&id];
    if (old.x, old.y, old.width, old.height) != (next.x, next.y, next.width, next.height) {
        if let Some(w) = app.get_webview_window(&id) {
            place_settings(&w, &next)?;
        }
    }
    let mut updated = layout.clone();
    updated.widgets.insert(id.clone(), next);
    if updated.sync_theme {
        updated.sync_material(&id)?;
    }
    persist(&app, &updated)?;
    *layout = updated.clone();
    drop(layout);
    if updated.sync_theme {
        for id in IDS {
            publish(&app, id, &updated)?;
        }
        Ok(())
    } else {
        publish(&app, &id, &updated)
    }
}
fn create_window(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    valid_id(id)?;
    storage_ready(app)?;
    if app.get_webview_window(id).is_some() {
        return Ok(());
    }
    let mut s = app.state::<State>().layout.lock().unwrap().widgets[id].clone();
    let main = app
        .get_webview_window("main")
        .ok_or("Painel indisponível")?;
    let monitors = main.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .iter()
        .find(|m| {
            let p = m.position();
            let z = m.size();
            s.x >= p.x && s.y >= p.y && s.x < p.x + z.width as i32 && s.y < p.y + z.height as i32
        })
        .or_else(|| monitors.first())
        .ok_or("Nenhum monitor ativo")?;
    let scale = monitor.scale_factor();
    let p = monitor.position();
    let z = monitor.size();
    let width = (s.width as f64 * scale).round() as i32;
    let height = (s.height as f64 * scale).round() as i32;
    s.x = s.x.clamp(p.x, (p.x + z.width as i32 - width).max(p.x));
    s.y = s.y.clamp(p.y, (p.y + z.height as i32 - height).max(p.y));
    let w = tauri::WebviewWindowBuilder::new(
        app,
        id,
        tauri::WebviewUrl::App(format!("index.html?widget={id}").into()),
    )
    .title(format!("Liquid {id}"))
    .inner_size(s.width as f64, s.height as f64)
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;
    let window = w.clone();
    let position = s.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let result = (|| {
            let h = handle(&window)?;
            native::configure_desktop_window(h)?;
            native::place(
                h,
                position.x,
                position.y,
                width,
                height,
                (32.0 * scale).round() as i32,
            )?;
            window.show().map_err(|e| e.to_string())?;
            let _ = native::maintain_desktop_layer(h);
            Ok::<(), String>(())
        })();
        let _ = sender.send(result);
    })
    .map_err(|e| e.to_string())?;
    if let Err(error) = receiver.recv().map_err(|e| e.to_string())? {
        let _ = w.close();
        return Err(error);
    }
    let state = app.state::<State>();
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    let entry = next.widgets.get_mut(id).unwrap();
    entry.enabled = true;
    entry.x = s.x;
    entry.y = s.y;
    if let Err(error) = persist(app, &next) {
        let _ = w.close();
        return Err(error);
    }
    *layout = next.clone();
    drop(layout);
    publish(app, id, &next)
}
#[tauri::command]
async fn create_widget(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Use o painel para adicionar widgets".into());
    }
    tauri::async_runtime::spawn_blocking(move || create_window(&app, &id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn remove_widget(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Use o painel para remover widgets".into());
    }
    valid_id(&id)?;
    let state = app.state::<State>();
    {
        let mut drag = state.dragging.lock().unwrap();
        if drag.as_ref().is_some_and(|d| d.id == id) {
            drag.take();
        }
    }
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    next.widgets.get_mut(&id).unwrap().enabled = false;
    persist(&app, &next)?;
    *layout = next.clone();
    drop(layout);
    if let Some(w) = app.get_webview_window(&id) {
        w.close().map_err(|e| e.to_string())?
    }
    if id == "system" {
        state.monitor.lock().unwrap().take();
    }
    publish(&app, &id, &next)
}
#[tauri::command]
fn save_note(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    text: String,
) -> Result<(), String> {
    if window.label() != "notes" {
        return Err("Janela de notas inválida".into());
    }
    if text.len() > 32_000 {
        return Err("Limite de 32 KB por nota".into());
    }
    let state = app.state::<State>();
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    next.widgets.get_mut("notes").unwrap().note = text;
    persist(&app, &next)?;
    *layout = next.clone();
    drop(layout);
    publish(&app, "notes", &next)
}
#[tauri::command]
fn widget_rect(window: tauri::WebviewWindow) -> Result<[i32; 4], String> {
    valid_id(window.label())?;
    native::rect(handle(&window)?)
}
#[tauri::command]
fn begin_drag(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
    let id = window.label();
    valid_id(id)?;
    let state = app.state::<State>();
    if !state.layout.lock().unwrap().widgets[id].edit {
        return Err("Ative o modo edição no painel".into());
    }
    let mut session = state.dragging.lock().unwrap();
    if session.is_some() {
        return Err("Já existe um arraste em andamento".into());
    }
    let hwnd = handle(&window)?;
    *session = Some(DragSession {
        id: id.into(),
        hwnd,
        origin: native::rect(hwnd)?,
        cursor: native::cursor()?,
    });
    Ok(())
}
#[tauri::command]
fn update_drag(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
    let id = window.label();
    valid_id(id)?;
    let state = app.state::<State>();
    if !state.layout.lock().unwrap().widgets[id].edit {
        return Err("Modo edição desativado".into());
    }
    let session = state.dragging.lock().unwrap();
    let drag = session
        .as_ref()
        .filter(|d| d.id == id)
        .ok_or("Arraste não iniciado")?;
    let cursor = native::cursor()?;
    native::move_to(
        drag.hwnd,
        drag.origin[0] + cursor[0] - drag.cursor[0],
        drag.origin[1] + cursor[1] - drag.cursor[1],
    )?;
    app.emit_to(id, "surface-moved", ())
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn finish_drag(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    cancel: bool,
) -> Result<(), String> {
    let id = window.label();
    valid_id(id)?;
    let state = app.state::<State>();
    let drag = {
        let mut session = state.dragging.lock().unwrap();
        if !session.as_ref().is_some_and(|d| d.id == id) {
            return Ok(());
        }
        session.take().unwrap()
    };
    if cancel {
        native::move_to(drag.hwnd, drag.origin[0], drag.origin[1])?;
        app.emit_to(id, "surface-moved", ())
            .map_err(|e| e.to_string())?;
    }
    let rect = native::rect(drag.hwnd)?;
    let mut layout = state.layout.lock().unwrap();
    let mut next = layout.clone();
    let entry = next.widgets.get_mut(id).unwrap();
    entry.x = rect[0];
    entry.y = rect[1];
    persist(&app, &next)?;
    *layout = next.clone();
    drop(layout);
    publish(&app, id, &next)
}
fn main() {
    if std::env::args().any(|a| a == "--diagnose") {
        let report = match wallpaper::read() {
            Ok(w) => {
                serde_json::json!({"compatible_host":native::probe(),"wallpaper_mode":w.mode,"monitors":w.monitors.iter().map(|m|serde_json::json!({"rect":m.rect,"texture_loaded":m.image.is_some(),"error":m.error})).collect::<Vec<_>>()})
            }
            Err(e) => serde_json::json!({"error":e}),
        };
        std::fs::write(
            "liquid-desktop-diagnostics.json",
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .expect("Falha ao gravar diagnóstico");
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_panel(app)
        }))
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            let open = MenuItem::with_id(app, "open", "Abrir painel", true, None::<&str>)?;
            let quit =
                MenuItem::with_id(app, "quit", "Sair do Liquid Desktop", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            // An opaque, high contrast ring remains visible on light and dark taskbars.
            let mut pixels = vec![0u8; 32 * 32 * 4];
            for y in 0..32 {
                for x in 0..32 {
                    let d = (x as f64 - 15.5).hypot(y as f64 - 15.5);
                    if d < 14.0 {
                        let color = if d > 10.0 {
                            [164, 219, 239, 255]
                        } else {
                            [27, 47, 61, 255]
                        };
                        pixels[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4].copy_from_slice(&color);
                    }
                }
            }
            TrayIconBuilder::with_id("liquid-desktop")
                .icon(tauri::image::Image::new_owned(pixels, 32, 32))
                .tooltip("Liquid Desktop — abrir painel")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_panel(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        show_panel(tray.app_handle());
                    }
                })
                .build(app)?;
            if !std::env::args().any(|a| a == "--background") {
                show_panel(app.handle());
            }
            let loaded = app
                .path()
                .app_config_dir()
                .map_err(|e| e.to_string())
                .and_then(|p| match std::fs::read(p.join("settings.json")) {
                    Ok(bytes) => Layout::load(&bytes),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Layout::default()),
                    Err(e) => Err(e.to_string()),
                });
            let (layout, load_error) = match loaded {
                Ok(l) => (l, None),
                Err(e) => (
                    Layout::default(),
                    Some(format!(
                        "Preferências não carregadas; arquivo original preservado: {e}"
                    )),
                ),
            };
            let restore: Vec<String> = layout
                .widgets
                .iter()
                .filter(|(_, s)| s.enabled)
                .map(|(id, _)| id.clone())
                .collect();
            app.manage(State {
                layout: Mutex::new(layout),
                dragging: Mutex::new(None),
                monitor: Mutex::new(None),
                load_error,
            });
            app.manage(island::Runtime::default());
            app.manage(topbar::Runtime::default());
            let restore_app = app.handle().clone();
            std::thread::spawn(move || {
                {
                    let runtime = restore_app.state::<island::Runtime>();
                    let _lifecycle = runtime.lifecycle.lock().unwrap();
                    let restore_island = restore_app
                        .state::<State>()
                        .layout
                        .lock()
                        .unwrap()
                        .island
                        .enabled;
                    if restore_island {
                        if let Err(error) = island::create(&restore_app) {
                            let _ = restore_app.emit_to("main", "widget-error", error);
                        }
                    }
                }
                {
                    let runtime=restore_app.state::<topbar::Runtime>();
                    let _lifecycle=runtime.lifecycle.lock().unwrap();
                    if let Err(error)=topbar::create(&restore_app) {
                        let _=restore_app.emit_to("main","widget-error",error);
                    }
                }
                for id in restore {
                    if let Err(error) = create_window(&restore_app, &id) {
                        let _ =
                            restore_app.emit_to("main", "widget-error", format!("{id}: {error}"));
                    }
                }
            });
            let app_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let task_app = app_handle.clone();
                if app_handle
                    .run_on_main_thread(move || {
                        let handles: Vec<isize> = IDS
                            .iter()
                            .filter_map(|id| task_app.get_webview_window(id))
                            .filter_map(|w| handle(&w).ok())
                            .collect();
                        let _ = native::maintain_desktop_layers(&handles);
                        island::maintain(&task_app);
                        let _=topbar::maintain(&task_app);
                    })
                    .is_err()
                {
                    break;
                }
            });
            Ok(())
        })
        .on_window_event(|w, event| {
            if w.label() == "main" {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                    tauri::WindowEvent::Resized(_) if w.is_minimized().unwrap_or(false) => {
                        let _ = w.hide();
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            island::get_island_config,
            topbar::topbar_snapshot,
            topbar::topbar_volume,
            topbar::topbar_open_panel,
            topbar::update_topbar,
            island::island_monitors,
            island::update_island,
            island::island_snapshot,
            island::island_media_action,
            island::island_volume,
            island::island_timer,
            island::island_region,
            hide_panel,
            startup_enabled,
            set_startup,
            set_theme_sync,
            get_layout,
            get_settings,
            get_wallpaper,
            system_snapshot,
            update_widget,
            create_widget,
            remove_widget,
            save_note,
            widget_rect,
            begin_drag,
            update_drag,
            finish_drag
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar Liquid Desktop");
}
