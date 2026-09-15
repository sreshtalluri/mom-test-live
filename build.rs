#[path = "src/model_catalog.rs"]
mod model_catalog;

fn main() {
    use sha2::{Digest, Sha256};
    println!("cargo:rerun-if-changed=assets/ggml-silero-v5.1.2.bin");
    let vad =
        std::fs::read("assets/ggml-silero-v5.1.2.bin").expect("missing bundled speech detector");
    assert_eq!(
        format!("{:x}", Sha256::digest(&vad)),
        "29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf",
        "speech detector checksum mismatch"
    );
    println!("cargo:rerun-if-env-changed=MOM_TEST_BUNDLE_MODEL");
    println!("cargo:rerun-if-changed=src/model_catalog.rs");
    if std::env::var_os("CARGO_FEATURE_BUNDLED_MODEL").is_none() {
        return;
    }
    let path = std::env::var_os("MOM_TEST_BUNDLE_MODEL")
        .expect("bundled-model needs MOM_TEST_BUNDLE_MODEL pointing to ggml-base.bin");
    let path = std::fs::canonicalize(path).expect("couldn't locate bundled base model");
    let mut file = std::fs::File::open(&path).expect("couldn't open bundled base model");
    let model = model_catalog::find("base").unwrap();
    assert_eq!(
        file.metadata().unwrap().len(),
        model.bytes,
        "base model size mismatch"
    );
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut hash).expect("couldn't hash base model");
    assert_eq!(
        format!("{:x}", hash.finalize()),
        model.sha256,
        "base model checksum mismatch"
    );
    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rustc-env=MOM_TEST_BUNDLE_MODEL={}", path.display());
}
