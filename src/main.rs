//! mom-test-live: after-call transcript prep for The Mom Test.
//!
//! Pipeline (canonical order — see docs/designs/live-coach-companion.md and the
//! plan-eng-review that hardened it):
//!
//!   ┌─────────┐   ┌─────────────┐   ┌──────────┐   ┌───────────┐   ┌────────┐   ┌────────┐
//!   │  parse  │──▶│ path exists │──▶│ CONSENT  │──▶│  detect   │──▶│ decode │──▶│ write  │──▶ hand-off
//!   │  args   │   │ (no read)   │   │  GATE    │   │ input type│   │/ read  │   │ output │
//!   └─────────┘   └─────────────┘   └──────────┘   └───────────┘   └────────┘   └────────┘
//!
//! The consent gate is unconditionally first (before the file's content is ever
//! touched) and has no bypass flag — see docs/designs/live-coach-companion.md,
//! Constraints, and the plan-eng-review cross-model tension that reaffirmed this.

mod consent;
mod decode;
mod detect;
mod handoff;
mod labels;
mod output;
mod transcribe;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let path = parse_args(args)?;

    // Step: path validation. Metadata only — the file's content is not read here.
    if !path.exists() {
        return Err(format!(
            "no such file: {}\n\nUsage: mom-test-live import <file>",
            path.display()
        ));
    }

    // Step: CONSENT GATE. First real action. No file content has been read yet.
    // No bypass flag exists, deliberately — see the design's Constraints section.
    consent::require_consent(&mut std::io::stdin().lock(), &mut std::io::stdout())?;

    // Step: detect input type (extension + magic-byte sanity check).
    let kind = detect::classify(&path)?;

    // Step: get transcript text, either by reading it directly or transcribing audio.
    let raw_text = match kind {
        detect::InputKind::Text => {
            fs::read_to_string(&path).map_err(|e| format!("couldn't read {}: {e}", path.display()))?
        }
        // src/transcribe.rs (which delegates to src/decode.rs) is a real module
        // with a real error today, not inline logic here — see both files'
        // doc comments for why: whisper-rs/whisper-cpp-plus-rs and symphonia
        // couldn't be fetched in the environment this scaffold was built in.
        // The rest of this pipeline (labels, output, hand-off) is fully
        // implemented and tested independent of this branch.
        detect::InputKind::Audio => transcribe::run(&path)?,
    };

    // Step: speaker-label detection. Never fabricates labels — warns and proceeds.
    let label_check = labels::check(&raw_text);
    if !label_check.already_labeled {
        eprintln!(
            "warning: this transcript has no speaker labels. A debrief needs speaker \
             separation to score talk-ratio, pitching, and leading accurately.\n\
             Open the file and add labels (e.g. \"F: ...\" / \"C: ...\", or \"Name: ...\") \
             before handing it to mom-test-debrief, if you want accurate scoring."
        );
    }

    // Step: write output. Gitignored by default, atomic write.
    let out_path = output::write_transcript(&path, &raw_text)?;

    // Step: hand-off. Exact, pasteable, no paraphrase needed.
    handoff::print_instructions(&out_path);

    Ok(())
}

fn parse_args(args: &[String]) -> Result<PathBuf, String> {
    match args {
        [_bin, cmd, file] if cmd == "import" => Ok(PathBuf::from(file)),
        _ => Err("Usage: mom-test-live import <file>".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_requires_import_subcommand_and_one_file() {
        let ok = parse_args(&[
            "mom-test-live".into(),
            "import".into(),
            "call.txt".into(),
        ]);
        assert_eq!(ok, Ok(PathBuf::from("call.txt")));
    }

    #[test]
    fn parse_args_rejects_missing_file() {
        let err = parse_args(&["mom-test-live".into(), "import".into()]);
        assert!(err.is_err());
    }

    #[test]
    fn parse_args_rejects_unknown_subcommand() {
        let err = parse_args(&[
            "mom-test-live".into(),
            "transcribe".into(),
            "call.txt".into(),
        ]);
        assert!(err.is_err());
    }

    #[test]
    fn parse_args_rejects_extra_arguments() {
        let err = parse_args(&[
            "mom-test-live".into(),
            "import".into(),
            "call.txt".into(),
            "extra".into(),
        ]);
        assert!(err.is_err());
    }

    #[test]
    fn run_errors_clearly_on_nonexistent_path() {
        let args = vec![
            "mom-test-live".to_string(),
            "import".to_string(),
            "/definitely/does/not/exist/on/this/machine.txt".to_string(),
        ];
        let err = run(&args).unwrap_err();
        assert!(err.contains("no such file"));
    }
}
