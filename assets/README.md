# Bundled speech detector

`ggml-silero-v5.1.2.bin` (885,098 bytes) is the MIT-licensed
[Silero VAD conversion distributed by ggml-org](https://huggingface.co/ggml-org/whisper-vad).
It is embedded in every binary. Only its public weights are temporarily written to
disk because whisper-rs's VAD API requires a filename; call audio is never written there.

Source: https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v5.1.2.bin

SHA-256: `29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf`.
Verified during every build. License: [SILERO-LICENSE](SILERO-LICENSE).

The larger Whisper models are deliberately outside git. Their exact sizes and
SHA-256 checksums live in `src/model_catalog.rs`. Release packaging embeds the
verified multilingual base model into the executable.
