use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[tauri::command]
fn rename_files(folder_path: String) -> Result<String, String> {
    let dir = Path::new(&folder_path);

    if !dir.is_dir() {
        return Err("ドロップされたパスはフォルダではありません".to_string());
    }

    let files_in_folder: Vec<String> = fs::read_dir(dir)
        .map_err(|e| format!("フォルダを読み込めませんでした: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || entry.path().is_dir() || name == "filename.txt" {
                return None;
            }
            Some(name)
        })
        .collect();

    if files_in_folder.is_empty() {
        return Err("フォルダにファイルが見つかりませんでした".to_string());
    }

    // entries: (original_name, current_name_in_folder)
    let entries: Vec<(String, String)> = build_entries(&dir, &files_in_folder);

    let total = entries.len();
    let digits = if total >= 100 { 3 } else { 2 };

    // Phase 1: rename current names to temp names to avoid conflicts
    let mut temp_entries: Vec<(String, String)> = Vec::new();
    for (i, (orig, current)) in entries.iter().enumerate() {
        let temp = format!(".renamer_tmp_{}", i);
        fs::rename(dir.join(current), dir.join(&temp))
            .map_err(|e| format!("一時リネームに失敗しました ({}): {}", current, e))?;
        temp_entries.push((orig.clone(), temp));
    }

    // Phase 2: rename from temp to final sequential names
    let mut original_names_in_order: Vec<String> = Vec::new();
    for (i, (orig, temp)) in temp_entries.iter().enumerate() {
        let new_name = seq_name(i, orig, digits);
        fs::rename(dir.join(temp), dir.join(&new_name))
            .map_err(|e| format!("リネームに失敗しました ({}): {}", orig, e))?;
        original_names_in_order.push(orig.clone());
    }

    // Write filename.txt
    let content = original_names_in_order.join("\n");
    fs::write(dir.join("filename.txt"), content)
        .map_err(|e| format!("filename.txt の書き込みに失敗しました: {}", e))?;

    Ok(format!("{} 件のファイルをリネームしました", total))
}

#[tauri::command]
fn restore_files(folder_path: String) -> Result<String, String> {
    let dir = Path::new(&folder_path);

    if !dir.is_dir() {
        return Err("ドロップされたパスはフォルダではありません".to_string());
    }

    let list = dir.join("filename.txt");
    let content = fs::read_to_string(&list)
        .map_err(|e| format!("filename.txt を読み込めませんでした: {}", e))?;
    let originals: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    if originals.is_empty() {
        return Err("filename.txt が空です".to_string());
    }

    let digits = if originals.len() >= 100 { 3 } else { 2 };
    let seq_names: Vec<String> = originals
        .iter()
        .enumerate()
        .map(|(i, orig)| seq_name(i, orig, digits))
        .collect();

    // 何も変更する前に全件チェック
    let mut seen = HashSet::new();
    for orig in &originals {
        if !seen.insert(*orig) {
            return Err(format!("filename.txt に同じ名前が重複しています: {}", orig));
        }
        if orig.starts_with(".renamer_tmp_") {
            return Err(format!("不正なファイル名です: {}", orig));
        }
        // filename.txt は外部入力扱い: 単一の通常ファイル名のみ許可（パストラバーサル対策）
        if Path::new(orig).file_name() != Some(std::ffi::OsStr::new(orig)) {
            return Err(format!("不正なファイル名です: {}", orig));
        }
        // 連番ファイル以外の既存ファイルは上書きしない
        if dir.join(orig).exists() && !seq_names.iter().any(|s| s == orig) {
            return Err(format!("同名のファイルが既に存在します: {}", orig));
        }
    }
    for seq in &seq_names {
        if !dir.join(seq).is_file() {
            return Err(format!("連番ファイルが見つかりませんでした: {}", seq));
        }
    }

    // Phase 1: 一時名へ（連番名と元の名前の衝突防止）
    for (i, seq) in seq_names.iter().enumerate() {
        fs::rename(dir.join(seq), dir.join(format!(".renamer_tmp_{}", i)))
            .map_err(|e| format!("一時リネームに失敗しました ({}): {}", seq, e))?;
    }

    // Phase 2: 元の名前へ
    for (i, orig) in originals.iter().enumerate() {
        fs::rename(dir.join(format!(".renamer_tmp_{}", i)), dir.join(orig))
            .map_err(|e| format!("リネームに失敗しました ({}): {}", orig, e))?;
    }

    fs::remove_file(&list)
        .map_err(|e| format!("filename.txt の削除に失敗しました: {}", e))?;

    Ok(format!("{} 件のファイルを元に戻しました", originals.len()))
}

