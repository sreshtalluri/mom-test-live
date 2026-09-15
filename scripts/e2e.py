#!/usr/bin/env python3
"""Exercise the real CLI through a PTY. Python standard library only, macOS/Linux.

python3 scripts/e2e.py --binary target/debug/mom-test-live
python3 scripts/e2e.py --binary target/release/mom-test-live --audio
Models are prepared separately; every import in this suite is strictly offline.
"""
import argparse
import concurrent.futures
import hashlib
import math
import os
from pathlib import Path
import pty
import select
import shutil
import struct
import subprocess
import tempfile
import time
import unittest
import wave

ROOT = Path(__file__).resolve().parents[1]
PARSER = argparse.ArgumentParser()
PARSER.add_argument("--binary", type=Path, required=True)
PARSER.add_argument("--audio", action="store_true")
ARGS = PARSER.parse_args()
BINARY = ARGS.binary.resolve()


def invoke(directory, source, answer="YES", options=(), env=None, timeout=180):
    master, slave = pty.openpty()
    process = subprocess.Popen(
        [str(BINARY), "import", str(source), *options],
        cwd=directory, stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        env=env or os.environ.copy(),
    )
    os.close(slave)
    output = b""
    deadline = time.monotonic() + timeout
    try:
        # Wait for the flushed prompt before supplying confirmation. This catches
        # consent after file/model processing, as well as a hidden buffered prompt.
        while b"Type YES" not in output:
            if time.monotonic() > deadline:
                raise AssertionError("consent prompt timed out: " + output.decode(errors="replace"))
            if select.select([process.stdout], [], [], 0.1)[0]:
                data = os.read(process.stdout.fileno(), 65536)
                if not data:
                    raise AssertionError("exited before consent: " + output.decode(errors="replace"))
                output += data
        if b"Decoding" in output or b"Loading" in output or b"Downloading" in output:
            raise AssertionError("processing began before consent")
        os.write(master, (answer + "\n").encode())
        rest, _ = process.communicate(timeout=max(1, deadline - time.monotonic()))
        return process.returncode, (output + rest).decode(errors="replace")
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()
        os.close(master)


def write_wav(path, samples, rate=16000):
    with wave.open(str(path), "wb") as wav:
        wav.setparams((1, 2, rate, 0, "NONE", "not compressed"))
        wav.writeframes(struct.pack("<" + "h" * len(samples), *samples))


class ImportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mom-test-e2e-")
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.env = os.environ.copy()
        self.env["MOM_TEST_MODEL_DIR"] = str(self.directory / "models")

    def files(self):
        return list((self.directory / "mom-test-live").glob("*.txt"))

    def call(self, source, answer="YES", options=("--offline",)):
        return invoke(self.directory, source, answer, options, self.env)

    def test_labeled_transcript_handoff_and_no_discovery_mutation(self):
        source = self.directory / "call.txt"
        text = "F: When did that last happen?\nC: On Tuesday, it cost us two hours.\n"
        source.write_text(text)
        discovery = self.directory / "discovery"
        discovery.mkdir()
        (discovery / "assumptions.md").write_text("A1 stays unchanged")
        code, output = self.call(source)
        self.assertEqual(code, 0, output)
        self.assertNotIn("warning:", output)
        self.assertIn("/mom-test-debrief", output)
        self.assertIn("/mom-test-memory record", output)
        self.assertEqual(self.files()[0].read_text(), text)
        self.assertEqual((discovery / "assumptions.md").read_text(), "A1 stays unchanged")
        self.assertFalse((self.directory / "models").exists())
        subprocess.run(["git", "init", "-q", str(self.directory)], check=True)
        ignored = subprocess.run(["git", "check-ignore", str(self.files()[0])],
                                 cwd=self.directory, capture_output=True)
        self.assertEqual(ignored.returncode, 0)

    def test_decline_happens_before_classifying_or_downloading(self):
        source = self.directory / "broken.wav"
        source.write_text("not audio")
        code, output = self.call(source, answer="no", options=())
        self.assertNotEqual(code, 0)
        self.assertIn("consent not confirmed", output)
        self.assertNotIn("contents look", output)
        self.assertFalse((self.directory / "models").exists())
        self.assertFalse((self.directory / "mom-test-live").exists())

    def test_caption_exports_preserve_real_speaker_turns(self):
        source = self.directory / "call.vtt"
        source.write_text("WEBVTT\n\n00:00:00.000 --> 00:00:03.000\n<v F>When?</v><v C>Tuesday.</v>\n")
        code, output = self.call(source)
        self.assertEqual(code, 0, output)
        self.assertNotIn("warning:", output)
        self.assertEqual(self.files()[0].read_text(), "F: When?\nC: Tuesday.\n")

    def test_unlabeled_text_warns(self):
        source = self.directory / "call.txt"
        source.write_text("Last Tuesday we spent two hours fixing it.")
        code, output = self.call(source)
        self.assertEqual(code, 0, output)
        self.assertIn("warning:", output)
        self.assertIn("F: / C:", output)

    def test_empty_binary_and_corrupt_files_never_write_output(self):
        cases = [("empty.txt", b" \n\t"), ("binary.txt", b"\x00\xff"),
                 ("bad.wav", b"RIFF\x00\x00\x00\x00WAVE"), ("bad.docx", b"whatever"),
                 ("invalid.txt", b"\xff\xfe\xff")]
        for name, contents in cases:
            with self.subTest(name=name):
                source = self.directory / name
                source.write_bytes(contents)
                code, output = self.call(source)
                self.assertNotEqual(code, 0, output)
                self.assertFalse(self.files())

    def test_silence_and_dc_never_download_a_model_or_save_output(self):
        for amplitude in [0, 4000]:
            source = self.directory / "silence.wav"
            write_wav(source, [amplitude] * 32000)
            code, output = self.call(source, options=())
            self.assertNotEqual(code, 0)
            self.assertIn("no speech detected", output)
            self.assertFalse((self.directory / "models").exists())
            self.assertFalse(self.files())

    def test_missing_and_corrupt_explicit_model_fail_clearly(self):
        source = ROOT / "tests/fixtures/jfk.wav"
        for contents in [None, b"invalid" * 300]:
            model = self.directory / "bad.bin"
            if contents is not None:
                model.write_bytes(contents)
            code, output = self.call(source, options=("--offline", "--model-path", str(model)))
            self.assertNotEqual(code, 0, output)
            self.assertIn("model", output)
            self.assertFalse(self.files())

    def test_missing_managed_model_is_strictly_offline(self):
        # small is never embedded, including in release builds.
        code, output = self.call(ROOT / "tests/fixtures/jfk.wav",
                                 options=("--offline", "--model", "small"))
        self.assertNotEqual(code, 0, output)
        self.assertIn("--offline forbids downloads", output)
        self.assertFalse((self.directory / "models").exists())
        self.assertFalse(self.files())

    def test_concurrent_imports_do_not_overwrite_each_other(self):
        source = self.directory / "call.txt"
        source.write_text("F: hello\nC: hi\n")
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(lambda _: self.call(source), range(4)))
        for code, output in results:
            self.assertEqual(code, 0, output)
        self.assertEqual(len(self.files()), 4)
        self.assertEqual(len({p.name for p in self.files()}), 4)

    @unittest.skipUnless(ARGS.audio, "pass --audio with a prepared base model")
    def test_real_recorded_speech_and_transcoded_formats(self):
        self.assertEqual(hashlib.sha256((ROOT / "tests/fixtures/jfk.wav").read_bytes()).hexdigest(),
                         "59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e")
        # Use the caller's prepared cache, or the embedded base model.
        self.env = os.environ.copy()
        source = ROOT / "tests/fixtures/jfk.wav"
        formats = [(source, "wav")]
        if shutil.which("ffmpeg"):
            for extension, codec in [("mp3", "libmp3lame"), ("m4a", "aac"),
                                     ("flac", "flac"), ("ogg", "vorbis"),
                                     ("mp4", "aac")]:
                path = self.directory / ("speech." + extension)
                subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", str(source),
                                "-ar", "44100", "-ac", "2", "-c:a", codec, "-strict", "-2", str(path)],
                               check=True)
                formats.append((path, extension))
        for path, extension in formats:
            with self.subTest(format=extension):
                before = set(self.files())
                code, output = self.call(path, options=("--offline", "--language", "en"))
                self.assertEqual(code, 0, output)
                self.assertIn("warning:", output)
                transcript = (set(self.files()) - before).pop().read_text().lower()
                for phrase in ["ask not", "your country", "do for"]:
                    self.assertIn(phrase, transcript)

    @unittest.skipUnless(ARGS.audio, "pass --audio with a prepared base model")
    def test_non_speech_tone_does_not_become_evidence(self):
        self.env = os.environ.copy()
        source = self.directory / "tone.wav"
        write_wav(source, [int(5000 * math.sin(2 * math.pi * 440 * i / 16000)) for i in range(160000)])
        code, output = self.call(source, options=("--offline", "--language", "en"))
        self.assertNotEqual(code, 0, output)
        self.assertIn("no speech detected", output)
        self.assertFalse(self.files())


if __name__ == "__main__":
    unittest.main(argv=[__file__], verbosity=2)
