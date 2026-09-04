//! Input-type detection: extension first, magic-byte sanity check second.
//!
//! Per finding 3A: detect by extension, unrecognized → clear error. Per TODO 2
//! (folded into build-now scope): a magic-byte check catches the obvious
//! misnamed-file case Codex flagged — a renamed binary passed off as `.txt`, or
//! vice versa — without trying to be an exhaustive codec sniffer. This is
//! deliberately not a full container-format parser: it only needs to catch
//! files that are obviously the wrong kind, per the design decision.

use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputKind {
    Text,
    Audio,
}

const AUDIO_EXTS: &[&str] = &[
    "wav", "mp3", "m4a", "aac", "flac", "ogg", "opus", "webm", "mp4",
];
const TEXT_EXTS: &[&str] = &["txt", "md", "vtt", "srt"];

pub fn classify(path: &Path) -> Result<InputKind, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| unrecognized_extension_error(path))?;

    let head = read_head(path, 512)?;

    if TEXT_EXTS.contains(&ext.as_str()) {
        if looks_binary(&head) {
            return Err(format!(
                "{} has a text extension ({ext}) but its contents look binary, not text. \
                 Check the file — it may be misnamed.",
                path.display()
            ));
        }
        return Ok(InputKind::Text);
    }

    if AUDIO_EXTS.contains(&ext.as_str()) {
        if looks_like_plain_text(&head) {
            return Err(format!(
                "{} has an audio extension ({ext}) but its contents look like plain text, \
                 not audio. Check the file — it may be misnamed.",
                path.display()
            ));
        }
        return Ok(InputKind::Audio);
    }

    Err(unrecognized_extension_error(path))
}

fn unrecognized_extension_error(path: &Path) -> String {
    format!(
        "{}: unrecognized file extension. Known audio: {}. Known transcript: {}. \
         Rename the file or convert it, then try again.",
        path.display(),
        AUDIO_EXTS.join(", "),
        TEXT_EXTS.join(", "),
    )
}

fn read_head(path: &Path, n: usize) -> Result<Vec<u8>, String> {
    let mut f = fs::File::open(path).map_err(|e| format!("couldn't open {}: {e}", path.display()))?;
    let mut buf = vec![0u8; n];
    let read = f
        .read(&mut buf)
        .map_err(|e| format!("couldn't read {}: {e}", path.display()))?;
    buf.truncate(read);
    Ok(buf)
}

/// True if the byte slice looks like binary data rather than text: any NUL
/// byte, or a high proportion of non-printable/non-whitespace control bytes.
fn looks_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false; // an empty file isn't "binary"; a separate concern
    }
    if bytes.contains(&0) {
        return true;
    }
    let control = bytes
        .iter()
        .filter(|&&b| b < 0x09 || (b > 0x0d && b < 0x20))
        .count();
    (control as f64 / bytes.len() as f64) > 0.10
}

/// True if the byte slice looks like plain ASCII/UTF-8 text with essentially no
/// binary noise — the mirror check for a mislabeled audio file.
fn looks_like_plain_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    !looks_binary(bytes) && std::str::from_utf8(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    /// Each call gets its own directory, keyed by a nanosecond timestamp plus
    /// the OS thread id — tests run in parallel by default, so two tests using
    /// the same `name` (e.g. "call.txt") in a shared directory will otherwise
    /// race and clobber each other's file. This bug was caught by the suite
    /// itself: `misnamed_binary_with_text_extension_errors` intermittently
    /// read back `plain_text_file_classified_as_text`'s content before this fix.
    fn write_temp(name: &str, contents: &[u8]) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = format!(
            "{:?}-{}",
            std::thread::current().id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        );
        let dir = std::env::temp_dir()
            .join("mom-test-live-detect-tests")
            .join(nonce);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(contents).unwrap();
        path
    }

    #[test]
    fn plain_text_file_classified_as_text() {
        let p = write_temp("call.txt", b"F: hello\nC: hi there\n");
        assert_eq!(classify(&p), Ok(InputKind::Text));
    }

    #[test]
    fn wav_header_classified_as_audio() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0u8; 4]);
        bytes.extend_from_slice(b"WAVEfmt ");
        let p = write_temp("call.wav", &bytes);
        assert_eq!(classify(&p), Ok(InputKind::Audio));
    }

    #[test]
    fn unrecognized_extension_errors_clearly() {
        let p = write_temp("call.docx", b"whatever");
        let err = classify(&p).unwrap_err();
        assert!(err.contains("unrecognized"));
    }

    #[test]
    fn no_extension_errors_clearly() {
        let p = write_temp("call", b"whatever");
        let err = classify(&p).unwrap_err();
        assert!(err.contains("unrecognized"));
    }

    #[test]
    fn misnamed_binary_with_text_extension_errors() {
        let mut bytes = vec![0u8; 64];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i % 256) as u8; // dense binary noise, definitely not text
        }
        let p = write_temp("call.txt", &bytes);
        let err = classify(&p).unwrap_err();
        assert!(err.contains("look binary"));
    }

    #[test]
    fn misnamed_text_with_audio_extension_errors() {
        let p = write_temp(
            "call.mp3",
            b"This is definitely a plain text transcript, not an audio file at all.",
        );
        let err = classify(&p).unwrap_err();
        assert!(err.contains("look like plain text"));
    }

    #[test]
    fn vtt_extension_is_text() {
        let p = write_temp("call.vtt", b"WEBVTT\n\n00:00:00.000 --> 00:00:02.000\nHello\n");
        assert_eq!(classify(&p), Ok(InputKind::Text));
    }
}
