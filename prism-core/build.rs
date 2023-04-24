fn main() {
    prost_build::compile_protos(&["proto/prism.proto"], &["proto/"]).unwrap();
}
