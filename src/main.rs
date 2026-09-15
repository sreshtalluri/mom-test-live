//! Consent always precedes reading input, loading models, or making downloads.
mod consent;
mod decode;
mod detect;
mod handoff;
mod labels;
mod model_catalog;
mod models;
mod output;
mod speech;
mod transcribe;
mod transcript;

use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(
    version,
    about = "Prepare a call transcript locally, then hand it to your Mom Test skills."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Import one recording or transcript; requires interactive consent every time.
    Import {
        file: PathBuf,
        /// Local multilingual Whisper model. Never falls back to another size.
        #[arg(long, default_value = "base", value_parser = ["tiny", "base", "small"])]
        model: String,
        /// Use your own trusted GGML model file instead of a managed model.
        #[arg(long, conflicts_with = "model")]
        model_path: Option<PathBuf>,
        /// Forbid downloads; use only a bundled, cached, or explicit model.
        #[arg(long)]
        offline: bool,
        /// Spoken language code, or auto to detect it. Never translates speech.
        #[arg(long, default_value = "auto", value_parser = parse_language)]
        language: String,
        /// Inference CPU threads (default: available cores, up to 8).
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=64))]
        threads: Option<u8>,
    },
    /// List models or download one ahead of an offline call. No recordings involved.
    Models {
        #[command(subcommand)]
        command: ModelCommand,
    },
}

#[derive(Subcommand)]
enum ModelCommand {
    List,
    Download {
        #[arg(default_value = "base", value_parser = ["tiny", "base", "small"])]
        model: String,
    },
}

fn parse_language(value: &str) -> Result<String, String> {
    if value == "auto" || whisper_rs::get_lang_id(value).is_some() {
        Ok(value.to_string())
    } else {
        Err("unknown Whisper language; use a language code such as en, es, fr, or auto".into())
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let Command::Import {
        file,
        model,
        model_path,
        offline,
        language,
        threads,
    } = cli.command
    else {
        return match cli.command {
            Command::Models {
                command: ModelCommand::List,
            } => models::list(),
            Command::Models {
                command: ModelCommand::Download { model },
            } => {
                let path = models::download_named(&model)?;
                println!("Verified model: {}", path.display());
                Ok(())
            }
            _ => unreachable!(),
        };
    };

    // Metadata only. Directories and device files must not enter the pipeline.
    let metadata =
        fs::metadata(&file).map_err(|e| format!("couldn't access {}: {e}", file.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a regular file", file.display()));
    }
    consent::require_consent(&mut std::io::stdin().lock(), &mut std::io::stdout())?;

    let kind = detect::classify(&file)?;
    let raw_text = match kind {
        detect::InputKind::Text => transcript::read(&file)?,
        detect::InputKind::Audio => transcribe::run(
            &file,
            &transcribe::Options {
                model: &model,
                model_path: model_path.as_deref(),
                offline,
                language: &language,
                threads,
            },
        )?,
    };
    if !raw_text.chars().any(char::is_alphanumeric) {
        return Err(
            "the transcript is empty or contains no readable text; nothing was saved".into(),
        );
    }

    // Audio is always unlabeled, even if the model hallucinates a colon/name.
    if kind == detect::InputKind::Audio || !labels::check(&raw_text).already_labeled {
        eprintln!(
            "warning: this transcript has no verified speaker separation. Open the saved file \
            and add F: / C: (or speaker names) before mom-test-debrief for accurate talk-ratio, \
            pitching, and leading scores. Audio imports never assign speaker identities."
        );
    }
    let out_path = output::write_transcript(&file, &raw_text)?;
    handoff::print_instructions(&out_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_contract() {
        assert!(Cli::try_parse_from(["mt", "import", "call.wav"]).is_ok());
        for args in [
            vec!["mt", "import"],
            vec!["mt", "transcribe", "call.wav"],
            vec!["mt", "import", "call.wav", "extra"],
            vec!["mt", "import", "call.wav", "--yes"],
            vec!["mt", "import", "call.wav", "--force"],
            vec!["mt", "import", "call.wav", "--threads", "0"],
            vec!["mt", "import", "call.wav", "--language", "bogus"],
            vec!["mt", "import", "call.wav", "--model", "large"],
            vec![
                "mt",
                "import",
                "call.wav",
                "--model",
                "tiny",
                "--model-path",
                "x.bin",
            ],
        ] {
            assert!(Cli::try_parse_from(&args).is_err(), "accepted {args:?}");
        }
        assert!(Cli::try_parse_from([
            "mt",
            "import",
            "call.wav",
            "--model-path",
            "x.bin",
            "--offline"
        ])
        .is_ok());
    }
}
