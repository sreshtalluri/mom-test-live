# Remaining product work

v0 implementation is complete; see STATUS.md for executed checks.

## Founder acceptance

Recruit at least three founders who each complete import → review/label → debrief →
memory record on their own call and project. Record, with their permission:

- Whether the full loop completed and where they stopped.
- Input format and approximate duration; time to a usable transcript.
- Whether manual speaker labeling was necessary and whether they completed it.
- Transcription mistakes that changed the meaning of a customer statement.
- Whether the resulting evidence changed their next question or decision.

Use direct interviews and a small manual log; no telemetry or contact messages are
sent by the app. This tests the after-call habit, not demand for live coaching.

## Distribution acceptance

Run the checked-in CI/release workflows on GitHub and review their native Linux,
macOS Intel, macOS ARM, and Windows results. Publish the prepared draft only after
review. The local development run cannot certify unexecuted platforms or a published
release. Signing/notarization remains deferred per the design.

## v1 after founder usage

Build the live overlay only after the v0 acceptance work. For a remote 1:1 call,
separate microphone and system-output capture can establish speaker roles when the
streams are isolated. Speakerphone/in-person audio is mixed; those cases still need
explicit role handling or diarization. See the design's implementation update.
