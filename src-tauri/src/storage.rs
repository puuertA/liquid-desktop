use serde::Serialize;
use sysinfo::Disks;

#[derive(Serialize)]
pub struct Drive {pub id:String,pub name:String,pub total:u64,pub available:u64}

#[tauri::command]
pub async fn storage_snapshot(window:tauri::WebviewWindow)->Result<Vec<Drive>,String>{
    if !["storage","fusion","main"].contains(&window.label()){return Err("Superficie nao autorizada".into())}
    tauri::async_runtime::spawn_blocking(||{
        Ok(Disks::new_with_refreshed_list().list().iter().filter(|d|d.total_space()>0).map(|d|Drive{
            id:d.mount_point().to_string_lossy().into_owned(),name:d.name().to_string_lossy().into_owned(),total:d.total_space(),available:d.available_space()
        }).collect())
    }).await.map_err(|e|e.to_string())?
}
#[cfg(test)]
mod tests{#[test]#[ignore="Reads actual Windows drive capacity"]fn reads_real_drives(){let drives=sysinfo::Disks::new_with_refreshed_list();assert!(drives.list().iter().any(|disk|disk.total_space()>0));for disk in drives.list(){assert!(disk.available_space()<=disk.total_space())}}}
