use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use windows::{
    Media::Control::*,
    Storage::Streams::*,
    Win32::{
        Media::Audio::{Endpoints::*, *},
        System::Com::*,
        System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
    },
};

pub struct Apartment;
impl Apartment {
    pub fn new() -> Result<Self, String> {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED).map_err(|e| e.to_string())?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { RoUninitialize() }
    }
}
#[derive(Clone, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub app: String,
}
#[derive(Clone, Serialize)]
pub struct Track {
    pub id: String,
    pub app: String,
    pub title: String,
    pub artist: String,
    pub artwork: Option<String>,
    pub playing: bool,
    pub position: f64,
    pub start: f64,
    pub end: f64,
    pub toggle: bool,
    pub previous: bool,
    pub next: bool,
    pub seek: bool,
}
#[derive(Clone, Serialize)]
pub struct MediaSnapshot {
    pub sessions: Vec<SessionInfo>,
    pub track: Option<Track>,
}
struct Session {
    id: String,
    app: String,
    session: GlobalSystemMediaTransportControlsSession,
    art_key: String,
    art: Option<String>,
}
#[derive(Default)]
pub struct Provider {
    manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
    sessions: Vec<Session>,
    next_id: u64,
}
impl Provider {
    pub fn snapshot(&mut self, selected: Option<&str>) -> Result<MediaSnapshot, String> {
        let result = self.read(selected).map_err(|e| e.to_string());
        if result.is_err() {
            self.manager = None;
            self.sessions.clear();
        }
        result
    }
    fn read(&mut self, selected: Option<&str>) -> windows::core::Result<MediaSnapshot> {
        if self.manager.is_none() {
            self.manager =
                Some(GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?);
        }
        let manager = self.manager.as_ref().unwrap();
        let current = manager.GetCurrentSession().ok();
        let list = manager.GetSessions()?;
        let mut updated = Vec::new();
        for i in 0..list.Size()?.min(16) {
            let session = list.GetAt(i)?;
            if let Some(index) = self.sessions.iter().position(|old| old.session == session) {
                updated.push(self.sessions.remove(index));
            } else {
                self.next_id += 1;
                updated.push(Session {
                    id: format!("session-{}", self.next_id),
                    app: session.SourceAppUserModelId()?.to_string(),
                    session,
                    art_key: String::new(),
                    art: None,
                });
            }
        }
        self.sessions = updated;
        let sessions = self
            .sessions
            .iter()
            .map(|s| SessionInfo {
                id: s.id.clone(),
                app: s.app.clone(),
            })
            .collect();
        let index = selected
            .and_then(|id| self.sessions.iter().position(|s| s.id == id))
            .or_else(|| {
                current.and_then(|current| self.sessions.iter().position(|s| s.session == current))
            })
            .or_else(|| {
                if self.sessions.is_empty() {
                    None
                } else {
                    Some(0)
                }
            });
        let Some(index) = index else {
            return Ok(MediaSnapshot {
                sessions,
                track: None,
            });
        };
        let entry = &mut self.sessions[index];
        let session = &entry.session;
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;
        let title = properties.Title()?.to_string();
        let artist = properties.Artist()?.to_string();
        let key = format!(
            "{title}\0{artist}\0{}",
            properties.AlbumTitle().unwrap_or_default()
        );
        if key != entry.art_key {
            entry.art_key = key;
            entry.art = artwork(&properties).ok();
        }
        let info = session.GetPlaybackInfo()?;
        let controls = info.Controls()?;
        let timeline = session.GetTimelineProperties()?;
        let start = timeline.StartTime()?.Duration as f64 / 10_000_000.0;
        let end = timeline.EndTime()?.Duration as f64 / 10_000_000.0;
        let mut position = timeline.Position()?.Duration as f64 / 10_000_000.0;
        let playing = info.PlaybackStatus()?
            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
        // Advance from the provider's real timestamp/rate, never from a made-up duration.
        if playing {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
                + 11_644_473_600.0;
            let updated = timeline.LastUpdatedTime()?.UniversalTime as f64 / 10_000_000.0;
            let rate = info
                .PlaybackRate()
                .ok()
                .and_then(|r| r.Value().ok())
                .unwrap_or(1.0);
            if updated > 0.0 && updated <= now && rate.is_finite() {
                position += (now - updated) * rate;
            }
        }
        Ok(MediaSnapshot {
            sessions,
            track: Some(Track {
                id: entry.id.clone(),
                app: entry.app.clone(),
                title,
                artist,
                artwork: entry.art.clone(),
                playing,
                start,
                end,
                position: position.max(start).min(end.max(start)),
                toggle: controls.IsPlayPauseToggleEnabled()?,
                previous: controls.IsPreviousEnabled()?,
                next: controls.IsNextEnabled()?,
                seek: controls.IsPlaybackPositionEnabled()? && end > start && start >= 0.0,
            }),
        })
    }
    pub fn action(&self, id: &str, action: &str, position: Option<f64>) -> Result<(), String> {
        let session = &self
            .sessions
            .iter()
            .find(|s| s.id == id)
            .ok_or("Sessão indisponível; atualize a ilha")?
            .session;
        if action == "seek" {
            let timeline = session.GetTimelineProperties().map_err(|e| e.to_string())?;
            let start =
                timeline.StartTime().map_err(|e| e.to_string())?.Duration as f64 / 10_000_000.0;
            let end = timeline.EndTime().map_err(|e| e.to_string())?.Duration as f64 / 10_000_000.0;
            if !position.is_some_and(|p| p.is_finite() && p >= start && p <= end) {
                return Err("Posição fora da faixa da sessão".into());
            }
        }
        let result = (|| -> windows::core::Result<bool> {
            Ok(match action {
                "toggle" => session.TryTogglePlayPauseAsync()?.get()?,
                "previous" => session.TrySkipPreviousAsync()?.get()?,
                "next" => session.TrySkipNextAsync()?.get()?,
                "seek" => session
                    .TryChangePlaybackPositionAsync(
                        (position.unwrap_or(0.0) * 10_000_000.0) as i64,
                    )?
                    .get()?,
                _ => false,
            })
        })()
        .map_err(|e| e.to_string())?;
        if result {
            Ok(())
        } else {
            Err("O aplicativo não aceitou esse comando de mídia".into())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Reads Windows media sessions and default audio endpoint without changing playback or volume"]
    fn reads_live_media_and_audio() {
        let _apartment = Apartment::new().unwrap();
        let mut provider = Provider::default();
        let snapshot = provider.snapshot(None).expect("GSMTC read");
        eprintln!(
            "GSMTC accessible; sessions={}, track={}",
            snapshot.sessions.len(),
            snapshot.track.is_some()
        );
        let volume = volume(None, None).expect("Core Audio read");
        assert!((0.0..=1.0).contains(&volume.level));
        let peak=peak().expect("Core Audio peak");
        assert!((0.0..=1.0).contains(&peak));
        assert!(provider.action("invalid-session", "toggle", None).is_err());
    }
}
fn artwork(
    properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> windows::core::Result<String> {
    let stream = properties.Thumbnail()?.OpenReadAsync()?.get()?;
    let size = stream.Size()?;
    if size == 0 || size > 2 * 1024 * 1024 {
        return Err(windows::core::Error::from_win32());
    }
    let mime = stream.ContentType()?.to_string();
    if !["image/png", "image/jpeg", "image/webp", "image/bmp"].contains(&mime.as_str()) {
        return Err(windows::core::Error::from_win32());
    }
    let buffer = Buffer::Create(size as u32)?;
    let buffer = stream
        .ReadAsync(&buffer, size as u32, InputStreamOptions::None)?
        .get()?;
    let reader = DataReader::FromBuffer(&buffer)?;
    let mut bytes = vec![0; buffer.Length()? as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}
#[derive(Clone, Serialize)]
pub struct Volume {
    pub level: f32,
    pub muted: bool,
}
fn endpoint() -> windows::core::Result<IAudioEndpointVolume> {
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)?
            .Activate(CLSCTX_ALL, None)
    }
}
pub fn peak()->Result<f32,String>{unsafe{
    let enumerator:IMMDeviceEnumerator=CoCreateInstance(&MMDeviceEnumerator,None,CLSCTX_ALL).map_err(|e|e.to_string())?;
    let device=enumerator.GetDefaultAudioEndpoint(eRender,eMultimedia).map_err(|e|e.to_string())?;
    let meter:IAudioMeterInformation=device.Activate(CLSCTX_ALL,None).map_err(|e|e.to_string())?;
    meter.GetPeakValue().map(|value|value.clamp(0.0,1.0)).map_err(|e|e.to_string())
}}
pub fn volume(level: Option<f32>, muted: Option<bool>) -> Result<Volume, String> {
    unsafe {
        let endpoint = endpoint().map_err(|e| e.to_string())?;
        if let Some(level) = level {
            if !level.is_finite() || !(0.0..=1.0).contains(&level) {
                return Err("Volume inválido".into());
            }
            endpoint
                .SetMasterVolumeLevelScalar(level, std::ptr::null())
                .map_err(|e| e.to_string())?;
        }
        if let Some(muted) = muted {
            endpoint
                .SetMute(muted, std::ptr::null())
                .map_err(|e| e.to_string())?;
        }
        Ok(Volume {
            level: endpoint
                .GetMasterVolumeLevelScalar()
                .map_err(|e| e.to_string())?,
            muted: endpoint.GetMute().map_err(|e| e.to_string())?.as_bool(),
        })
    }
}
