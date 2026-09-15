//! Hard, unskippable consent gate.
//!
//! Deliberately has no bypass flag. This was raised explicitly during eng review
//! (Codex: kills batch/CI/scripted usage) and reaffirmed as-is: a `--yes` flag IS
//! the escape hatch the design's Constraints section rules out. Tests that need
//! to exercise the confirmed path type the confirmation as a real input, same as
//! a human would — see `prompt_and_check`'s tests below.

use std::io::{IsTerminal, Read, Write};

const CONFIRM_PHRASE: &str = "YES";

/// Must be called before any file content is read. Real entrypoint — checks the
/// real stdin's TTY-ness, which can't be faked in a unit test, so this specific
/// refusal path is an [→E2E] case (see docs' test coverage diagram), not a unit
/// test. `prompt_and_check` below covers the confirmation logic itself.
pub fn require_consent<W: Write>(stdin: &mut dyn Read, stdout: &mut W) -> Result<(), String> {
    if !std::io::stdin().is_terminal() {
        return Err(
            "consent must be given interactively — this looks like a non-interactive \
             session (piped input, no terminal). mom-test-live has no --yes flag: \
             consent can't have an escape hatch."
                .to_string(),
        );
    }
    prompt_and_check(stdin, stdout)
}

/// The confirmation logic itself, decoupled from the real TTY check so it's
/// deterministically unit-testable with an injected reader/writer.
fn prompt_and_check<W: Write>(stdin: &mut dyn Read, stdout: &mut W) -> Result<(), String> {
    writeln!(
        stdout,
        "This will process a call recording or transcript. Recording or transcribing a \
         call may require the other participant's consent depending on your jurisdiction.\n\
         Type {CONFIRM_PHRASE} to confirm you have the right to do this for this specific call:"
    )
    .map_err(|e| e.to_string())?;

    stdout
        .flush()
        .map_err(|e| format!("couldn't show consent prompt: {e}"))?;

    let response = read_line(stdin)?;

    if response.trim() == CONFIRM_PHRASE {
        Ok(())
    } else {
        Err("consent not confirmed — aborting, nothing was read or written".to_string())
    }
}

fn read_line(stdin: &mut dyn Read) -> Result<String, String> {
    let mut buf = String::new();
    let mut byte = [0u8; 1];
    loop {
        match stdin.read(&mut byte) {
            Ok(0) => break, // EOF — treated as "no confirmation given", not an error here
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                if byte[0] != b'\r' {
                    buf.push(byte[0] as char);
                }
            }
            Err(e) => return Err(format!("couldn't read confirmation: {e}")),
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn confirmed_when_exact_phrase_given() {
        let mut input = Cursor::new(b"YES\n".to_vec());
        let mut output = Vec::new();
        assert!(prompt_and_check(&mut input, &mut output).is_ok());
    }

    #[test]
    fn confirmed_with_surrounding_whitespace() {
        let mut input = Cursor::new(b"  YES  \n".to_vec());
        let mut output = Vec::new();
        assert!(prompt_and_check(&mut input, &mut output).is_ok());
    }

    #[test]
    fn declined_on_mismatch() {
        let mut input = Cursor::new(b"yes\n".to_vec()); // wrong case, deliberately strict
        let mut output = Vec::new();
        assert!(prompt_and_check(&mut input, &mut output).is_err());
    }

    #[test]
    fn declined_on_empty_input() {
        let mut input = Cursor::new(b"\n".to_vec());
        let mut output = Vec::new();
        assert!(prompt_and_check(&mut input, &mut output).is_err());
    }

    #[test]
    fn declined_on_eof_ctrl_d() {
        let mut input = Cursor::new(Vec::new()); // immediate EOF, no newline at all
        let mut output = Vec::new();
        assert!(prompt_and_check(&mut input, &mut output).is_err());
    }

    #[test]
    fn prompt_names_the_confirm_phrase() {
        let mut input = Cursor::new(b"YES\n".to_vec());
        let mut output = Vec::new();
        let _ = prompt_and_check(&mut input, &mut output);
        let printed = String::from_utf8(output).unwrap();
        assert!(printed.contains(CONFIRM_PHRASE));
    }
}
