//! Local Silero VAD prevents silence/noise from being handed to a text decoder.
//! The small MIT model is always embedded, including ordinary source builds.
use std::io::Write;
use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams, WhisperVadSegment};

pub fn detect(samples: &[f32]) -> Result<Vec<WhisperVadSegment>, String> {
    if samples.len() > i32::MAX as usize {
        return Err("recording exceeds Whisper's sample limit; split it into shorter files".into());
    }
    // The upstream VAD API accepts a filename, so materialize only the public
    // detector weights in a private, auto-cleaned temp file. No audio is written.
    let mut model = tempfile::NamedTempFile::new()
        .map_err(|e| format!("couldn't prepare local speech detector: {e}"))?;
    model
        .write_all(include_bytes!("../assets/ggml-silero-v5.1.2.bin"))
        .map_err(|e| format!("couldn't prepare speech detector: {e}"))?;
    model.flush().map_err(|e| e.to_string())?;
    let path = model
        .path()
        .to_str()
        .ok_or("speech detector temp path must be valid UTF-8")?;
    let mut params = WhisperVadContextParams::default();
    params.set_n_threads(1);
    params.set_use_gpu(false);
    let mut context = WhisperVadContext::new(path, params)
        .map_err(|e| format!("couldn't load local speech detector: {e}"))?;
    let mut params = WhisperVadParams::default();
    params.set_min_speech_duration(150);
    params.set_speech_pad(150);
    let segments = context
        .segments_from_samples(params, samples)
        .map_err(|e| format!("speech detection failed: {e}; no transcript was saved"))?;
    let segments: Vec<_> = segments.collect();
    if segments.is_empty() {
        return Err("no speech detected; no transcript was saved. Check that the recording contains audible voices.".into());
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_speech_passes_but_tone_and_noise_do_not() {
        whisper_rs::install_logging_hooks();
        let fixture =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jfk.wav");
        assert!(!detect(&crate::decode::run(&fixture).unwrap())
            .unwrap()
            .is_empty());
        let tone: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / 16000.0).sin() * 0.2)
            .collect();
        assert!(detect(&tone).is_err());
        let mut seed = 42u32;
        let noise: Vec<f32> = (0..48_000)
            .map(|_| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed as f64 / u32::MAX as f64 - 0.5) as f32 * 0.2
            })
            .collect();
        assert!(detect(&noise).is_err());
    }
}
