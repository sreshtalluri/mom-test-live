# Recorded-speech fixture

`jfk.wav` is the 11-second JFK inaugural-address excerpt distributed in
[whisper.cpp's samples](https://github.com/ggml-org/whisper.cpp/tree/master/samples).
Downloaded 2026-09-07 from
https://raw.githubusercontent.com/ggml-org/whisper.cpp/master/samples/jfk.wav.

SHA-256: `59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e`.

This is a real human recording of a public US presidential speech, not a private
customer interview or synthesized speech. Expected words include “ask not,”
“your country,” and “do for.” Upstream code/sample distribution is MIT; the
underlying US government speech is public domain.

The end-to-end suite transcodes it with FFmpeg to test common codecs, stereo
downmixing, and 44.1 kHz resampling. FFmpeg is test tooling only, never required by
the shipped binary. A real noisy customer call remains a separate acceptance test.
