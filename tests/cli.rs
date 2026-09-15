//! Portable process-level checks; interactive TTY scenarios live in scripts/e2e.py.
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mom-test-live"))
}

#[test]
fn help_version_and_model_list_work_without_consent_or_network() {
    for args in [vec!["--help"], vec!["--version"], vec!["models", "list"]] {
        let dir = tempfile::tempdir().unwrap();
        let output = bin()
            .args(args)
            .env("MOM_TEST_MODEL_DIR", dir.path().join("cache"))
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

#[test]
fn piping_yes_never_bypasses_consent() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("call.txt");
    fs::write(&source, "F: Hello\nC: Hi\n").unwrap();
    let mut child = bin()
        .args(["import"])
        .arg(&source)
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(b"YES\n");
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("interactively"));
    assert!(!dir.path().join("mom-test-live").exists());
}

#[test]
fn rejects_directories_and_missing_inputs_before_consent() {
    let dir = tempfile::tempdir().unwrap();
    for path in [dir.path().to_path_buf(), dir.path().join("missing.wav")] {
        let output = bin().arg("import").arg(path).output().unwrap();
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("Type YES"));
    }
}
