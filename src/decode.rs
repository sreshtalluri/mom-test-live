//! Audio decoding — NOT YET IMPLEMENTED.
//!
//! whisper.cpp wants decoded PCM samples, not arbitrary `.mp3`/`.m4a`/`.webm`
//! bytes (finding 4A). Needs the `symphonia` crate (pure Rust, no external
//! binary dependency, chosen over shelling out to FFmpeg) — unreachable from
//! this sandbox for the same reason noted in `transcribe.rs`.

use std::path::Path;

/// Decoded mono PCM samples at whisper's expected sample rate. Returns a clear
/// error today; the real implementation should reject the file outright on a
/// genuinely corrupt/unreadable container rather than returning empty samples.
pub fn run(_path: &Path) -> Result<Vec<f32>, String> {
    Err(
        "audio decoding isn't wired up yet in this build — needs the symphonia crate \
         (see TODOS.md and Cargo.toml)."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_a_clear_error_not_a_panic_or_empty_samples() {
        let result = run(&PathBuf::from("call.mp3"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("isn't wired up"));
    }
}
