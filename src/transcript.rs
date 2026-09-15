//! Preserve plain transcripts; extract readable turns from SRT/WebVTT exports.
use std::{fs, path::Path};

pub fn read(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("couldn't read {} as UTF-8 text: {e}", path.display()))?;
    let text = text.trim_start_matches('\u{feff}');
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if extension == "vtt" || extension == "srt" {
        subtitles(text)
    } else {
        Ok(text.to_string())
    }
}

fn subtitles(text: &str) -> Result<String, String> {
    let normalized = text.replace("\r\n", "\n");
    let mut result = String::new();
    for block in normalized.split("\n\n") {
        let block = block.trim();
        if block.starts_with("NOTE") || block.starts_with("STYLE") || block.starts_with("REGION") {
            continue;
        }
        let lines: Vec<_> = block.lines().collect();
        let Some(cue) = lines.iter().position(|line| line.contains(" --> ")) else {
            continue;
        };
        let mut speaker = None;
        for line in &lines[cue + 1..] {
            let line = line.trim();
            let mut plain = String::new();
            let mut remaining = line;
            while let Some(start) = remaining.find('<') {
                plain.push_str(&remaining[..start]);
                let Some(end) = remaining[start..].find('>') else {
                    plain.push_str(&remaining[start..]);
                    remaining = "";
                    break;
                };
                let tag = &remaining[start + 1..start + end];
                if let Some(name) = tag.strip_prefix("v ") {
                    append_turn(&mut result, &mut plain, speaker.as_deref());
                    speaker = Some(name.trim().to_string());
                } else if tag.starts_with("v.") {
                    if let Some((_, name)) = tag.split_once(' ') {
                        append_turn(&mut result, &mut plain, speaker.as_deref());
                        speaker = Some(name.trim().to_string());
                    }
                } else if tag == "/v" {
                    append_turn(&mut result, &mut plain, speaker.as_deref());
                    speaker = None;
                }
                remaining = &remaining[start + end + 1..];
            }
            plain.push_str(remaining);
            append_turn(&mut result, &mut plain, speaker.as_deref());
        }
    }
    if !result.chars().any(char::is_alphanumeric) {
        return Err("subtitle file has no readable cue text; no transcript was saved".into());
    }
    Ok(result)
}

fn append_turn(result: &mut String, plain: &mut String, speaker: Option<&str>) {
    let decoded = plain
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&amp;", "&");
    if !decoded.trim().is_empty() {
        if let Some(name) = speaker {
            result.push_str(name);
            result.push_str(": ");
        }
        result.push_str(decoded.trim());
        result.push('\n');
    }
    plain.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vtt_voice_tags_and_srt_labels_survive_without_cue_metadata() {
        let vtt = "WEBVTT\n\nNOTE private metadata\nignore this\n\n1\n00:00:00.000 --> 00:00:02.000 align:start\n<v Priya>When was the last time?</v>\n\n2\n00:00:02.000 --> 00:00:04.000\n<v Jordan><b>Tuesday</b> &amp; Friday.</v>\n";
        let parsed = subtitles(vtt).unwrap();
        assert_eq!(
            parsed,
            "Priya: When was the last time?\nJordan: Tuesday & Friday.\n"
        );
        assert!(crate::labels::check(&parsed).already_labeled);
        assert_eq!(
            subtitles("1\r\n00:00:00,000 --> 00:00:02,000\r\nF: Hello\r\n").unwrap(),
            "F: Hello\n"
        );
    }

    #[test]
    fn subtitle_metadata_alone_is_not_a_transcript() {
        for text in [
            "WEBVTT",
            "1\n00:00:00.000 --> 00:00:02.000\n",
            "NOTE some text",
        ] {
            assert!(subtitles(text).is_err());
        }
    }

    #[test]
    fn multiple_voices_in_one_cue_are_never_merged_under_the_last_name() {
        let cue = "00:00:00.000 --> 00:00:02.000\n<v F>When?</v><v C>Tuesday.</v>";
        assert_eq!(subtitles(cue).unwrap(), "F: When?\nC: Tuesday.\n");
    }
}
