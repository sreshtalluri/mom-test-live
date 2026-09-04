//! Speaker-label detection: does this transcript already distinguish speakers?
//!
//! Per TODO 2 (folded into build-now scope, per explicit user direction): rather
//! than special-case F:/C: separately from Zoom/Otter export conventions, use
//! one general heuristic — "does most of the transcript consist of short
//! `<label>: text` lines?" — which covers all three naturally. `F:`/`C:` and
//! `Speaker 2:` and `Priya Sharma:` are all instances of the same shape.
//!
//! v0 never diarizes and never fabricates labels — this only detects whether
//! labels already exist. If they don't, main.rs prints a warning and proceeds.

pub struct LabelCheck {
    pub already_labeled: bool,
    /// Fraction of non-empty lines that matched a `<label>: text` shape, for
    /// anyone debugging a borderline case. Not read by main.rs today — kept
    /// public and computed because it's the field you want the moment a real
    /// transcript sits right at the threshold and you need to know why.
    #[allow(dead_code)]
    pub labeled_line_ratio: f64,
}

const MAX_LABEL_LEN: usize = 40;
/// Majority of non-empty lines must look labeled to call the transcript labeled.
const THRESHOLD: f64 = 0.5;

pub fn check(text: &str) -> LabelCheck {
    let mut non_empty = 0usize;
    let mut labeled = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        non_empty += 1;
        if looks_labeled(trimmed) {
            labeled += 1;
        }
    }

    let ratio = if non_empty == 0 {
        0.0
    } else {
        labeled as f64 / non_empty as f64
    };

    LabelCheck {
        already_labeled: ratio >= THRESHOLD,
        labeled_line_ratio: ratio,
    }
}

/// A line "looks labeled" if it starts with a short label (letters, digits,
/// spaces, `.`, `'`, `-`) immediately followed by `:` and then whitespace or
/// end of line. Covers "F:", "C:", "Speaker 2:", "Priya Sharma:", timestamp-
/// stripped Zoom/Otter export lines, deliberately without a regex dependency
/// (none was reachable in this build environment — see Cargo.toml).
fn looks_labeled(line: &str) -> bool {
    let Some(colon_idx) = line.find(':') else {
        return false;
    };
    if colon_idx == 0 || colon_idx > MAX_LABEL_LEN {
        return false;
    }
    let label = &line[..colon_idx];
    if !label
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '.' || c == '\'' || c == '-')
    {
        return false;
    }
    // Reject labels that are just whitespace, and reject the case where the
    // colon is mid-sentence with no trailing space/EOL (e.g. a URL like
    // "http://example.com" or a ratio "3:1" embedded in a longer sentence).
    if label.trim().is_empty() {
        return false;
    }
    match line[colon_idx + 1..].chars().next() {
        None => true,             // colon at end of line
        Some(c) => c == ' ' || c == '\t',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f_c_convention_detected() {
        let text = "F: What's the hardest part of your week?\nC: Honestly, no-shows.\nF: Tell me more.\n";
        assert!(check(text).already_labeled);
    }

    #[test]
    fn named_speaker_convention_detected() {
        let text = "Priya Sharma: Let's get started.\nJordan Lee: Sounds good.\nPriya Sharma: Great.\n";
        assert!(check(text).already_labeled);
    }

    #[test]
    fn otter_style_speaker_number_detected() {
        let text = "Speaker 1: Hi there.\nSpeaker 2: Hey, thanks for the time.\nSpeaker 1: Of course.\n";
        assert!(check(text).already_labeled);
    }

    #[test]
    fn flat_unlabeled_transcript_not_detected() {
        let text = "So we started talking about the schedule and then things got busy \
                     and I mentioned the deposit fee and they seemed unsure about it.";
        assert!(!check(text).already_labeled);
    }

    #[test]
    fn minority_labeled_lines_not_enough() {
        // Only one of four lines looks labeled — below the 50% threshold.
        let text = "This is a long narrative paragraph with no structure at all here.\n\
                     Another unstructured line follows right after this one too.\n\
                     F: just one aside that happens to have a label on it.\n\
                     And then more unstructured narrative text continues on.\n";
        assert!(!check(text).already_labeled);
    }

    #[test]
    fn empty_transcript_not_labeled() {
        assert!(!check("").already_labeled);
    }

    #[test]
    fn url_in_text_not_mistaken_for_a_label() {
        let text = "Check out http://example.com for more info on this, it was really useful.\n\
                     They also mentioned https://another-example.org as a backup option too.\n";
        assert!(!check(text).already_labeled);
    }

    #[test]
    fn ratio_reported_accurately() {
        let text = "F: one\nC: two\nunlabeled line here\n";
        let result = check(text);
        assert!((result.labeled_line_ratio - (2.0 / 3.0)).abs() < 1e-9);
    }
}
