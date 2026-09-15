# Model mechanics

The built-in catalogue names immutable artifacts by expected byte length and SHA-256
(the upstream Git LFS OID). The downloader only contacts
`https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-<name>.bin`,
following HTTPS redirects. Although the URL uses main, changed upstream bytes fail
verification instead of being trusted automatically.

| Model | Bytes | SHA-256 |
| --- | ---: | --- |
| tiny | 77,691,713 | be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21 |
| base | 147,951,465 | 60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe |
| small | 487,601,967 | 1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b |

All three are multilingual Whisper GGML models licensed MIT. Sources:
[upstream model files](https://huggingface.co/ggerganov/whisper.cpp/tree/main),
[Whisper model license](https://github.com/openai/whisper/blob/main/LICENSE).
Runtime catalogue: `src/model_catalog.rs`.

## Resolution

1. Explicit `--model-path`, if supplied: use that trusted user model, no download.
   Incompatible/invalid weights fail in Whisper. No known checksum is claimed.
2. Embedded base model, for a packaged binary when base is selected.
3. Cache file in `MOM_TEST_MODEL_DIR`, or the OS cache path listed in README.
   Verify the expected size and SHA-256 on every use.
4. If missing and online, download the selected model. If `--offline`, fail with
   the exact model path and preparation command without creating a cache directory.

Imports enter this resolution only after interactive consent, successful decoding,
and local speech detection. Text imports never need a model. Explicit
`models download` is a setup action with no call data, so it needs no per-call
consent prompt.

## Downloads and failures

- HTTPS certificate validation remains enabled. Ureq uses its standard
  `HTTPS_PROXY` / `HTTP_PROXY` / `ALL_PROXY` environment configuration.
  Custom TLS-intercepting enterprise roots are not configured by the CLI; prepare
  the model through an approved download tool and provide `--model-path` if needed.
- Connection timeout: 30 seconds. Body and total request budget: 30 minutes.
- Data streams into a unique temporary file beside the final cache path.
  Never buffer a whole download in RAM. Refuse bytes above the expected size.
- Announce expected disk size and report progress every ~25 MB. Write/flush errors
  include disk-space guidance. The full model must fit; no smaller-model fallback.
- Verify size and hash, flush, then publish without overwriting an existing file.
  Simultaneous importers may download twice, but each verifies the winning file.
- Handled errors clean up partial files. A forcibly killed process can leave a
  hidden temporary file; it is never used as a model. It can be removed when no
  importer is running.
- Resume is deliberately unsupported in v0. Retry starts a fresh download.
- A corrupt cached model is not silently replaced. The error identifies it and
  prints the repair command; remove the damaged file and redownload.

## Bundled release

`scripts/package.py` takes a prepared base model. `build.rs` checks its full size and
SHA-256, then `bundled-model` embeds it into the executable. The native Whisper C++
runtime is statically linked; system runtime libraries remain normal OS requirements.
No model extraction/cache write is needed to load embedded Whisper weights.

The 885 KB Silero VAD model is included in the source tree and embedded in every build,
with its own build-time checksum check and MIT license. Its API requires a filename,
so its public weights are written to a private temporary file and removed on normal
exit. Raw call audio and transcript content never enter that temporary file.
