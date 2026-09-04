//! Audio transcription — NOT YET IMPLEMENTED.
//!
//! Needs `whisper-rs` or `whisper-cpp-plus-rs` (see finding 1A: Rust chosen
//! specifically for the cross-platform single-binary story these crates
//! provide) plus the model-download mechanics tracked in TODOS.md ("Model
//! download mechanics" — fail clear on failure, no silent fallback, per
//! finding 2A). Neither dependency was reachable when this scaffold was built:
//! crates.io returned 403 from this sandbox, and `cmake` (required to build
//! whisper.cpp's C++ core regardless of which Rust binding wraps it) wasn't
//! installed. See Cargo.toml's commented-out dependency lines.
//!
//! Everything upstream and downstream of this module (input detection, label
//! detection, output writing, hand-off) is fully implemented and tested
//! independent of this gap — see `main.rs`'s pipeline diagram.

use crate::decode;
use std::path::Path;

/// Decoded, transcribed text from an audio file. Returns a clear error today;
/// never panics, never returns a fabricated or empty transcript.
///
/// Intended real pipeline: `decode::run` produces PCM samples, then those get
/// fed to a whisper model (whisper-rs/whisper-cpp-plus-rs, per finding 1A).
/// Delegating to `decode::run` first, rather than short-circuiting here
/// directly, means the moment decoding is implemented this naturally starts
/// exercising it instead of needing a second edit.
pub fn run(path: &Path) -> Result<String, String> {
    let _samples = decode::run(path)?;
    Err(
        "audio transcription isn't wired up yet in this build — needs whisper-rs/\
         whisper-cpp-plus-rs (see TODOS.md, \"Model download mechanics\"). \
         Pass a text transcript instead for now."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_a_clear_error_not_a_panic_or_fake_transcript() {
        let result = run(&PathBuf::from("call.wav"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("isn't wired up"));
    }
}
