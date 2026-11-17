use windows_registry::CURRENT_USER;
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "Liquid Desktop";
pub fn enabled() -> Result<bool, String> {
    match CURRENT_USER.open(RUN).and_then(|key| key.get_string(NAME)) {
        Ok(_) => Ok(true),
        Err(e) if e.code().0 as u32 == 0x80070002 => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}
pub fn set(enabled: bool) -> Result<bool, String> {
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let command = format!("\"{}\" --background", exe.display());
        CURRENT_USER
            .create(RUN)
            .and_then(|key| key.set_string(NAME, &command))
            .map_err(|e| e.to_string())?;
    } else {
        match CURRENT_USER
            .options()
            .write()
            .open(RUN)
            .and_then(|key| key.remove_value(NAME))
        {
            Ok(_) => {}
            Err(e) if e.code().0 as u32 == 0x80070002 => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    self::enabled()
}
