use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct CalendarEvent {pub id:String,pub date:String,pub time:String,pub title:String,pub reminder:bool}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub x: i32,
    pub y: i32,
    pub refraction: f64,
    pub opacity: f64,
    pub edit: bool,
    pub static_wallpaper: bool,
    pub tint: String,
    pub shine: f64,
    pub shade: f64,
    pub width: u32,
    pub height: u32,
    pub enabled: bool,
    pub show_seconds: bool,
    pub hour24: bool,
    pub note: String,
    pub material: String,
    pub thickness: f64,
    pub bezel: f64,
    pub dispersion: f64,
    pub frost: f64,
    pub quality: String,
    pub motion: bool,
    pub media_layout: String,
    pub media_visualizer: bool,
    pub weather_city: String,
    pub weather_latitude: Option<f64>,
    pub weather_longitude: Option<f64>,
    pub weather_unit: String,
    pub weather_refresh: u32,
    pub gallery_interval: u32,
    pub gallery_fit: String,
    pub gallery_random: bool,
    pub gallery_frame: bool,
    pub storage_devices: Vec<String>,
    pub system_layout: String,
    pub system_metrics: Vec<String>,
    pub calendar_events: Vec<CalendarEvent>,
    pub notes_style: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            refraction: 18.0,
            opacity: 0.12,
            edit: false,
            static_wallpaper: false,
            tint: "#b9d5e8".into(),
            shine: 0.65,
            shade: 0.42,
            width: 360,
            height: 220,
            enabled: false,
            show_seconds: false,
            hour24: true,
            note: String::new(),
            material: "custom".into(),
            thickness: 18.0,
            bezel: 26.0,
            dispersion: 0.1,
            frost: 0.35,
            quality: "balanced".into(),
            motion: true,
            media_layout: "horizontal".into(), media_visualizer: false,
            weather_city: String::new(), weather_latitude: None, weather_longitude: None,
            weather_unit: "celsius".into(), weather_refresh: 30,
            gallery_interval: 10, gallery_fit: "cover".into(), gallery_random: false, gallery_frame: true,
            storage_devices: Vec::new(), system_layout: "expanded".into(),
            system_metrics: vec!["cpu".into(),"memory".into(),"network".into()],
            calendar_events: Vec::new(), notes_style: "plain".into(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if ![
            "custom", "clear", "thin", "regular", "frosted", "dark", "adaptive", "liquid",
        ]
        .contains(&self.material.as_str())
            || !["economy", "balanced", "high"].contains(&self.quality.as_str())
            || !self.thickness.is_finite()
            || !(4.0..=40.0).contains(&self.thickness)
            || !self.bezel.is_finite()
            || !(8.0..=48.0).contains(&self.bezel)
            || !self.dispersion.is_finite()
            || !(0.0..=0.5).contains(&self.dispersion)
            || !self.frost.is_finite()
            || !(0.0..=8.0).contains(&self.frost)
        {
            return Err("Material óptico fora do intervalo permitido".into());
        }
        if !(200..=640).contains(&self.width)
            || !(180..=600).contains(&self.height)
            || self.note.len() > 32_000
        {
            return Err("Tamanho do widget ou nota fora do limite".into());
        }
        if self.tint.len() != 7
            || !self.tint.starts_with('#')
            || !self.tint[1..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("Use uma cor no formato #RRGGBB".into());
        }
        if !self.shine.is_finite()
            || !(0.0..=1.0).contains(&self.shine)
            || !self.shade.is_finite()
            || !(0.2..=0.75).contains(&self.shade)
        {
            return Err("Intensidade do vidro fora do intervalo".into());
        }
        if !(-1_000_000..=1_000_000).contains(&self.x)
            || !(-1_000_000..=1_000_000).contains(&self.y)
        {
            return Err("Coordenadas fora do intervalo permitido".into());
        }
        if !self.refraction.is_finite()
            || !(0.0..=40.0).contains(&self.refraction)
            || !self.opacity.is_finite()
            || !(0.0..=0.6).contains(&self.opacity)
        {
            return Err("Parâmetros fora do intervalo permitido".into());
        }
        if !["compact","horizontal","expanded"].contains(&self.media_layout.as_str())
            || !["celsius","fahrenheit"].contains(&self.weather_unit.as_str())
            || self.weather_city.len()>120
            || self.weather_latitude.is_some_and(|v|!v.is_finite()||!(-90.0..=90.0).contains(&v))
            || self.weather_longitude.is_some_and(|v|!v.is_finite()||!(-180.0..=180.0).contains(&v))
            || !(5..=180).contains(&self.weather_refresh)
            || !(3..=3600).contains(&self.gallery_interval)
            || !["cover","contain"].contains(&self.gallery_fit.as_str())
            || self.storage_devices.len()>32 || self.storage_devices.iter().any(|s|s.len()>200)
            || !["compact","expanded"].contains(&self.system_layout.as_str())
            || self.system_metrics.len()>8 || self.system_metrics.iter().any(|s|!["cpu","memory","network","frequency"].contains(&s.as_str()))
            || !["plain","paper","minimal"].contains(&self.notes_style.as_str())
            || self.calendar_events.len()>200
            || self.calendar_events.iter().any(|e| e.id.len()>50||e.title.is_empty()||e.title.len()>160||e.date.len()!=10||!e.date.bytes().all(|b|b.is_ascii_digit()||b==b'-')||e.time.len()>5||!e.time.bytes().all(|b|b.is_ascii_digit()||b==b':'))
        {return Err("Configuracao do widget invalida".into())}
        Ok(())
    }
}

