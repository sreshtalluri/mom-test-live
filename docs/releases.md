# Verify and distribute v0

## Review builds

[PR #1](https://github.com/sreshtalluri/mom-test-live/pull/1) runs tests and native
packaging for Linux x64, macOS Apple Silicon, macOS Intel, and Windows x64. Open
the **Package offline binaries** check and download the artifact for your platform
after that job passes. Downloading Actions artifacts requires a GitHub login.
These are review builds, not a published release.

Each artifact contains an archive and a `.sha256` checksum file. Check the archive
before extracting it. On macOS use `shasum -a 256 <archive>`; on Linux use
`sha256sum <archive>`; in PowerShell use `Get-FileHash <archive> -Algorithm SHA256`.
Compare the result with the checksum file, then extract the archive.

Use `mom-test-live.exe` on Windows, or `./mom-test-live` on macOS/Linux. Run
`--version` and `models list` first; base should be listed as bundled. Then run the
binary by its absolute path from a founder project, for example:

```sh
/path/to/extracted/mom-test-live import /path/to/call.wav --offline
```

An audio import must still request per-call consent. The bundled base model works
without a first-run download. Review the transcript and follow the printed debrief
and memory-record instructions. The [founder trial protocol](founder-trials.md)
covers product acceptance.

## Maintainer release steps

1. Review both test and packaging matrices on the intended commit. Investigate any
   failed platform before advertising it as supported. Checks must exercise real
   offline speech inference, not only compile successfully.
2. Review the PR and merge it. Confirm `Cargo.toml`'s version matches the intended
   tag. A release from this implementation is `v0.2.0`.
3. Tag the reviewed commit and push that tag. The tag workflow rebuilds native
   archives and creates a **draft** GitHub Release only after packaging succeeds.
4. Check all four archives and checksums are attached, review the release notes,
   and state the unsigned-binary limitation. Check the tag names the reviewed
   commit. Publishing the draft makes the release publicly downloadable.
5. Record the release link in README and STATUS. Keep founder acceptance separate:
   a downloadable release does not establish adoption or justify the live overlay.

For a local Mac/Linux smoke test, after preparing a base model:

```sh
python3 scripts/package.py --model /path/to/ggml-base.bin
MOM_TEST_MODEL_DIR=/path/to/an/empty/cache \
  python3 scripts/e2e.py --binary dist/native/mom-test-live --audio
```

The local test uses the real recorded-speech fixture and transcodes it to five
additional formats if FFmpeg is installed. Also extract the archive, check its
checksum, and repeat against the extracted binary. On Windows, the workflow runs
portable CLI checks and bundled offline inference; interactive console import
still needs manual verification on Windows.
