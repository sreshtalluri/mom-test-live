# Third-party components

The release executable includes:

- [Whisper model weights](https://github.com/openai/whisper), MIT, Copyright OpenAI.
- [whisper.cpp](https://github.com/ggml-org/whisper.cpp), MIT, Copyright Georgi Gerganov.
- [whisper-rs](https://github.com/tazz4843/whisper-rs), Unlicense.
- [Silero VAD](https://github.com/snakers4/silero-vad), MIT, Copyright Silero Team.
- [Symphonia](https://github.com/pdeljanov/Symphonia), MPL-2.0. Its source is
  unmodified. The exact version (0.6.1) and all component versions are recorded in
  Cargo.lock; corresponding source is available through crates.io and its upstream
  repository. No Symphonia source is modified by this project.

Copies of the Whisper, whisper.cpp, Symphonia, and Silero notices accompany packaged
binaries. Other Rust dependency versions and their declared licenses are available
through `cargo metadata --locked`; all are built from the checked-in Cargo.lock.
