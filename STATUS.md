# mom-test-live — Project Status

**Last updated:** 2026-09-07

## What's Live (main)

- CLI scaffold (Rust), `mom-test-live import <file>`, 41 tests passing, release build clean.
- Hard, unskippable consent gate — no bypass flag, verified via a real pty (not just mocked).
- Input-type detection by extension + magic-byte sanity check.
- Real audio decoding: any container Symphonia understands (WAV/MP3/M4A/FLAC/OGG) → mono 16kHz PCM, verified against a real synthesized WAV through the actual decode pipeline, not just synthetic arrays.
- Speaker-label detection (F:/C:, named speakers, Otter-style "Speaker 1:") via one general heuristic.
- Output to gitignored `mom-test-live/`, atomic write.
- Hand-off message: paste/attach instructions for the user's own coding agent to run `mom-test-debrief` then `mom-test-memory record` — this tool never calls an LLM or writes to `discovery/` itself.

## What's In Progress

- **v0 — audio transcription** — 🚧 code complete except inference
  - Decoding done and tested (`decode.rs`)
  - `whisper-rs` added as a real dependency; its C++ core (whisper.cpp) builds cleanly here
  - Not yet written: the actual `WhisperContext`/`FullParams` inference call in `transcribe.rs`, and model download/caching (see TODOS.md)
  - Running `import` on audio today: decodes for real, then fails honestly with "not wired up yet" — never a crash, never a fake transcript

- **v1 — live in-call overlay** — ⬜ not started
  - Design direction set: dual-stream capture (mic + system audio output) gives founder/customer separation for free, no Diart needed for the core 1:1-call case — works identically for Zoom, Meet, phone, or in-person, since it's OS-level, not app-specific
  - This insight isn't in `docs/designs/live-coach-companion.md` yet — logged as a learning, not yet written back to the design doc
  - Explicitly deferred until v0 has real founder usage

## Blocking Items

1. **No real audio test fixture.** All decode tests use synthesized WAV data; nothing has been verified against an actual call recording (real codec quirks, real noise, real length). Unblocks once someone runs it on a real file — worth doing before claiming v0 audio "works."
2. **Model download mechanics undefined** (TODOS.md) — blocks writing real inference code, since `transcribe.rs` needs a model file path to hand `WhisperContext`.

## Key Files

| File | Why it matters |
|---|---|
| `docs/designs/live-coach-companion.md` | Full design history — 3 review rounds, the cost research, the cross-model tensions and how they were resolved |
| `src/transcribe.rs` | Doc comment explains exactly what's left (inference call + model download) and why decode was chosen to run first |
| `src/decode.rs` | Doc comment explains why hand-written linear resampling over the `rubato` crate |
| `TODOS.md` | Model download mechanics + v0 success metrics — both deferred, both needed before real usage |

## Next Steps (priority order)

1. Define model download/caching (TODOS.md item 1): source, checksum, cache path, offline behavior.
2. Wire real whisper.cpp inference in `transcribe.rs` (`WhisperContext` + `FullParams`, per whisper-rs's README example).
3. Test the full pipeline against a real call recording, not synthesized audio.
4. GitHub Actions release workflow — cross-platform binaries (Linux/macOS/Windows), currently only built locally.
5. Once v0 is real end to end: get it in front of a few real founders before starting v1.
6. Write the dual-stream-capture (no-Diart-needed) insight back into the v1 section of the design doc.
