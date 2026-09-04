//! Hand-off message: the exact next step, printed verbatim.
//!
//! Per the round-3 design fix and plan-eng-review's clarity finding: this is
//! "paste/attach", not a path argument, because neither `mom-test-debrief` nor
//! `mom-test-memory` documents a file-path invocation syntax — they're written
//! for pasted or attached notes. This tool never calls an LLM and never writes
//! to `discovery/` itself; the founder's own coding agent does both, with real
//! file access this CLI doesn't have.

use std::path::Path;

pub fn print_instructions(transcript_path: &Path) {
    println!(
        "\nSaved: {}\n\n\
         Next step — in your coding agent (the one with the mom-test skills installed):\n\
         \n  1. Paste or attach the contents of {} and say: \"/mom-test-debrief\"\n\
         \n  2. Once it finishes, say: \"/mom-test-memory record\"\n",
        transcript_path.display(),
        transcript_path.display(),
    );
}

#[cfg(test)]
mod tests {
    // print_instructions writes directly to stdout, which isn't worth mocking
    // for a two-line format string — this is the [→E2E] case in the coverage
    // diagram (verify by running the binary and checking real stdout).
    // Nothing here is untestable in principle; it's just not unit-test shaped.
}
