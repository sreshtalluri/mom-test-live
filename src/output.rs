//! Private, gitignored output, published atomically without overwriting prior imports.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn write_transcript(source: &Path, text: &str) -> Result<PathBuf, String> {
    write_in(Path::new("mom-test-live"), source, text)
}

fn write_in(dir: &Path, source: &Path, text: &str) -> Result<PathBuf, String> {
    if text.trim().is_empty() {
        return Err("refusing to save an empty transcript".into());
    }
    ensure_output_dir(dir)?;
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("transcript");
    // Limit filename length, and neutralize control characters in terminal output.
    let stem: String = stem
        .chars()
        .take(80)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut temporary = tempfile::Builder::new()
        .prefix(".import-")
        .tempfile_in(dir)
        .map_err(|e| format!("couldn't create transcript in {}: {e}", dir.display()))?;
    temporary
        .write_all(text.as_bytes())
        .map_err(|e| format!("couldn't write transcript: {e}"))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|e| format!("couldn't flush transcript: {e}"))?;
    let nonce = temporary
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .replace(".import-", "");
    let destination = dir.join(format!("{stem}-{nonce}.txt"));
    temporary
        .persist_noclobber(&destination)
        .map_err(|e| format!("couldn't finalize {}: {}", destination.display(), e.error))?;
    fs::canonicalize(&destination).map_err(|e| format!("couldn't locate saved transcript: {e}"))
}

fn ensure_output_dir(dir: &Path) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(dir) {
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(format!(
                "{} must be a real directory, not a link or file",
                dir.display()
            ));
        }
    }
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(dir)
        .map_err(|e| format!("couldn't create {}: {e}", dir.display()))?;
    let gitignore = dir.join(".gitignore");
    if fs::symlink_metadata(&gitignore).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(format!("{} must not be a symlink", gitignore.display()));
    }
    // Always enforce privacy, even if an earlier .gitignore was edited.
    let mut ignore = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    ignore
        .write_all(b"*\n")
        .map_err(|e| format!("couldn't write output .gitignore: {e}"))?;
    ignore.as_file().sync_all().map_err(|e| e.to_string())?;
    ignore
        .persist(&gitignore)
        .map_err(|e| format!("couldn't protect transcript output: {}", e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_private_ignored_output_without_overwriting() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("mom-test-live");
        let first = write_in(&dir, Path::new("call.txt"), "F: hello\nC: hi\n").unwrap();
        let second = write_in(&dir, Path::new("call.txt"), "different").unwrap();
        assert_ne!(first, second);
        assert!(first.is_absolute());
        assert_eq!(fs::read_to_string(&first).unwrap(), "F: hello\nC: hi\n");
        assert_eq!(fs::read_to_string(dir.join(".gitignore")).unwrap(), "*\n");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(first).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(dir).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }

    #[test]
    fn repairs_existing_ignore_and_rejects_empty_output() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join(".gitignore"), "*.tmp\n!*.txt\n").unwrap();
        write_in(root.path(), Path::new("call.txt"), "text").unwrap();
        assert_eq!(
            fs::read_to_string(root.path().join(".gitignore")).unwrap(),
            "*\n"
        );
        assert!(write_in(root.path(), Path::new("call.txt"), " \n").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlink_output() {
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join("linked");
        std::os::unix::fs::symlink(root.path(), &link).unwrap();
        assert!(write_in(&link, Path::new("call.txt"), "text").is_err());
        std::os::unix::fs::symlink(
            root.path().join("elsewhere"),
            root.path().join(".gitignore"),
        )
        .unwrap();
        assert!(write_in(root.path(), Path::new("call.txt"), "text").is_err());
    }
}
