#!/usr/bin/env python3
"""Build and package a native offline binary. Uses only the Python standard library.

python3 scripts/package.py --model /path/to/ggml-base.bin
This builds for the native host, never cross-compiles or publishes a release.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--model", type=Path, required=True)
args = parser.parse_args()
model = args.model.resolve()
env = os.environ.copy()
env["MOM_TEST_BUNDLE_MODEL"] = str(model)
# No instructions specific to the build machine's CPU in distributed binaries.
for key in ["GGML_NATIVE", "GGML_BMI2", "GGML_AVX", "GGML_AVX2", "GGML_FMA", "GGML_F16C"]:
    env[key] = "OFF"
host = next(line.split(": ", 1)[1] for line in subprocess.check_output(
    ["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
env["CARGO_TARGET_DIR"] = str(ROOT / "target" / "distribution")
subprocess.run(["cargo", "build", "--release", "--locked", "--target", host,
                "--features", "bundled-model"], cwd=ROOT, env=env, check=True)
binary_name = "mom-test-live.exe" if os.name == "nt" else "mom-test-live"
binary = ROOT / "target" / "distribution" / host / "release" / binary_name
# Explicit --target (e.g. user configuration) is not silently packaged as native.
subprocess.run([str(binary), "--version"], check=True)
dist = ROOT / "dist"
dist.mkdir(exist_ok=True)
(dist / "native").mkdir(exist_ok=True)
shutil.copy2(binary, dist / "native" / binary_name)
archive_name = "mom-test-live-" + host
with tempfile.TemporaryDirectory(prefix="mom-test-package-") as temp:
    package = Path(temp) / archive_name
    package.mkdir()
    shutil.copy2(binary, package / binary_name)
    for name in ["README.md", "LICENSE", "THIRD_PARTY.md", "STATUS.md"]:
        shutil.copy2(ROOT / name, package / name)
    shutil.copytree(ROOT / "docs", package / "docs")
    shutil.copytree(ROOT / "licenses", package / "licenses")
    # A native build only downloads dependencies for its target. Unfiltered
    # metadata tries to fetch other platforms' crates even with --offline (for
    # example anstyle-wincon on macOS), breaking packaging on a fresh runner.
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--offline",
         "--filter-platform", host, "--features", "bundled-model"], cwd=ROOT, env=env))
    inventory = ["# Rust dependency sources and licenses", ""]
    for dependency in metadata["packages"]:
        name, version = dependency["name"], dependency["version"]
        if name == "mom-test-live":
            continue
        inventory.append(f"- {name} {version}: {dependency.get('license')}; "
                         f"https://crates.io/crates/{name}/{version}")
        crate = Path(dependency["manifest_path"]).parent
        notices = [p for p in crate.iterdir() if p.is_file() and
                   p.name.lower().startswith(("license", "licence", "copying", "notice", "unlicense"))]
        if notices:
            target = package / "licenses" / f"{name}-{version}"
            target.mkdir()
            for notice in notices:
                shutil.copy2(notice, target / notice.name)
    (package / "licenses" / "DEPENDENCIES.md").write_text("\n".join(inventory) + "\n")
    shutil.copy2(ROOT / "assets" / "SILERO-LICENSE", package / "licenses" / "SILERO-LICENSE")
    archive = Path(shutil.make_archive(str(dist / archive_name),
                   "zip" if os.name == "nt" else "gztar", temp, archive_name))
digest = hashlib.sha256()
with archive.open("rb") as file:
    for chunk in iter(lambda: file.read(1024 * 1024), b""):
        digest.update(chunk)
archive.with_name(archive.name + ".sha256").write_text(digest.hexdigest() + "  " + archive.name + "\n")
print("Packaged:", archive)
