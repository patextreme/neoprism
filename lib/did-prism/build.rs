fn main() {
    prost_build::compile_protos(
        &[
            "proto/prism.proto",
            "proto/prism-ssi.proto",
            "proto/prism-storage.proto",
            "proto/prism-version.proto",
        ],
        &["proto/"],
    )
    .unwrap();
}
