use protobuf_codegen::Codegen;

fn main() {
    Codegen::new()
        .include("proto")
        .inputs([
            "proto/prism.proto",
            "proto/prism-ssi.proto",
            "proto/prism-storage.proto",
            "proto/prism-version.proto",
        ])
        .cargo_out_dir("generated")
        .run_from_script();
}
