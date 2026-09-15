//! Downloads never contain call data. Managed models are verified before every load.
//! Unique temporary files prevent interrupted/concurrent downloads exposing partial models.
use crate::model_catalog::{self, Model};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub enum Source {
    File(PathBuf),
    #[cfg(feature = "bundled-model")]
    Embedded(&'static [u8]),
}

pub fn cache_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MOM_TEST_MODEL_DIR").filter(|p| !p.is_empty()) {
        return Ok(path.into());
    }
    dirs::cache_dir()
        .map(|p| p.join("mom-test-live").join("models"))
        .ok_or_else(|| "couldn't locate a cache directory; set MOM_TEST_MODEL_DIR".into())
}

pub fn list() -> Result<(), String> {
    let dir = cache_dir()?;
    println!(
        "Models (multilingual, MIT; base is default)\nCache: {}",
        dir.display()
    );
    for model in model_catalog::MODELS {
        let path = dir.join(format!("ggml-{}.bin", model.name));
        let state = if cfg!(feature = "bundled-model") && model.name == "base" {
            "bundled"
        } else if path.exists() {
            "on disk (verified on use)"
        } else {
            "not downloaded"
        };
        println!(
            "  {:5} {:>4} MB  {state}",
            model.name,
            model.bytes / 1_000_000
        );
    }
    Ok(())
}

pub fn resolve(name: &str, explicit: Option<&Path>, offline: bool) -> Result<Source, String> {
    if let Some(path) = explicit {
        let meta = fs::metadata(path)
            .map_err(|e| format!("couldn't access model {}: {e}", path.display()))?;
        if !meta.is_file() || meta.len() < 1024 {
            return Err(format!(
                "{} is not a usable GGML model file",
                path.display()
            ));
        }
        // User-supplied models have no known checksum; whisper.cpp validates the format.
        return Ok(Source::File(path.to_path_buf()));
    }
    let model = model_catalog::find(name)?;
    #[cfg(feature = "bundled-model")]
    if name == "base" {
        return Ok(Source::Embedded(include_bytes!(env!(
            "MOM_TEST_BUNDLE_MODEL"
        ))));
    }
    ensure(model, &cache_dir()?, offline).map(Source::File)
}

pub fn download_named(name: &str) -> Result<PathBuf, String> {
    ensure(model_catalog::find(name)?, &cache_dir()?, false)
}

fn ensure(model: &Model, dir: &Path, offline: bool) -> Result<PathBuf, String> {
    let path = dir.join(format!("ggml-{}.bin", model.name));
    if path.exists() {
        verify(&path, model).map_err(|e| format!("{e}\nRemove that damaged model file and run `mom-test-live models download {}` again.", model.name))?;
        return Ok(path);
    }
    if offline {
        return Err(format!(
            "model {} is not cached at {} and --offline forbids downloads. \
            Run `mom-test-live models download {}` while online, or use --model-path.",
            model.name,
            path.display(),
            model.name
        ));
    }
    fs::create_dir_all(dir)
        .map_err(|e| format!("couldn't create model cache {}: {e}", dir.display()))?;
    eprintln!(
        "Downloading {} model ({} MB) from Hugging Face; audio stays on this device.",
        model.name,
        model.bytes / 1_000_000
    );
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model.name
    );
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_recv_body(Some(Duration::from_secs(1800)))
        .timeout_global(Some(Duration::from_secs(1800)))
        .https_only(true)
        .build()
        .into();
    let mut response = agent.get(&url).call()
        .map_err(|e| format!("model download failed: {e}. Check your connection/proxy or use --offline with a cached model. Retry to restart the download."))?;
    if response.status() != 200 {
        return Err(format!(
            "model download returned HTTP {}; expected 200",
            response.status()
        ));
    }
    save_download(&mut response.body_mut().as_reader(), model, &path)?;
    eprintln!("Model verified and cached: {}", path.display());
    Ok(path)
}

pub fn verify(path: &Path, model: &Model) -> Result<(), String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("couldn't read model {}: {e}", path.display()))?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if size != model.bytes {
        return Err(format!(
            "model size mismatch at {}: expected {} bytes, got {size}",
            path.display(),
            model.bytes
        ));
    }
    let mut digest = Sha256::new();
    std::io::copy(&mut file, &mut digest).map_err(|e| format!("couldn't hash model: {e}"))?;
    if format!("{:x}", digest.finalize()) != model.sha256 {
        return Err(format!(
            "model SHA-256 checksum mismatch at {}",
            path.display()
        ));
    }
    Ok(())
}

fn save_download(reader: &mut impl Read, model: &Model, destination: &Path) -> Result<(), String> {
    let dir = destination
        .parent()
        .ok_or("model destination has no parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| format!("couldn't create model download in {}: {e}", dir.display()))?;
    let mut copied = 0u64;
    let mut reported = 0u64;
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|e| format!("model download interrupted: {e}; retry to restart"))?;
        if count == 0 {
            break;
        }
        copied += count as u64;
        if copied > model.bytes {
            return Err("model download exceeded its expected size; nothing was installed".into());
        }
        temporary
            .write_all(&buffer[..count])
            .map_err(|e| format!("couldn't save model download (check free disk space): {e}"))?;
        if copied - reported >= 25_000_000 {
            eprintln!(
                "Downloaded {} / {} MB",
                copied / 1_000_000,
                model.bytes / 1_000_000
            );
            reported = copied;
        }
    }
    temporary
        .as_file()
        .sync_all()
        .map_err(|e| format!("couldn't flush model to disk: {e}"))?;
    verify(temporary.path(), model)?;
    match temporary.persist_noclobber(destination) {
        Ok(_) => Ok(()),
        Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
            // Another importer completed first. Never overwrite it; verify the winner.
            verify(destination, model)
        }
        Err(e) => Err(format!(
            "couldn't install model {}: {}",
            destination.display(),
            e.error
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> Model {
        Model {
            name: "test",
            bytes: 3,
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        }
    }

    #[test]
    fn download_is_verified_and_published_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("model.bin");
        save_download(&mut Cursor::new(b"abc"), &fixture(), &dest).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"abc");
        save_download(&mut Cursor::new(b"abc"), &fixture(), &dest).unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn corrupt_short_and_oversized_downloads_never_install() {
        for bytes in [b"bad".as_slice(), b"ab", b"abcd"] {
            let dir = tempfile::tempdir().unwrap();
            let dest = dir.path().join("model.bin");
            assert!(save_download(&mut Cursor::new(bytes), &fixture(), &dest).is_err());
            assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn offline_missing_model_does_not_create_cache() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");
        assert!(ensure(&fixture(), &missing, true)
            .unwrap_err()
            .contains("--offline"));
        assert!(!missing.exists());
    }

    #[test]
    fn existing_corrupt_model_is_never_silently_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ggml-test.bin");
        fs::write(&path, b"bad").unwrap();
        assert!(ensure(&fixture(), dir.path(), false)
            .unwrap_err()
            .contains("checksum"));
        assert_eq!(fs::read(&path).unwrap(), b"bad");
    }

    #[test]
    fn interrupted_download_removes_partial_file() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionReset,
                    "interrupted",
                ))
            }
        }
        let dir = tempfile::tempdir().unwrap();
        assert!(save_download(&mut Broken, &fixture(), &dir.path().join("m.bin")).is_err());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
