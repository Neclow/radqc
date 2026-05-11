use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Project {
    radqc: String,
    reviewer: String,
    project: String,
    image_dir: String,
    #[serde(default)]
    annotations: BTreeMap<String, Annotation>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Annotation {
    severity: String,
    reason: String,
}

#[tauri::command]
fn list_pngs(folder: String) -> Result<Vec<String>, String> {
    let root = PathBuf::from(&folder);
    if !root.is_dir() {
        return Err(format!("not a directory: {folder}"));
    }

    let mut paths: Vec<String> = WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            entry
                .path()
                .strip_prefix(&root)
                .ok()
                .and_then(|rel| rel.to_str())
                .map(String::from)
        })
        .collect();

    paths.sort();
    Ok(paths)
}

#[tauri::command]
fn read_project(path: String) -> Result<Option<Project>, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&p).map_err(|e| format!("read failed: {e}"))?;
    let project: Project =
        serde_yml::from_str(&contents).map_err(|e| format!("parse failed: {e}"))?;
    if project.radqc.is_empty() {
        return Err("not a RadQC file: missing top-level 'radqc' marker".to_string());
    }
    Ok(Some(project))
}

#[tauri::command]
fn save_project(
    path: String,
    reviewer: String,
    project: String,
    image_dir: String,
    annotations: BTreeMap<String, Annotation>,
) -> Result<(), String> {
    let p = PathBuf::from(&path);

    let project_data = Project {
        radqc: env!("CARGO_PKG_VERSION").to_string(),
        reviewer,
        project,
        image_dir,
        annotations,
    };

    let yaml = serde_yml::to_string(&project_data).map_err(|e| format!("serialize failed: {e}"))?;

    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    let tmp = p.with_extension("yaml.tmp");
    std::fs::write(&tmp, yaml).map_err(|e| format!("write failed: {e}"))?;
    std::fs::rename(&tmp, &p).map_err(|e| format!("rename failed: {e}"))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_pngs,
            read_project,
            save_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
