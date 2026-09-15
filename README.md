# mom-test-live

Turn a customer-call recording or transcript into a readable local transcript, then
hand it to [the Mom Test skills](https://github.com/sreshtalluri/mom-test) in your
coding agent for a debrief and evidence recording.

**v0 is implemented:** real local speech transcription, mandatory per-call consent,
speaker-label warnings, private output, and an exact hand-off. No API key or hosted
service is required. The live coaching overlay is the next product stage, after
founders have used this after-call workflow. See [STATUS.md](STATUS.md) for tested
coverage and remaining acceptance work.

Native review builds and the publication checklist are described in
[docs/releases.md](docs/releases.md). For end-to-end product validation, use the
[founder trial protocol](docs/founder-trials.md).

## Run it

Build from source with a stable Rust toolchain, a C++ compiler, CMake, and libclang:

```sh
git clone https://github.com/sreshtalluri/mom-test-live.git
cd mom-test-live
cargo build --release --locked

# Run from your founder project so the output folder lands beside discovery/.
# Use the full binary path, or add target/release to PATH.
./target/release/mom-test-live import path/to/call.txt
./target/release/mom-test-live import path/to/call.m4a
```

Type `YES` when the CLI asks you to confirm the right to process that specific
call. It prompts before reading content, running speech detection, or loading or
downloading a transcription model. Piped input and bypass flags are rejected.

A source build downloads the multilingual **base model (~148 MB)** on its first
audio import. Download progress is shown and SHA-256 is verified. Subsequent imports
work offline. Release packaging embeds that model directly into a single executable,
so packaged builds also work on their first run without network access. This branch
adds that packaging workflow; it does not imply a release has already been published.

The CLI prints the saved transcript's absolute path and these steps:

1. Paste or attach the saved transcript in your coding agent and run `/mom-test-debrief`.
2. Once the debrief is ready, run `/mom-test-memory record`.

Install [the skills](https://github.com/sreshtalluri/mom-test#install) in that agent
first. The agent reads your actual assumptions and asks before recording evidence.
This CLI prepares the transcript; it does not call a conversational LLM, generate
a debrief, or write to `discovery/`.

## Inputs and output

- UTF-8 TXT/Markdown, preserving supplied text and speaker labels.
- SRT/WebVTT caption exports, removing cue metadata and preserving names in voice tags.
- WAV, MP3, AAC/ALAC inside M4A or MP4, FLAC, and OGG/Vorbis. WebM is supported only
  when its codec is supported by Symphonia; **Opus and raw ADTS AAC are not supported**.
  Unsupported/corrupt input produces an error; convert it to PCM WAV before retrying.
- Audio is decoded to mono 16 kHz and processed by local Silero voice detection and
  Whisper. No audio or transcript is uploaded. Model downloads contact Hugging Face.
- Audio output has timestamps and **no inferred speaker identities**. Add `F:` and
  `C:` or real names using your editor for accurate talk-ratio and pitching scores.
- Transcripts go in `./mom-test-live/`, with an enforced `*\n` .gitignore and atomic,
  unique filenames. On Unix, newly created directories are mode 700 and transcripts
  are mode 600. Existing git-tracked files are not made untracked by a .gitignore.

Silent audio, non-speech detected by VAD, invalid samples, empty text, and detected
decoding/model failures produce no transcript. Speech detection and recognition are
probabilistic: review wording against the recording, especially with noise, accents,
overlapping speakers, or multiple languages. This is not automatic evidence validation.

## Models and offline use

```sh
mom-test-live models list
mom-test-live models download base
mom-test-live models download small
mom-test-live import call.m4a --offline
mom-test-live import call.wav --model small --language en --threads 4
mom-test-live import call.wav --model-path /path/to/trusted-ggml-model.bin --offline
```

`tiny` (~78 MB), `base` (~148 MB, default), and `small` (~488 MB) are multilingual.
The language defaults to automatic detection; `--language en` pins English.
Speech stays in its original language. The selected model is never silently
replaced by a different size.

Cache paths:

- macOS: `~/Library/Caches/mom-test-live/models/`
- Linux: `$XDG_CACHE_HOME/mom-test-live/models/`, otherwise `~/.cache/mom-test-live/models/`
- Windows: `%LOCALAPPDATA%\mom-test-live\models\`
- Override on any OS with `MOM_TEST_MODEL_DIR`.

Managed files are size- and SHA-256-checked before each use. Bundled base weights are
verified at build time. A custom `--model-path` must be a trusted GGML Whisper model;
it has no upstream checksum guarantee. A corrupt cached model is reported with
repair instructions, never overwritten silently. See [docs/models.md](docs/models.md)
for source, license, proxy, interruption, timeout, and disk-space behavior.

## Development and verification

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked

# Real terminal gate and failure cases; Python stdlib only, macOS/Linux.
python3 scripts/e2e.py --binary target/debug/mom-test-live

# Real recorded-speech inference on any OS.
cargo run --release --locked -- models download base
cargo test --release --locked transcribes_real_recorded_speech_offline -- --ignored

# Also transcode into common codecs when FFmpeg is installed (test tooling only).
python3 scripts/e2e.py --binary target/release/mom-test-live --audio

# Build an offline archive with the base model inside the executable.
python3 scripts/package.py --model /path/to/ggml-base.bin
```

No Python packages are required. CI covers Linux, macOS Apple Silicon, macOS Intel,
and Windows. PTY tests run on Unix; portable process tests and real offline
inference also run on Windows. The release workflow runs the tests, packages native
binaries and checksums, and prepares a draft GitHub Release on a version tag.
Publishing that draft is a separate maintainer action.

On macOS, install build prerequisites with `xcode-select --install` and `brew install cmake`.
On Debian/Ubuntu: `sudo apt-get install build-essential cmake clang libclang-dev`.
On Windows: install Rust MSVC, Visual Studio C++ Build Tools, CMake, and LLVM; set
`LIBCLANG_PATH` to LLVM's `bin` directory if bindgen cannot find it.

Packaged binaries are unsigned. If macOS quarantines a downloaded binary, remove
its quarantine attribute only for a build you trust:
`xattr -d com.apple.quarantine ./mom-test-live`.

## Direction

The eventual live product quietly flags the founder's own pitching, leading, and
premature solutions during a conversation. It does not supply answers about the
customer. The approved design sequences the after-call workflow first, then an
OpenCluely-based overlay with local capture/transcription and a bring-your-own-key
text classifier. See [the design](docs/designs/live-coach-companion.md).

## License

MIT. Bundled components have their own [notices](THIRD_PARTY.md).
