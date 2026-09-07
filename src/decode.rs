//! Audio decoding: any container Symphonia understands → mono f32 PCM @ 16kHz,
//! the format whisper.cpp requires (finding 4A).
//!
//! Resampling is a hand-written linear interpolation, not a dedicated crate
//! (`rubato` was evaluated and dropped — its fixed-chunk async API has real
//! complexity around delay trimming and partial chunks that's genuinely
//! error-prone to get right without reference audio to verify against; linear
//! interpolation is ~15 lines, has no dependency, and every property that
//! matters — output length, endpoint values, monotonic sampling — is directly
//! testable with synthetic signals below). Quality is lower than a proper sinc
//! resampler, but whisper.cpp is trained on diverse real-world audio and
//! tolerant of this; if transcription quality on downsampled input turns out
//! to be a real problem in practice, that's the signal to revisit this with
//! real audio fixtures in hand, not before.

use std::path::Path;
use symphonia::core::audio::sample::Sample;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

pub fn run(path: &Path) -> Result<Vec<f32>, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("couldn't open {}: {e}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| format!("{}: unrecognized or corrupt audio container: {e}", path.display()))?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| format!("{}: no audio track found", path.display()))?;
    let track_id = track.id;
    let codec_params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or_else(|| format!("{}: unsupported or missing audio codec parameters", path.display()))?
        .clone();

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&codec_params, &AudioDecoderOptions::default())
        .map_err(|e| format!("{}: unsupported audio codec: {e}", path.display()))?;

    let mut interleaved: Vec<f32> = Vec::new();
    let mut channels: Option<usize> = None;
    let mut source_rate: Option<u32> = None;
    let mut scratch: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break, // end of stream
            Err(SymphoniaError::ResetRequired) => {
                return Err(format!(
                    "{}: the track list changed mid-file (e.g. chained OGG streams) — not \
                     supported in this build",
                    path.display()
                ));
            }
            Err(e) => return Err(format!("{}: error reading audio stream: {e}", path.display())),
        };

        if packet.track_id != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let spec = audio_buf.spec();
                channels.get_or_insert(spec.channels().count());
                source_rate.get_or_insert(spec.rate());

                scratch.resize(audio_buf.samples_interleaved(), f32::MID);
                audio_buf.copy_to_slice_interleaved(&mut scratch);
                interleaved.extend_from_slice(&scratch);
            }
            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(format!("{}: decode error: {e}", path.display())),
        }
    }

    let channels = channels.ok_or_else(|| format!("{}: no decodable audio packets found", path.display()))?;
    let source_rate = source_rate.unwrap_or(WHISPER_SAMPLE_RATE);

    if interleaved.is_empty() {
        return Err(format!("{}: decoded to zero audio samples (silent or empty file)", path.display()));
    }

    let mono = downmix_to_mono(&interleaved, channels);
    let resampled = if source_rate == WHISPER_SAMPLE_RATE {
        mono
    } else {
        resample_linear(&mono, source_rate, WHISPER_SAMPLE_RATE)
    };

    Ok(resampled)
}

/// Average all channels into one. A no-op (returns the input) when `channels <= 1`.
fn downmix_to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Linear-interpolation resample. See the module doc comment for why this
/// over a dedicated resampling crate.
fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() || from_rate == to_rate {
        return input.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = ((input.len() as f64) / ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos.floor() as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = input[idx.min(input.len() - 1)];
        let b = input[(idx + 1).min(input.len() - 1)];
        out.push(a + (b - a) * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmix_stereo_averages_channels() {
        // L, R, L, R — averaging (1,3) and (2,4) should give (2, 3).
        let stereo = vec![1.0, 3.0, 2.0, 4.0];
        assert_eq!(downmix_to_mono(&stereo, 2), vec![2.0, 3.0]);
    }

    #[test]
    fn downmix_mono_is_identity() {
        let mono = vec![1.0, 2.0, 3.0];
        assert_eq!(downmix_to_mono(&mono, 1), mono);
    }

    #[test]
    fn downmix_five_one_averages_all_channels() {
        let frame = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0]; // one 6-channel frame, all equal
        assert_eq!(downmix_to_mono(&frame, 6), vec![1.0]);
    }

    #[test]
    fn resample_same_rate_is_identity() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(resample_linear(&input, 16000, 16000), input);
    }

    #[test]
    fn resample_downsamples_to_expected_length() {
        // 32000 -> 16000 halves the sample count.
        let input: Vec<f32> = (0..32000).map(|i| i as f32).collect();
        let out = resample_linear(&input, 32000, 16000);
        assert!((out.len() as i64 - 16000).abs() <= 1);
    }

    #[test]
    fn resample_upsamples_to_expected_length() {
        // 8000 -> 16000 doubles the sample count.
        let input: Vec<f32> = (0..8000).map(|i| i as f32).collect();
        let out = resample_linear(&input, 8000, 16000);
        assert!((out.len() as i64 - 16000).abs() <= 1);
    }

    #[test]
    fn resample_preserves_a_constant_signal() {
        // A DC signal (all-1.0) must resample to all-1.0 — interpolation
        // between equal values can't introduce ripple.
        let input = vec![1.0f32; 1000];
        let out = resample_linear(&input, 44100, 16000);
        assert!(out.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn resample_preserves_endpoints_of_a_ramp() {
        let input: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let out = resample_linear(&input, 44100, 16000);
        assert!((out[0] - input[0]).abs() < 1e-3);
        // The last output sample should be close to the last input sample —
        // linear interpolation clamps at the boundary rather than extrapolating.
        assert!((out[out.len() - 1] - input[input.len() - 1]).abs() < 200.0);
    }

    #[test]
    fn resample_empty_input_returns_empty() {
        let out = resample_linear(&[], 44100, 16000);
        assert!(out.is_empty());
    }

    #[test]
    fn run_errors_clearly_on_a_nonexistent_file() {
        let err = run(std::path::Path::new("/definitely/not/a/real/file.wav")).unwrap_err();
        assert!(err.contains("couldn't open"));
    }

    #[test]
    fn run_errors_clearly_on_a_non_audio_file() {
        let dir = std::env::temp_dir().join(format!(
            "mom-test-live-decode-test-{:?}",
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not-audio.wav");
        std::fs::write(&path, b"this is not a real wav file at all").unwrap();
        let err = run(&path).unwrap_err();
        assert!(err.contains("unrecognized or corrupt"));
    }
}
