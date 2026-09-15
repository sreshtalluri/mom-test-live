# mom-test-live — Project Status

**Last updated:** 2026-09-07

## Implemented locally (v0, 0.2.0)

The after-call product now has a complete executable pipeline:

1. Accept one input path and inspect metadata only.
2. Require interactive, per-call consent before reading content or loading models.
3. Read TXT/Markdown or extract turns from SRT/WebVTT; for audio, decode real codecs
   to mono 16 kHz, reject unusable/non-speech audio with bundled local Silero VAD,
   and run local whisper.cpp inference.
4. Preserve supplied speaker names. Audio is timestamped and explicitly unlabeled.
5. Atomically save a unique transcript in a private, gitignored folder.
6. Print the exact debrief → memory-record hand-off for the founder's own agent.

No API key, conversational LLM call, raw-audio upload, telemetry, or discovery/ write.

Model management: pinned sizes and SHA-256, streaming HTTPS download, OS-specific
cache, strict offline mode, custom trusted model paths, progress, actionable errors,
and no silent fallback. Release builds embed the verified multilingual base model
and VAD weights into one executable. Packaging includes an archive and checksum.

## Verification

On the development Apple Silicon Mac:

- 53 Rust tests passed (49 unit, 3 process, and the explicit offline inference
  test), plus formatting and Clippy with warnings denied.
- 11 end-to-end scenarios: real PTY consent, no piped/bypass consent,
  empty/corrupt input, speech/silence/tone,
  private gitignored output, exact hand-off, no discovery mutation, and concurrent
  imports.
- Real human-recorded JFK speech from whisper.cpp's public fixture, including
  WAV → MP3, AAC/M4A, AAC/MP4, FLAC, and OGG/Vorbis at 44.1 kHz stereo.
- Bundled release binary tested offline with an empty model cache.

The historical speech fixture is not a customer interview. Noise and tone tests do
not establish accuracy for arbitrary noisy calls; VAD and Whisper remain probabilistic.

## Ready for external verification

- CI/release workflows for Linux x64, macOS ARM/Intel, and Windows x64 are written.
  Platforms not executed locally remain unverified until those workflows run.
- Release publication has not been performed. Tagged builds prepare a draft.
- Test the full import → manual labeling/review → agent debrief → memory-record loop
  with at least three real founders and their own recordings/projects.

## v1 direction

The live overlay remains deferred until real founder usage of v0, per the approved
design. The dual-stream microphone/system-output insight is now recorded in the
design with its scope: isolated remote 1:1 calls can use capture source as a role hint;
speakerphone and in-person recordings still contain mixed voices.

## Key files

| File | Purpose |
| --- | --- |
| src/main.rs | CLI and consent-first pipeline |
| src/models.rs, src/model_catalog.rs | Verified model downloads/cache |
| src/speech.rs, src/transcribe.rs | Local VAD and Whisper inference |
| src/transcript.rs | Caption export conversion without inventing speaker identities |
| src/output.rs | Atomic private output |
| scripts/e2e.py | Actual terminal and codec acceptance tests |
| scripts/package.py | Native offline binary packaging |
| docs/models.md | Exact model mechanics |
| TODOS.md | Founder and distribution acceptance work |
