# TODOs

## Model download mechanics
**What:** Define the exact source, license, checksum verification, cache location,
offline/proxy behavior, resume support, and disk-space handling for the whisper GGML
model file.
**Why:** The eng review settled the failure *philosophy* (fail clear, no silent
fallback to a smaller model — see docs/designs/live-coach-companion.md) but not the
mechanics. An implementer needs this before writing the download code.
**Pros:** Prevents a whole class of "works on my machine" first-run failures; checksum
verification protects against a corrupted or tampered model file.
**Cons:** Real implementation effort; several small decisions (cache dir convention,
checksum source) that don't need design-doc-level review.
**Context:** whisper.cpp's own `models/download-ggml-model.sh` script + Hugging
Face-hosted GGML files are the reference. Reasonable default: XDG-style cache dir
(`~/.cache/mom-test-live/models/` on Unix, platform-appropriate equivalent on
Windows), sha256 verified against the upstream repo's published hash.
**Depends on / blocked by:** Rust + whisper-rs/whisper-cpp-plus-rs scaffold (in
progress).

## v0 success metrics
**What:** Define what "v0 is working" means beyond "doesn't crash" — number of real
founders who complete the full loop (import → consent → transcript → hand-off run),
where in the pipeline they abandon it, how often the flat-transcript warning fires
(i.e. how often manual F:/C: labeling was needed).
**Why:** Without this, "get it in front of a few real founders" has no way to know if
it succeeded. Also the concrete version of the eng review's cross-model tension
finding: track v0 usage as evidence for "founders will run the after-call habit," not
as evidence for v1's live-coaching demand — those are different questions.
**Pros:** Turns "ship and see" into something evaluable.
**Cons:** For a pre-launch side project with no users yet, formal metrics tracking may
be premature — asking people directly works fine at this scale.
**Context:** Cheapest version: a single opt-in line printed after hand-off ("mind
telling me how this went? [link]"), not instrumented telemetry.
**Depends on / blocked by:** v0 shipped and in front of real founders first.
