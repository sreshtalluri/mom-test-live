# mom-test-live

A live conversation companion for [The Mom Test](https://github.com/sreshtalluri/mom-test).
Not a meeting copilot that tells you what to say to your customer — a **conversation
contamination firewall** that catches **your own** pitching, leading questions, and
premature-solution talk before it contaminates the interview.

Every existing "invisible AI overlay" tool in this category (Cluely and its clones)
answers "what should I say to them." This one does the opposite: the founder is the one
being coached, not the customer.

Status: **v0 in progress.** The after-call CLI below is real, tested, and working for
text transcripts. Audio transcription isn't wired up yet (see [Status](#status)). The
live in-call overlay (v1) hasn't started. See
[the full design doc](docs/designs/live-coach-companion.md) for the complete rationale,
three rounds of adversarial design review, the eng review that hardened v0's
architecture, and the real cost research behind it.

## Quickstart

You need two things installed: this CLI, and the [mom-test skills](https://github.com/sreshtalluri/mom-test)
in whatever coding agent you use (Claude Code, etc.) — the CLI hands off to those
skills, it doesn't reimplement them.

```sh
# 1. Build the CLI (Rust toolchain required)
git clone https://github.com/sreshtalluri/mom-test-live.git
cd mom-test-live
cargo build --release
# binary is at target/release/mom-test-live

# 2. Run it on a transcript. It will ask for consent before touching the file.
mom-test-live import path/to/call-transcript.txt
```

It prints exactly two things to do next — paste the saved file into your coding agent
and run `/mom-test-debrief`, then `/mom-test-memory record`. If you haven't installed
the mom-test skills in that agent yet, do that first: see
[sreshtalluri/mom-test's install instructions](https://github.com/sreshtalluri/mom-test#install).
Without them, the hand-off commands won't do anything — this CLI only prepares the
transcript, it never calls an LLM or writes to `discovery/` itself.

### macOS: if the binary won't open

An unsigned CLI binary downloaded from the internet can still get flagged by macOS
Gatekeeper on first run, even without a `.app` bundle. If you see "cannot be opened
because the developer cannot be verified," run:

```sh
xattr -d com.apple.quarantine target/release/mom-test-live
```

This is expected for early technical users of an unsigned OSS tool — full
code-signing/notarization is deferred to v1's `.app` (see the design doc's
Distribution Plan).

## Status

**Working today (v0):**
- Hard, unskippable consent gate — no bypass flag, by design
- Input-type detection (extension + a magic-byte sanity check for misnamed files)
- Real audio decoding (any container [Symphonia](https://github.com/pdeljanov/Symphonia)
  understands — WAV/MP3/M4A/FLAC/OGG) into mono 16kHz PCM, verified against real
  encoded audio, not just synthetic test data
- Speaker-label detection — recognizes `F:`/`C:`, named speakers ("Priya Sharma:"),
  and Otter-style ("Speaker 1:") conventions; warns and proceeds if none are found
- Output written to a gitignored `mom-test-live/` folder, atomic write
- Exact hand-off instructions for your coding agent

**Not wired up yet:** the actual whisper.cpp inference call (`whisper-rs` is a real
dependency and its C++ core builds cleanly here — decoding audio into the exact PCM
shape whisper.cpp wants is done and tested; running the model on those samples isn't
written yet). Running `import` on an audio file today gives you a clear "not
implemented yet" error after decoding succeeds, not a crash or a silent failure. Pass
a text transcript for the full working loop in the meantime.

**v1 (later, once v0 has real usage):** a quiet, invisible-until-triggered overlay
during the actual call — and it needs zero integration with Zoom, Meet, or any
specific app. It captures your microphone and the call's system audio output as two
separate streams (`ScreenCaptureKit` on macOS, WASAPI loopback on Windows), so
founder-vs-customer separation comes for free from *which stream the audio came
from*, not from a diarization model untangling one mixed recording — this works
identically for Zoom, Meet, a phone call on speaker, or an in-person meeting. A
founder-violation classifier (Groq, bring-your-own-key, ~$0.005/call) flags the
founder's own pitching/leading, not the customer's answers. The overlay itself is
forked from [OpenCluely](https://github.com/TechyCSR/OpenCluely) for the
invisible-until-triggered window plumbing.

## Why v0 before v1

Building the risky, expensive part (real-time diarization, a classifier that doesn't
exist yet) before anyone's used the cheap part would be exactly the mistake the book
warns against, applied to this project's own roadmap. v0 ships first, gets real
founders using it, and v1 gets built informed by that instead of guesses. (One caveat
from the eng review, worth being honest about: v0 tests whether founders will run the
after-call debrief habit, not whether live in-call nudges would change anyone's
behavior — those are related but different questions. See the design doc's Review
History.)

## Development

```sh
cargo build       # debug build — needs network access (crates) and cmake (whisper.cpp)
cargo test        # 41 unit tests
cargo build --release
```

Two real dependencies: `symphonia` (audio decoding, pure Rust) and `whisper-rs`
(wraps whisper.cpp's C++ core — needs `cmake` on your machine, e.g. `brew install
cmake` on macOS; Linux/Windows CI runners typically have it preinstalled). Everything
else (consent, detect, labels, output, handoff) is pure std, deliberately — no
dependency was added for what a few lines of stdlib cover.

What's left to reach a fully working audio path: `src/transcribe.rs` decodes audio
(done, real, tested) but doesn't yet load a whisper model or run inference — see its
doc comment and [TODOS.md](TODOS.md) for exactly what's next (model download/caching
mechanics, then the `WhisperContext`/`FullParams` call itself, following whisper-rs's
own README example).

## Contributing

v0's core pipeline works; audio transcription is the next real gap (see
[TODOS.md](TODOS.md) and the `todo!()`/stub markers in `src/`). PRs welcome once
that's further along — for now, watch this repo or the
[parent skills repo](https://github.com/sreshtalluri/mom-test) for updates.

## License

MIT.
