# mom-test-live

Companion to github.com/sreshtalluri/mom-test. Read docs/designs/live-coach-companion.md
before changing architecture. v0 prepares transcripts locally and hands them to the
founder's own agent; it never calls a conversational LLM or writes discovery/.

Consent must precede any input read, speech detection, model load, or download for
an import. No bypass flag or piped confirmation. Model setup commands process no
call data. Audio never receives invented speaker identities.

See STATUS.md for implemented scope and verification; TODOS.md for acceptance work.
Run cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, cargo test
--locked, and the applicable real audio/PTY tests documented in README.md after
pipeline changes. Skills remain owned by the separate mom-test repository.
