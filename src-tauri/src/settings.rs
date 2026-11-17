use serde::{Deserialize, Serialize};
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
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
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
        Ok(())
    }
}

pub const IDS: [&str; 5] = ["clock", "analog", "calendar", "notes", "system"];
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
    #[serde(default)]
    pub sync_theme: bool,
    pub widgets: std::collections::BTreeMap<String, Settings>,
}
impl Default for Layout {
    fn default() -> Self {
        let mut widgets = std::collections::BTreeMap::new();
        for (i, id) in IDS.iter().enumerate() {
            let mut s = Settings::default();
            s.x = 100 + (i as i32 % 2) * 400;
            s.y = 100 + (i as i32 / 2) * 290;
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
                _ => {}
            }
            widgets.insert(id.to_string(), s);
        }
        Self {
            version: 2,
            sync_theme: false,
            widgets,
        }
    }
}
impl Layout {
    pub fn sync_material(&mut self, source: &str) -> Result<(), String> {
        valid_id(source)?;
        let material = self.widgets[source].clone();
        for widget in self.widgets.values_mut() {
            widget.tint = material.tint.clone();
            widget.opacity = material.opacity;
            widget.shine = material.shine;
            widget.shade = material.shade;
            widget.refraction = material.refraction;
            widget.static_wallpaper = material.static_wallpaper;
        }
        Ok(())
    }
    pub fn load(bytes: &[u8]) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut layout = Self::default();
        if value.get("widgets").is_some() {
            let saved: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
            if saved.version != 2 {
                return Err("Versão de layout não suportada".into());
            }
            layout.sync_theme = saved.sync_theme;
            for (id, settings) in saved.widgets {
                valid_id(&id)?;
                settings.validate()?;
                layout.widgets.insert(id, settings);
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
            !Layout::load(br#"{"version":2,"widgets":{}}"#)
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
        assert_eq!(layout.widgets.len(), 5);
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
}
