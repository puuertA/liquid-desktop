use base64::{engine::general_purpose::STANDARD,Engine};
use std::{fs,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};
use tauri::Manager;

fn allowed(w:&tauri::WebviewWindow)->Result<(),String>{if ["gallery","fusion"].contains(&w.label()){Ok(())}else{Err("Use a galeria".into())}}
fn directory(app:&tauri::AppHandle)->Result<PathBuf,String>{let path=app.path().app_data_dir().map_err(|e|e.to_string())?.join("gallery");fs::create_dir_all(&path).map_err(|e|e.to_string())?;Ok(path)}
fn valid_name(name:&str)->bool{name.len()<100&&name.len()>4&&name.chars().all(|c|c.is_ascii_alphanumeric()||c=='.'||c=='-')&&["png","jpg","webp","gif"].iter().any(|ext|name.ends_with(&format!(".{ext}")))}
fn image_type(data:&[u8])->Option<&'static str>{if data.starts_with(&[0x89,0x50,0x4e,0x47]){Some("png")}else if data.starts_with(&[0xff,0xd8,0xff]){Some("jpg")}else if data.starts_with(b"RIFF")&&data.get(8..12)==Some(b"WEBP"){Some("webp")}else if data.starts_with(b"GIF8"){Some("gif")}else{None}}
#[tauri::command]
pub fn gallery_list(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Vec<String>,String>{allowed(&window)?;let mut names=fs::read_dir(directory(&app)?).map_err(|e|e.to_string())?.filter_map(|e|e.ok()).filter_map(|e|e.file_name().to_str().map(str::to_owned)).filter(|s|valid_name(s)).collect::<Vec<_>>();names.sort();Ok(names)}
#[tauri::command]
pub fn gallery_import(app:tauri::AppHandle,window:tauri::WebviewWindow,data:String)->Result<String,String>{allowed(&window)?;if data.len()>34_000_000{return Err("Imagem acima de 25 MB".into())}let bytes=STANDARD.decode(data).map_err(|_|"Imagem invalida")?;if bytes.len()>25_000_000{return Err("Imagem acima de 25 MB".into())}let ext=image_type(&bytes).ok_or("Formato de imagem nao suportado")?;let folder=directory(&app)?;if fs::read_dir(&folder).map_err(|e|e.to_string())?.count()>=120{return Err("Limite de 120 imagens".into())}let stamp=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos();let name=format!("{stamp}.{ext}");fs::write(folder.join(&name),bytes).map_err(|e|e.to_string())?;Ok(name)}
#[tauri::command]
pub fn gallery_image(app:tauri::AppHandle,window:tauri::WebviewWindow,name:String)->Result<String,String>{allowed(&window)?;if !valid_name(&name){return Err("Imagem invalida".into())}let data=fs::read(directory(&app)?.join(&name)).map_err(|e|e.to_string())?;if data.len()>25_000_000{return Err("Imagem grande demais".into())}let ext=image_type(&data).ok_or("Imagem invalida")?;Ok(format!("data:image/{ext};base64,{}",STANDARD.encode(data)))}
#[tauri::command]
pub fn gallery_remove(app:tauri::AppHandle,window:tauri::WebviewWindow,name:String)->Result<(),String>{allowed(&window)?;if !valid_name(&name){return Err("Imagem invalida".into())}fs::remove_file(directory(&app)?.join(name)).map_err(|e|e.to_string())}
#[cfg(test)]
mod tests{use super::*;#[test]fn rejects_non_images_and_unsafe_names(){assert_eq!(image_type(b"not an image"),None);assert_eq!(image_type(&[0x89,0x50,0x4e,0x47]),Some("png"));assert!(!valid_name("../secret.jpg"));assert!(!valid_name("data.svg"));assert!(valid_name("12345.jpg"));}}