/// i 番目 (0 始まり) の連番名。拡張子は元のファイル名から引き継ぐ。
fn seq_name(i: usize, orig: &str, digits: usize) -> String {
    let ext = Path::new(orig)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    format!("{:0>width$}{}", i + 1, ext, width = digits)
}

fn build_entries(dir: &Path, files_in_folder: &[String]) -> Vec<(String, String)> {
    let existing_originals: Vec<String> = fs::read_to_string(dir.join("filename.txt"))
        .map(|content| {
            content
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default();

    if existing_originals.is_empty() {
        let mut files = files_in_folder.to_vec();
        files.sort();
        return files.into_iter().map(|f| (f.clone(), f)).collect();
    }

    let files_set: HashSet<&String> = files_in_folder.iter().collect();
    let old_digits = if existing_originals.len() >= 100 { 3 } else { 2 };

    let mut matched: Vec<(String, String)> = Vec::new();
    let mut known_sequential: HashSet<String> = HashSet::new();

    for (i, orig) in existing_originals.iter().enumerate() {
        let seq = seq_name(i, orig, old_digits);
        if files_set.contains(&seq) {
            known_sequential.insert(seq.clone());
            matched.push((orig.clone(), seq));
        }
    }

    let mut new_files: Vec<String> = files_in_folder
        .iter()
        .filter(|f| !known_sequential.contains(*f))
        .cloned()
        .collect();
    new_files.sort();

    matched.extend(new_files.into_iter().map(|f| (f.clone(), f)));
    matched
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
        fs::remove_dir_all(&dir).ok();
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

    #[test]
    fn reprocess_appends_new_file_with_original_names() {
        let dir = tmp();
        fs::write(dir.join("a.jpg"), "").unwrap();
        fs::write(dir.join("b.jpg"), "").unwrap();
        call(&dir).unwrap();

        // Add a new file after initial processing
        fs::write(dir.join("c.jpg"), "").unwrap();
        call(&dir).unwrap();

        assert!(dir.join("01.jpg").exists());
        assert!(dir.join("02.jpg").exists());
        assert!(dir.join("03.jpg").exists());
        // filename.txt must contain original names, not sequential names
        assert_eq!(
            fs::read_to_string(dir.join("filename.txt")).unwrap(),
            "a.jpg\nb.jpg\nc.jpg"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reprocess_preserves_existing_order_appends_new() {
        let dir = tmp();
        fs::write(dir.join("z.txt"), "").unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();
        call(&dir).unwrap();
        // After first run: 01.txt=a.txt, 02.txt=z.txt

        fs::write(dir.join("m.txt"), "").unwrap();
        call(&dir).unwrap();

        assert_eq!(
            fs::read_to_string(dir.join("filename.txt")).unwrap(),
            "a.txt\nz.txt\nm.txt"
        );

        fs::remove_dir_all(&dir).ok();
    }

    fn no_tmp_left(dir: &std::path::Path) -> bool {
        fs::read_dir(dir)
            .unwrap()
            .all(|e| !e.unwrap().file_name().to_string_lossy().starts_with(".renamer_tmp_"))
    }

    #[test]
    fn rename_rolls_back_when_phase2_fails() {
        let dir = tmp();
        fs::write(dir.join("a.txt"), "A").unwrap();
        fs::write(dir.join("b.txt"), "B").unwrap();
        // サブディレクトリ "02.txt" があると b.txt → 02.txt の rename が失敗する
        fs::create_dir(dir.join("02.txt")).unwrap();

        assert!(call(&dir).is_err());
        assert_eq!(fs::read_to_string(dir.join("a.txt")).unwrap(), "A");
        assert_eq!(fs::read_to_string(dir.join("b.txt")).unwrap(), "B");
        assert!(!dir.join("01.txt").exists());
        assert!(!dir.join("filename.txt").exists());
        assert!(no_tmp_left(&dir));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_rolls_back_when_phase2_fails() {
        let dir = tmp();
        fs::write(dir.join("a.txt"), "A").unwrap();
        fs::write(dir.join("b.txt"), "B").unwrap();
        call(&dir).unwrap(); // 01.txt=a.txt, 02.txt=b.txt
        // 事前チェックはすり抜けるが rename は失敗する名前 (NAME_MAX 超過) に差し替える
        fs::write(dir.join("filename.txt"), format!("a.txt\n{}.txt", "x".repeat(300))).unwrap();

        assert!(restore(&dir).is_err());
        assert_eq!(fs::read_to_string(dir.join("01.txt")).unwrap(), "A");
        assert_eq!(fs::read_to_string(dir.join("02.txt")).unwrap(), "B");
        assert!(!dir.join("a.txt").exists());
        assert!(dir.join("filename.txt").exists());
        assert!(no_tmp_left(&dir));

        fs::remove_dir_all(&dir).ok();
    }

    fn restore(dir: &std::path::Path) -> Result<String, String> {
        restore_files(dir.to_string_lossy().to_string())
    }

    #[test]
    fn restore_reverts_names_and_removes_filename_txt() {
        let dir = tmp();
        fs::write(dir.join("b.txt"), "B").unwrap();
        fs::write(dir.join("a.jpg"), "A").unwrap();
        call(&dir).unwrap();

        assert!(restore(&dir).is_ok());
        assert_eq!(fs::read_to_string(dir.join("a.jpg")).unwrap(), "A");
        assert_eq!(fs::read_to_string(dir.join("b.txt")).unwrap(), "B");
        assert!(!dir.join("01.jpg").exists());
        assert!(!dir.join("02.txt").exists());
        assert!(!dir.join("filename.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_handles_original_names_that_collide_with_sequential() {
        let dir = tmp();
        // 元の名前が連番形式で、入れ替わるケース
        fs::write(dir.join("02.txt"), "x").unwrap();
        fs::write(dir.join("03.txt"), "y").unwrap();
        call(&dir).unwrap(); // 01.txt=02.txt, 02.txt=03.txt

        assert!(restore(&dir).is_ok());
        assert_eq!(fs::read_to_string(dir.join("02.txt")).unwrap(), "x");
        assert_eq!(fs::read_to_string(dir.join("03.txt")).unwrap(), "y");
        assert!(!dir.join("01.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_three_digit() {
        let dir = tmp();
        for i in 0..100 {
            fs::write(dir.join(format!("file_{:03}.txt", i)), "").unwrap();
        }
        call(&dir).unwrap();

        assert!(restore(&dir).is_ok());
        assert!(dir.join("file_000.txt").exists());
        assert!(dir.join("file_099.txt").exists());
        assert!(!dir.join("001.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_errors_without_filename_txt() {
        let dir = tmp();
        fs::write(dir.join("01.txt"), "").unwrap();

        assert!(restore(&dir).is_err());
        assert!(dir.join("01.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_errors_and_changes_nothing_when_sequential_file_missing() {
        let dir = tmp();
        fs::write(dir.join("a.txt"), "").unwrap();
        fs::write(dir.join("b.txt"), "").unwrap();
        call(&dir).unwrap();
        fs::remove_file(dir.join("02.txt")).unwrap();

        assert!(restore(&dir).is_err());
        assert!(dir.join("01.txt").exists());
        assert!(dir.join("filename.txt").exists());
        assert!(!dir.join("a.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_rejects_path_traversal_and_changes_nothing() {
        let dir = tmp();
        let sub = dir.join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("01.txt"), "").unwrap();
        fs::write(sub.join("filename.txt"), "../escaped.txt").unwrap();

        assert!(restore(&sub).is_err());
        assert!(!dir.join("escaped.txt").exists());
        assert!(sub.join("01.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_rejects_duplicate_names_in_filename_txt() {
        let dir = tmp();
        fs::write(dir.join("a.txt"), "1").unwrap();
        fs::write(dir.join("b.txt"), "2").unwrap();
        call(&dir).unwrap();
        fs::write(dir.join("a.txt"), "3").unwrap();
        call(&dir).unwrap(); // filename.txt = a.txt, b.txt, a.txt

        assert!(restore(&dir).is_err());
        assert!(dir.join("01.txt").exists());
        assert!(dir.join("03.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_refuses_to_overwrite_existing_file() {
        let dir = tmp();
        fs::write(dir.join("a.txt"), "A").unwrap();
        call(&dir).unwrap();
        fs::write(dir.join("a.txt"), "other").unwrap(); // 元の名前が別ファイルで使われている

        assert!(restore(&dir).is_err());
        assert_eq!(fs::read_to_string(dir.join("a.txt")).unwrap(), "other");
        assert!(dir.join("01.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_errors_on_non_directory() {
        let dir = tmp();
        let file = dir.join("file.txt");
        fs::write(&file, "").unwrap();

        assert!(restore_files(file.to_string_lossy().to_string()).is_err());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reprocess_handles_digit_expansion() {
        let dir = tmp();
        // Create 99 files so first run uses 2-digit padding
        for i in 0..99 {
            fs::write(dir.join(format!("file_{:03}.txt", i)), "").unwrap();
        }
        call(&dir).unwrap();
        assert!(dir.join("01.txt").exists());
        assert!(dir.join("99.txt").exists());

        // Add one more file to push total to 100
        fs::write(dir.join("zzz_new.txt"), "").unwrap();
        call(&dir).unwrap();

        // Should now use 3-digit padding
        assert!(!dir.join("01.txt").exists());
        assert!(dir.join("001.txt").exists());
        assert!(dir.join("100.txt").exists());

        fs::remove_dir_all(&dir).ok();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![rename_files, restore_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
