//! Immutable upstream GGML artifacts (MIT). Hashes are the Hugging Face LFS OIDs.
//! Source: https://huggingface.co/ggerganov/whisper.cpp/tree/main

pub struct Model {
    pub name: &'static str,
    pub sha256: &'static str,
    pub bytes: u64,
}

pub const MODELS: &[Model] = &[
    Model {
        name: "tiny",
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        bytes: 77_691_713,
    },
    Model {
        name: "base",
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        bytes: 147_951_465,
    },
    Model {
        name: "small",
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        bytes: 487_601_967,
    },
];

pub fn find(name: &str) -> Result<&'static Model, String> {
    MODELS.iter().find(|m| m.name == name).ok_or_else(|| {
        format!("unknown model {name:?}; choose tiny, base, or small (no automatic fallback)")
    })
}
