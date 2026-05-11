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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"").unwrap();
    }

    fn s(p: &Path) -> String {
        p.to_string_lossy().to_string()
    }

    // --- list_pngs ---

    #[test]
    fn list_pngs_empty_folder_is_empty() {
        let dir = tempdir().unwrap();
        let result = list_pngs(s(dir.path())).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_pngs_finds_pngs_at_root() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("a.png"));
        touch(&dir.path().join("b.png"));
        let result = list_pngs(s(dir.path())).unwrap();
        assert_eq!(result, vec!["a.png", "b.png"]);
    }

    #[test]
    fn list_pngs_recursive() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("a.png"));
        touch(&dir.path().join("sub/b.png"));
        touch(&dir.path().join("sub/deeper/c.png"));
        let result = list_pngs(s(dir.path())).unwrap();
        assert_eq!(result, vec!["a.png", "sub/b.png", "sub/deeper/c.png"]);
    }

    #[test]
    fn list_pngs_filters_non_png_files() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("a.png"));
        touch(&dir.path().join("b.jpg"));
        touch(&dir.path().join("c.txt"));
        touch(&dir.path().join("d.csv"));
        let result = list_pngs(s(dir.path())).unwrap();
        assert_eq!(result, vec!["a.png"]);
    }

    #[test]
    fn list_pngs_extension_is_case_insensitive() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("a.png"));
        touch(&dir.path().join("b.PNG"));
        touch(&dir.path().join("c.Png"));
        let result = list_pngs(s(dir.path())).unwrap();
        assert_eq!(result, vec!["a.png", "b.PNG", "c.Png"]);
    }

    #[test]
    fn list_pngs_errors_on_non_directory() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("not-a-dir");
        fs::write(&file, b"").unwrap();
        let result = list_pngs(s(&file));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));
    }

    // --- read_project ---

    #[test]
    fn read_project_returns_none_for_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nope.yaml");
        let result = read_project(s(&path)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn read_project_parses_valid_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        fs::write(
            &path,
            "radqc: 0.1.0\nreviewer: neil\nproject: default\nimage_dir: /tmp/imgs\nannotations:\n  a.png:\n    severity: minor\n    reason: rotation\n",
        )
        .unwrap();
        let project = read_project(s(&path)).unwrap().unwrap();
        assert_eq!(project.radqc, "0.1.0");
        assert_eq!(project.reviewer, "neil");
        assert_eq!(project.project, "default");
        assert_eq!(project.image_dir, "/tmp/imgs");
        assert_eq!(project.annotations.len(), 1);
        assert_eq!(project.annotations["a.png"].severity, "minor");
        assert_eq!(project.annotations["a.png"].reason, "rotation");
    }

    #[test]
    fn read_project_errors_on_missing_required_field() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        // No radqc field — serde rejects.
        fs::write(
            &path,
            "reviewer: neil\nproject: default\nimage_dir: /tmp\nannotations: {}\n",
        )
        .unwrap();
        let result = read_project(s(&path));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("parse failed"));
    }

    #[test]
    fn read_project_errors_on_empty_radqc_marker() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        fs::write(
            &path,
            "radqc: \"\"\nreviewer: neil\nproject: default\nimage_dir: /tmp\nannotations: {}\n",
        )
        .unwrap();
        let result = read_project(s(&path));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a RadQC file"));
    }

    #[test]
    fn read_project_errors_on_malformed_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        fs::write(&path, "foo: [unclosed\n").unwrap();
        let result = read_project(s(&path));
        assert!(result.is_err());
    }

    // --- save_project ---

    #[test]
    fn save_project_writes_file_with_marker() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "a.png".to_string(),
            Annotation {
                severity: "minor".to_string(),
                reason: "rotation".to_string(),
            },
        );
        save_project(
            s(&path),
            "neil".to_string(),
            "default".to_string(),
            "/tmp/imgs".to_string(),
            annotations,
        )
        .unwrap();
        assert!(path.exists());
        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("radqc:"));
        assert!(contents.contains("neil"));
        assert!(contents.contains("a.png"));
    }

    #[test]
    fn save_project_creates_parent_directories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a/b/c/p.yaml");
        save_project(
            s(&nested),
            "neil".to_string(),
            "default".to_string(),
            "/tmp".to_string(),
            BTreeMap::new(),
        )
        .unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn save_project_leaves_no_temp_file_on_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        save_project(
            s(&path),
            "neil".to_string(),
            "default".to_string(),
            "/tmp".to_string(),
            BTreeMap::new(),
        )
        .unwrap();
        let tmp = path.with_extension("yaml.tmp");
        assert!(!tmp.exists());
    }

    // --- round-trip ---

    #[test]
    fn save_then_read_round_trips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("p.yaml");
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "a.png".to_string(),
            Annotation {
                severity: "minor".to_string(),
                reason: "rotation".to_string(),
            },
        );
        annotations.insert(
            "sub/b.png".to_string(),
            Annotation {
                severity: "major".to_string(),
                reason: "motion blur, severe".to_string(),
            },
        );
        save_project(
            s(&path),
            "neil".to_string(),
            "default".to_string(),
            "/tmp/imgs".to_string(),
            annotations,
        )
        .unwrap();

        let loaded = read_project(s(&path)).unwrap().unwrap();
        assert_eq!(loaded.reviewer, "neil");
        assert_eq!(loaded.project, "default");
        assert_eq!(loaded.image_dir, "/tmp/imgs");
        assert_eq!(loaded.annotations.len(), 2);
        assert_eq!(loaded.annotations["a.png"].severity, "minor");
        assert_eq!(loaded.annotations["a.png"].reason, "rotation");
        assert_eq!(loaded.annotations["sub/b.png"].severity, "major");
        assert_eq!(loaded.annotations["sub/b.png"].reason, "motion blur, severe");
    }
}
