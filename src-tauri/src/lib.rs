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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn tmp() -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("file_renamer_test_{}", n));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn call(dir: &std::path::Path) -> Result<String, String> {
        rename_files(dir.to_string_lossy().to_string())
    }

    #[test]
    fn basic_rename_and_filename_txt() {
        let dir = tmp();
        fs::write(dir.join("b.txt"), "").unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();

        assert!(call(&dir).is_ok());
        assert!(dir.join("01.txt").exists());
        assert!(dir.join("02.txt").exists());
        assert_eq!(fs::read_to_string(dir.join("filename.txt")).unwrap(), "a.txt\nb.txt");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn preserves_extension() {
        let dir = tmp();
        fs::write(dir.join("foo.mp4"), "").unwrap();

        assert!(call(&dir).is_ok());
        assert!(dir.join("01.mp4").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn no_extension() {
        let dir = tmp();
        fs::write(dir.join("README"), "").unwrap();

        assert!(call(&dir).is_ok());
        assert!(dir.join("01").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn skips_hidden_files() {
        let dir = tmp();
        fs::write(dir.join(".hidden"), "").unwrap();
        fs::write(dir.join("visible.txt"), "").unwrap();

        assert!(call(&dir).is_ok());
        assert!(dir.join("01.txt").exists());
        assert!(!dir.join("02.txt").exists());
        assert!(dir.join(".hidden").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn skips_subdirectories() {
        let dir = tmp();
        fs::create_dir(dir.join("subdir")).unwrap();
        fs::write(dir.join("file.txt"), "").unwrap();

        assert!(call(&dir).is_ok());
        assert!(dir.join("01.txt").exists());
        assert!(dir.join("subdir").is_dir());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn skips_existing_filename_txt() {
        let dir = tmp();
        fs::write(dir.join("filename.txt"), "old").unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();

        assert!(call(&dir).is_ok());
        assert!(dir.join("01.txt").exists());
        assert_eq!(fs::read_to_string(dir.join("filename.txt")).unwrap(), "a.txt");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn three_digit_padding_for_100_plus_files() {
        let dir = tmp();
        for i in 0..100 {
            fs::write(dir.join(format!("file_{:03}.txt", i)), "").unwrap();
        }

        assert!(call(&dir).is_ok());
        assert!(dir.join("001.txt").exists());
        assert!(dir.join("100.txt").exists());
        assert!(!dir.join("01.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn error_on_non_directory() {
        let dir = tmp();
        let file = dir.join("file.txt");
        fs::write(&file, "").unwrap();

        assert!(rename_files(file.to_string_lossy().to_string()).is_err());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn error_on_empty_directory() {
        let dir = tmp();

        assert!(call(&dir).is_err());

        fs::remove_dir_all(&dir).ok();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![rename_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
