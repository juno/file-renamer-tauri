use std::fs;
use std::path::Path;

#[tauri::command]
fn rename_files(folder_path: String) -> Result<String, String> {
    let dir = Path::new(&folder_path);

    if !dir.is_dir() {
        return Err("ドロップされたパスはフォルダではありません".to_string());
    }

    let mut files: Vec<String> = fs::read_dir(dir)
        .map_err(|e| format!("フォルダを読み込めませんでした: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            // skip hidden files, subdirectories, and filename.txt
            if name.starts_with('.') || entry.path().is_dir() || name == "filename.txt" {
                return None;
            }
            Some(name)
        })
        .collect();

    if files.is_empty() {
        return Err("フォルダにファイルが見つかりませんでした".to_string());
    }

    files.sort();

    let total = files.len();
    let digits = if total >= 100 { 3 } else { 2 };

    // Phase 1: rename to temp names to avoid conflicts with target names
    let mut temp_names: Vec<(String, String)> = Vec::new();
    for (i, original) in files.iter().enumerate() {
        let temp = format!(".renamer_tmp_{}", i);
        fs::rename(dir.join(original), dir.join(&temp))
            .map_err(|e| format!("一時リネームに失敗しました ({}): {}", original, e))?;
        temp_names.push((original.clone(), temp));
    }

    // Phase 2: rename from temp to final sequential names
    let mut original_names_in_order: Vec<String> = Vec::new();
    for (i, (original, temp)) in temp_names.iter().enumerate() {
        let ext = Path::new(original)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let new_name = format!("{:0>width$}{}", i + 1, ext, width = digits);
        fs::rename(dir.join(temp), dir.join(&new_name))
            .map_err(|e| format!("リネームに失敗しました ({}): {}", original, e))?;
        original_names_in_order.push(original.clone());
    }

    // Write filename.txt
    let content = original_names_in_order.join("\n");
    fs::write(dir.join("filename.txt"), content)
        .map_err(|e| format!("filename.txt の書き込みに失敗しました: {}", e))?;

    Ok(format!("{} 件のファイルをリネームしました", total))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![rename_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
