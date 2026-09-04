//! Output writing: gitignored by default, atomic write.
//!
//! Per finding 5A: transcripts are confidential by default. Writes go to
//! `./mom-test-live/` (relative to the current working directory — typically
//! the founder's project, next to their `discovery/` folder) with a
//! `.gitignore` inside that directory containing `*`, so nothing in it is ever
//! accidentally committed. The `.gitignore`-inside-the-directory idiom is used
//! deliberately instead of touching the project's own root `.gitignore`, which
//! may not exist or may have unrelated content this tool has no business
//! editing.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const OUTPUT_DIR: &str = "mom-test-live";

pub fn write_transcript(source: &Path, text: &str) -> Result<PathBuf, String> {
    let dir = PathBuf::from(OUTPUT_DIR);
    ensure_output_dir(&dir)?;

    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("transcript");
    let ts = unix_timestamp();
    let final_path = dir.join(format!("{ts}-{stem}.txt"));
    let tmp_path = dir.join(format!("{ts}-{stem}.txt.tmp"));

    fs::write(&tmp_path, text).map_err(|e| format!("couldn't write {}: {e}", tmp_path.display()))?;
    fs::rename(&tmp_path, &final_path)
        .map_err(|e| format!("couldn't finalize {}: {e}", final_path.display()))?;

    Ok(final_path)
}

fn ensure_output_dir(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("couldn't create {}: {e}", dir.display()))?;
    let gitignore = dir.join(".gitignore");
    if !gitignore.exists() {
        fs::write(&gitignore, "*\n")
            .map_err(|e| format!("couldn't write {}: {e}", gitignore.display()))?;
    }
    Ok(())
}

/// Nanosecond precision, not seconds — two `import` runs on the same source
/// file within the same second must not silently overwrite each other.
fn unix_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Tests share a process-wide cwd, so serialize them.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn in_temp_dir<T>(f: impl FnOnce() -> T) -> T {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("mom-test-live-output-test-{}", unix_timestamp()));
        fs::create_dir_all(&dir).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = f();
        std::env::set_current_dir(original).unwrap();
        result
    }

    #[test]
    fn writes_transcript_and_creates_gitignore() {
        in_temp_dir(|| {
            let source = PathBuf::from("2026-09-04-derm-office-manager.txt");
            let path = write_transcript(&source, "F: hi\nC: hello\n").unwrap();
            assert!(path.exists());
            assert_eq!(fs::read_to_string(&path).unwrap(), "F: hi\nC: hello\n");
            assert!(PathBuf::from(OUTPUT_DIR).join(".gitignore").exists());
            assert_eq!(
                fs::read_to_string(PathBuf::from(OUTPUT_DIR).join(".gitignore")).unwrap(),
                "*\n"
            );
        });
    }

    #[test]
    fn does_not_leave_a_tmp_file_behind_on_success() {
        in_temp_dir(|| {
            let source = PathBuf::from("call.txt");
            let path = write_transcript(&source, "content").unwrap();
            let tmp = path.with_extension("txt.tmp");
            assert!(!tmp.exists());
        });
    }

    #[test]
    fn second_write_does_not_duplicate_gitignore_content() {
        in_temp_dir(|| {
            let source = PathBuf::from("call.txt");
            write_transcript(&source, "first").unwrap();
            write_transcript(&source, "second").unwrap();
            let gi = fs::read_to_string(PathBuf::from(OUTPUT_DIR).join(".gitignore")).unwrap();
            assert_eq!(gi, "*\n");
        });
    }
}