pub const IDS: [&str; 9] = ["clock", "analog", "calendar", "notes", "system", "media", "weather", "gallery", "storage"];
pub fn valid_id(id: &str) -> Result<(), String> {
    if IDS.contains(&id) {
        Ok(())
    } else {
        Err("Widget desconhecido".into())
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Layout {
    pub version: u32,
    #[serde(default = "default_glass_source")]
    pub glass_source: String,
    #[serde(default)]
    pub sync_theme: bool,
    #[serde(default)]
    pub island: crate::island::Config,
    #[serde(default)]
    pub topbar: crate::topbar::Config,
    #[serde(default)]
    pub fusion: crate::fusion::Config,
    pub widgets: std::collections::BTreeMap<String, Settings>,
}
fn default_glass_source()->String {"clock".into()}
impl Default for Layout {
    fn default() -> Self {
        let mut widgets = std::collections::BTreeMap::new();
        for (i, id) in IDS.iter().enumerate() {
            let mut s = Settings::default();
            s.x = 60 + (i as i32 % 3) * 440;
            s.y = 80 + (i as i32 / 3) * 300;
            match *id {
                "analog" => {
                    s.width = 240;
                    s.height = 260;
                    s.show_seconds = true;
                }
                "calendar" => {
                    s.width = 380;
                    s.height = 320;
                }
                "notes" => {
                    s.width = 360;
                    s.height = 280;
                }
                "system" => {
                    s.width = 380;
                    s.height = 340;
                }
                "media" => {s.width=380;s.height=240;}
                "weather" => {s.width=400;s.height=360;}
                "gallery" => {s.width=400;s.height=300;}
                "storage" => {s.width=380;s.height=300;}
                _ => {}
            }
            widgets.insert(id.to_string(), s);
        }
        Self {
            version: 2,
            glass_source: default_glass_source(),
            sync_theme: true,
            island: crate::island::Config::default(),
            topbar: crate::topbar::Config::default(),
            fusion: crate::fusion::Config::default(),
            widgets,
        }
    }
}
impl Layout {
    pub fn sync_material(&mut self, source: &str) -> Result<(), String> {
        valid_id(source)?;
        self.glass_source=source.into();
        let material = self.widgets[source].clone();
        for widget in self.widgets.values_mut() {
            widget.tint = material.tint.clone();
            widget.opacity = material.opacity;
            widget.shine = material.shine;
            widget.shade = material.shade;
            widget.refraction = material.refraction;
            widget.static_wallpaper = material.static_wallpaper;
            widget.material = material.material.clone();
            widget.thickness = material.thickness;
            widget.bezel = material.bezel;
            widget.dispersion = material.dispersion;
            widget.frost = material.frost;
            widget.quality = material.quality.clone();
            widget.motion = material.motion;
        }
        Ok(())
    }
    pub fn load(bytes: &[u8]) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut layout = Self::default();
        if value.get("widgets").is_some() {
            let shared_material=value.get("glass_source").is_some();
            let saved: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
            let legacy_source=saved.widgets.iter().find(|(_,s)|s.enabled).or_else(||saved.widgets.iter().next()).map(|(id,_)|id.clone()).unwrap_or_else(default_glass_source);
            if saved.version != 2 {
                return Err("Versão de layout não suportada".into());
            }
            layout.sync_theme = saved.sync_theme;
            valid_id(&saved.glass_source)?;
            layout.glass_source=saved.glass_source;
            saved.island.validate()?;
            layout.island = saved.island;
            saved.topbar.validate()?;
            layout.topbar = saved.topbar;
            saved.fusion.validate()?;
            layout.fusion=saved.fusion;
            for (id, settings) in saved.widgets {
                valid_id(&id)?;
                settings.validate()?;
                layout.widgets.insert(id, settings);
            }
            if !shared_material {
                // First shared-material migration keeps an active widget's existing glass.
                layout.sync_theme=true;
                layout.sync_material(&legacy_source)?;
            } else if layout.sync_theme {
                let source=layout.glass_source.clone();
                layout.sync_material(&source)?;
            }
        } else {
            let mut clock: Settings = serde_json::from_value(value).map_err(|e| e.to_string())?;
            clock.validate()?;
            clock.enabled = true;
            layout.widgets.insert("clock".into(), clock);
        }
        Ok(layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_glass_migration_uses_active_material_and_preserves_widget_content(){
        let layout=Layout::load(br##"{"version":2,"widgets":{"analog":{"enabled":true,"tint":"#112233","static_wallpaper":false,"dispersion":0.4},"notes":{"x":540,"width":420,"note":"keep"}}}"##).unwrap();
        assert!(layout.sync_theme);assert_eq!(layout.glass_source,"analog");
        for s in layout.widgets.values(){assert_eq!(s.tint,"#112233");assert_eq!(s.dispersion,0.4);assert!(!s.static_wallpaper)}
        assert_eq!(layout.widgets["notes"].note,"keep");assert_eq!(layout.widgets["notes"].x,540);assert_eq!(layout.widgets["notes"].width,420);
        let saved=Layout::load(&serde_json::to_vec(&layout).unwrap()).unwrap();assert_eq!(saved.glass_source,"analog");
    }
    #[test]
    fn optical_upgrade_preserves_legacy_values_and_rejects_invalid_settings() {
        let layout=Layout::load(br##"{"version":2,"sync_theme":true,"widgets":{"notes":{"note":"Keep me","tint":"#ff8800","refraction":7,"opacity":0.3,"width":410}}}"##).unwrap();
        let notes = &layout.widgets["notes"];
        assert_eq!(notes.material, "custom");
        assert_eq!(notes.refraction, 7.0);
        assert_eq!(notes.opacity, 0.3);
        assert_eq!(notes.width, 410);
        assert_eq!(notes.note, "Keep me");
        let mut next = layout.clone();
        next.widgets.get_mut("notes").unwrap().material = "adaptive".into();
        next.widgets.get_mut("notes").unwrap().thickness = 30.0;
        next.sync_material("notes").unwrap();
        let restored = Layout::load(&serde_json::to_vec(&next).unwrap()).unwrap();
        assert_eq!(restored.widgets["clock"].thickness, 30.0);
        assert_eq!(restored.widgets["clock"].material, "adaptive");
        let mut bad = notes.clone();
        bad.dispersion = f64::NAN;
        assert!(bad.validate().is_err());
        bad = notes.clone();
        bad.quality = "invalid".into();
        assert!(bad.validate().is_err());
    }
    #[test]
    fn sync_preserves_content_geometry_and_survives_reload() {
        let mut layout = Layout::default();
        layout.widgets.get_mut("notes").unwrap().note = "Preservar".into();
        let before = layout.widgets["notes"].clone();
        layout.widgets.get_mut("clock").unwrap().tint = "#ff8800".into();
        layout.widgets.get_mut("clock").unwrap().shade = 0.6;
        layout.sync_theme = true;
        layout.sync_material("clock").unwrap();
        let restored = Layout::load(&serde_json::to_vec(&layout).unwrap()).unwrap();
        assert!(restored.sync_theme);
        for widget in restored.widgets.values() {
            assert_eq!(widget.tint, "#ff8800");
            assert_eq!(widget.shade, 0.6);
        }
        let notes = &restored.widgets["notes"];
        assert_eq!(notes.note, "Preservar");
        assert_eq!(
            (notes.x, notes.y, notes.width, notes.height, notes.enabled),
            (
                before.x,
                before.y,
                before.width,
                before.height,
                before.enabled
            )
        );
        assert!(
            Layout::load(br#"{"version":2,"widgets":{}}"#)
                .unwrap()
                .sync_theme
        );
    }
    #[test]
    fn old_preferences_gain_material_defaults() {
        let settings: Settings =
            serde_json::from_str(r##"{"x":220,"y":180,"opacity":0.3,"edit":true}"##).unwrap();
        assert_eq!(settings.x, 220);
        assert_eq!(settings.tint, "#b9d5e8");
        assert!(settings.validate().is_ok());
    }
    #[test]
    fn rejects_invalid_material_parameters() {
        let mut settings = Settings::default();
        settings.tint = "#xyz123".into();
        assert!(settings.validate().is_err());
        settings.tint = "#FF5511".into();
        settings.shine = f64::NAN;
        assert!(settings.validate().is_err());
        settings.shine = 0.5;
        settings.shade = 0.42;
        assert!(settings.validate().is_ok());
    }
    #[test]
    fn migrates_legacy_clock_without_changing_its_material() {
        let layout =
            Layout::load(br##"{"x":515,"y":230,"tint":"#ff8844","shade":0.55,"edit":true}"##)
                .unwrap();
        assert_eq!(layout.widgets.len(), 9);
        assert_eq!(layout.widgets["clock"].x, 515);
        assert_eq!(layout.widgets["clock"].tint, "#ff8844");
        assert!(layout.widgets["clock"].enabled);
        assert!(!layout.widgets["notes"].enabled);
        assert_eq!(layout.widgets["analog"].width, 240);
    }
    #[test]
    fn saves_each_widgets_notes_geometry_and_enabled_state_independently() {
        let mut layout = Layout::default();
        layout.widgets.get_mut("notes").unwrap().note = "Uma ideia\nOutra linha".into();
        layout.widgets.get_mut("system").unwrap().enabled = true;
        layout.widgets.get_mut("analog").unwrap().width = 300;
        let restored = Layout::load(&serde_json::to_vec(&layout).unwrap()).unwrap();
        assert_eq!(restored.widgets["notes"].note, "Uma ideia\nOutra linha");
        assert_eq!(restored.widgets["clock"].width, 360);
        assert_eq!(restored.widgets["analog"].width, 300);
        assert!(restored.widgets["system"].enabled);
        assert!(!restored.widgets["calendar"].enabled);
    }
    #[test]
    fn refuses_unknown_layout_version_and_invalid_widget_geometry() {
        assert!(Layout::load(br#"{"version":50,"widgets":{}}"#).is_err());
        assert!(Layout::load(br#"{"version":2,"widgets":{"unknown":{}}}"#).is_err());
        let mut s = Settings::default();
        s.width = 0;
        assert!(s.validate().is_err());
    }
    #[test]
    fn expands_old_layout_without_losing_notes_or_material(){
        let layout=Layout::load(br##"{"version":2,"glass_source":"notes","sync_theme":true,"widgets":{"notes":{"note":"Preservar","tint":"#aabbcc","enabled":true}}}"##).unwrap();
        assert_eq!(layout.widgets.len(),9);
        assert_eq!(layout.widgets["notes"].note,"Preservar");
        assert_eq!(layout.widgets["weather"].tint,"#aabbcc");
        assert!(!layout.widgets["media"].enabled);
        assert!(layout.widgets["calendar"].calendar_events.is_empty());
    }
    #[test]
    fn every_widget_window_has_native_acl(){
        let capabilities:serde_json::Value=serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        let windows=capabilities["windows"].as_array().unwrap();
        for id in IDS {assert!(windows.iter().any(|window|window.as_str()==Some(id)),"missing ACL for {id}")}
        assert!(capabilities["permissions"].as_array().unwrap().iter().any(|permission|permission.as_str()==Some("core:default")));
    }
}
