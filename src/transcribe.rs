//! Local Whisper inference. No recording or transcript is sent over the network.
use crate::{decode, models};
use std::{path::Path, time::Instant};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct Options<'a> {
    pub model: &'a str,
    pub model_path: Option<&'a Path>,
    pub offline: bool,
    pub language: &'a str,
    pub threads: Option<u8>,
}

pub fn run(path: &Path, options: &Options<'_>) -> Result<String, String> {
    eprintln!("Decoding audio…");
    let mut samples = decode::run(path)?;
    validate_audio(&samples)?;
    whisper_rs::install_logging_hooks();
    eprintln!("Checking for speech locally…");
    let speech = crate::speech::detect(&samples)?;
    let duration = samples.len() as f64 / decode::WHISPER_SAMPLE_RATE as f64;
    // whisper.cpp needs at least one second, including for a short utterance.
    if samples.len() < decode::WHISPER_SAMPLE_RATE as usize {
        samples.resize(decode::WHISPER_SAMPLE_RATE as usize, 0.0);
    }
    let source = models::resolve(options.model, options.model_path, options.offline)?;
    // With no log backend enabled this suppresses whisper/ggml diagnostic chatter.
    // Transcript contents must not stream to the terminal during inference.
    whisper_rs::install_logging_hooks();
    let mut context_params = WhisperContextParameters::default();
    context_params.use_gpu(false); // Portable CPU baseline, no accelerator requirement.
    eprintln!("Loading local Whisper model…");
    let context = match source {
        models::Source::File(path) => WhisperContext::new_with_params(&path, context_params),
        #[cfg(feature = "bundled-model")]
        models::Source::Embedded(bytes) => WhisperContext::new_from_buffer_with_params(bytes, context_params),
    }.map_err(|e| format!("couldn't load Whisper model: {e}. Check that it is a valid GGML Whisper model and there is enough memory."))?;
    let mut state = context
        .create_state()
        .map_err(|e| format!("couldn't allocate transcription state: {e}"))?;
    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: 5,
        patience: -1.0,
    });
    let threads = options.threads.map(i32::from).unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get().min(8) as i32)
            .unwrap_or(1)
    });
    params.set_n_threads(threads);
    params.set_language(if options.language == "auto" {
        None
    } else {
        Some(options.language)
    });
    params.set_translate(false);
    params.set_no_context(true);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_no_speech_thold(0.6);
    let mut reported = -10;
    params.set_progress_callback_safe(move |progress| {
        if progress >= reported + 10 {
            eprintln!("Transcribing: {progress}%");
            reported = progress;
        }
    });
    eprintln!(
        "Transcribing {:.1} minutes locally with {threads} CPU threads…",
        duration / 60.0
    );
    let start = Instant::now();
    state
        .full(params, &samples)
        .map_err(|e| format!("local transcription failed: {e}; no transcript was saved"))?;

    let mut transcript = String::new();
    for segment in state.as_iter() {
        let text = segment
            .to_str()
            .map_err(|e| format!("invalid transcript text from Whisper: {e}"))?
            .trim();
        let overlaps_speech = speech.iter().any(|range| {
            (segment.end_timestamp() as f32) > range.start
                && (segment.start_timestamp() as f32) < range.end
        });
        if !overlaps_speech || segment.no_speech_probability() >= 0.6 || !has_speech_text(text) {
            continue;
        }
        // Timestamps locate the original audio for correction; they are not speaker labels.
        transcript.push_str(&format!(
            "[{}] {}\n",
            timestamp(segment.start_timestamp()),
            text
        ));
    }
    if transcript.is_empty() {
        return Err("no speech detected with sufficient confidence; no transcript was saved. Check the recording or try --model small.".into());
    }
    eprintln!("Transcription finished in {:.1}s. Review it against the recording before treating it as evidence.", start.elapsed().as_secs_f64());
    Ok(transcript)
}

fn validate_audio(samples: &[f32]) -> Result<(), String> {
    if samples.is_empty() || samples.iter().any(|s| !s.is_finite()) {
        return Err("audio is empty or contains invalid samples; no transcript was saved".into());
    }
    if samples.len() < 1600 {
        return Err("audio is too short to transcribe (less than 0.1 seconds)".into());
    }
    // Check AC energy, not just amplitude: digital silence and DC offsets must
    // never reach Whisper, which can hallucinate text on silence.
    let mean = samples.iter().map(|&s| s as f64).sum::<f64>() / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|&s| (s as f64 - mean).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    if variance.sqrt() < 0.00001 {
        return Err("no speech detected: audio is silent or below the usable signal level; no transcript was saved".into());
    }
    Ok(())
}

fn has_speech_text(text: &str) -> bool {
    if (text.starts_with('[') && text.ends_with(']'))
        || (text.starts_with('(') && text.ends_with(')'))
    {
        return false;
    }
    text.chars().any(char::is_alphanumeric)
}

fn timestamp(centiseconds: i64) -> String {
    let cs = centiseconds.max(0);
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        cs / 360_000,
        cs / 6000 % 60,
        cs / 100 % 60,
        cs % 100 * 10
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a cached or bundled base model; run the documented audio test"]
    fn transcribes_real_recorded_speech_offline() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jfk.wav");
        let text = run(
            &fixture,
            &Options {
                model: "base",
                model_path: None,
                offline: true,
                language: "en",
                threads: Some(4),
            },
        )
        .unwrap()
        .to_lowercase();
        for phrase in ["ask not", "your country", "do for"] {
            assert!(text.contains(phrase), "missing {phrase:?}: {text}");
        }
    }

    #[test]
    fn invalid_silent_and_dc_audio_is_rejected_before_model_loading() {
        for samples in [
            vec![],
            vec![0.0; 16000],
            vec![0.5; 16000],
            vec![f32::NAN; 16000],
            vec![f32::INFINITY; 16000],
            vec![0.1; 100],
        ] {
            assert!(validate_audio(&samples).is_err());
        }
        let speech_like: Vec<f32> = (0..16000).map(|i| (i as f32 / 10.0).sin() * 0.1).collect();
        assert!(validate_audio(&speech_like).is_ok());
    }

    #[test]
    fn non_speech_annotations_are_not_transcripts() {
        for text in ["", "  ", "...", "[Music]", "(silence)", "♪"] {
            assert!(!has_speech_text(text));
        }
        assert!(has_speech_text("We tried that last week."));
    }

    #[test]
    fn timestamps_are_readable_and_not_speaker_labels() {
        assert_eq!(timestamp(372_345), "01:02:03.450");
        assert_eq!(timestamp(-5), "00:00:00.000");
    }
}
