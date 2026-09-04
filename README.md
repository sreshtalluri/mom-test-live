# mom-test-live

A live conversation companion for [The Mom Test](https://github.com/sreshtalluri/mom-test).
Not a meeting copilot that tells you what to say to your customer — a **conversation
contamination firewall** that catches **your own** pitching, leading questions, and
premature-solution talk before it contaminates the interview.

Every existing "invisible AI overlay" tool in this category (Cluely and its clones)
answers "what should I say to them." This one does the opposite: the founder is the one
being coached, not the customer.

Status: **pre-v0**. Design approved, nothing built yet. See
[the full design doc](https://github.com/sreshtalluri/mom-test/blob/main/docs/designs/live-coach-companion.md)
in the parent skills repo for the complete rationale, three rounds of adversarial
review, and the real cost research behind the architecture.

## The plan

**v0 — after-call transcript prep (CLI, in progress).** Drop in a call recording or
transcript. It runs a hard, unskippable consent check, transcribes locally with
whisper.cpp if needed, and hands you a clean transcript file plus the exact two
commands to run `/mom-test-debrief` and `/mom-test-memory record` yourself in your own
coding agent. No LLM call, no diarization, no `discovery/` write inside this tool at
all — it hands off to the real skills, which have real file access this CLI doesn't.
$0 cost, no API key needed.

**v1 — the live overlay (later, once v0 has real usage).** A quiet, invisible-until-
triggered overlay during the actual call: local whisper.cpp + Diart for real-time
transcription and diarization, a founder-violation classifier (Groq, bring-your-own-key,
~$0.005/call), forked from [OpenCluely](https://github.com/TechyCSR/OpenCluely) for the
stealth-overlay plumbing. Surfaces exactly one thing when a rule trips — `Signal / Why /
Say next / Don't say` — then gets out of the way again.

## Why v0 before v1

Building the risky, expensive part (real-time diarization, a classifier that doesn't
exist yet) before anyone's used the cheap part would be exactly the mistake the book
warns against, applied to this project's own roadmap. v0 ships first, gets real
founders using it, and v1 gets built informed by that instead of guesses.

## Contributing

Not accepting PRs yet — v0 doesn't exist as code. Watch this repo or the
[parent skills repo](https://github.com/sreshtalluri/mom-test) for updates.

## License

MIT.
