# Founder acceptance: the complete after-call loop

Status: **no real founder results recorded here yet**. Automated speech fixtures
and roleplay are engineering/practice checks, not founder acceptance.

The v0 milestone needs at least three founders to complete the workflow using
their own call and their own project. Record failures and assistance as well as
completion. This trial tests the after-call habit; it does not establish demand
for a live overlay.

## Prepare one session

- Give the founder a native build and link to the
  [Mom Test skills installation](https://github.com/sreshtalluri/mom-test#install).
- Ask them to choose a past customer call or existing transcript they have the
  right to process. They keep the original and resulting transcript locally.
- Have them open their own project in their coding agent and initialize
  `/mom-test-memory init` if needed. Use their actual assumptions, not sample data.
- Ask permission to record a short outcome summary. Do not collect the recording,
  customer names, quotes, API keys, or their private project files for this log.
- Start with the README instructions. Log any intervention needed to continue.

## Observe the workflow

1. From the founder project, run `mom-test-live import <their-file>` and respond to
   the per-call consent prompt. Record format, duration, platform, build version,
   and elapsed time until a transcript is saved (including first-run setup).
2. Have the founder open the saved transcript, compare it with the call, correct
   meaning-changing errors, and add speaker labels if needed. Time this separately.
3. They paste or attach the reviewed transcript in their agent and run
   `/mom-test-debrief`. Check that it uses their real assumption IDs.
4. They review the debrief and run `/mom-test-memory record`. Observe whether it
   writes the interview and updates the evidence links in their project.
5. They run `/mom-test-prep` or `/mom-test-gate` against their actual next decision.
   Record what changed, or explicitly record that nothing changed.

If someone declines consent or a write, respect the decision and log that step as
declined. Do not substitute a demo recording and count the trial as complete.

## Debrief from observed behavior

- Where did you stop or need help? What were you trying to do at that point?
- Show me one transcription correction that changed the meaning, if there was one.
- How did you handle this call's notes before trying this workflow?
- What did the evidence change about your next question or decision, if anything?
- At a later check-in: what happened after your next customer call? Did you use
  this workflow again, and which parts did you actually complete?

Compliments and hypothetical promises to use it again are not adoption evidence.

## Outcome log

Use anonymous trial IDs. Fill rows only from observed sessions, with permission.
`complete` means all five steps above were observed, including review and recording;
assisted completion must say what help was needed. Add new rows for retries.

| trial | date | platform / version | input / duration | import time | review / label time | furthest step | assistance / failure | meaning-changing errors | decision or next-question change | reused after next call? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

## Milestone review

- [ ] Three real founders completed the full loop; assistance and failures logged.
- [ ] Meaning-changing transcription errors were reviewed and actionable bugs filed.
- [ ] Manual labeling effort and the largest workflow drop-off are summarized.
- [ ] A follow-up records actual reuse, or explicitly says reuse is still unknown.
- [ ] Decide the next product step using these results. Completion alone is not
  evidence that live coaching is wanted.

Keep raw transcripts out of this repository. If trials produce evidence about a
product assumption, record it in the relevant founder project's `discovery/` with
their review, using the existing skills.
