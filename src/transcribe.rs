//! Audio transcription — decoding is real (see `decode.rs`), whisper.cpp
//! inference itself is NOT YET IMPLEMENTED.
//!
//! `whisper-rs` is a real dependency (Cargo.toml) and its C++ core builds
//! successfully in this environment (contrary to an earlier assumption in
//! this file's history — crates.io's own web frontend 403s from this
//! sandbox, but `index.crates.io`/`static.crates.io`, which is what Cargo
//! actually uses, don't; `cmake` installs cleanly via `brew`). What's left is
//! wiring `WhisperContext`/`FullParams` (see whisper-rs's README) plus the
//! model-download mechanics tracked in TODOS.md ("Model download
//! mechanics" — fail clear on failure, no silent fallback, per finding 2A).
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
    use std::fs;

    /// Minimal valid 16-bit PCM mono WAV, so this test exercises the real
    /// intended failure mode — decoding succeeds, transcription itself is
    /// what's still unimplemented — rather than an incidental "file not
    /// found" from decode.rs, which is a different, already-tested path
    /// (see decode::tests::run_errors_clearly_on_a_nonexistent_file).
    fn write_minimal_wav() -> std::path::PathBuf {
        let sample_rate: u32 = 16_000;
        let samples: Vec<i16> = (0..1600).map(|i| ((i % 100) * 100) as i16).collect(); // 0.1s
        let data_size = (samples.len() * 2) as u32;

        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
        wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat = PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // NumChannels = mono
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // ByteRate
        wav.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign
        wav.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        for s in samples {
            wav.extend_from_slice(&s.to_le_bytes());
        }

        let dir = std::env::temp_dir().join(format!(
            "mom-test-live-transcribe-test-{:?}",
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("minimal.wav");
        fs::write(&path, &wav).unwrap();
        path
    }

    #[test]
    fn decodes_real_audio_then_fails_clearly_on_unimplemented_inference() {
        let path = write_minimal_wav();
        let result = run(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Must NOT be a decode error — decoding this valid WAV should succeed;
        // the failure must come from the (still unimplemented) inference step.
        assert!(
            !err.contains("couldn't open") && !err.contains("unrecognized or corrupt"),
            "expected an inference-not-implemented error, got a decode error instead: {err}"
        );
        assert!(err.contains("isn't wired up"));
    }
}
