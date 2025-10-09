// Build script to compile protobuf files

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate protobuf code directly to OUT_DIR (default behavior)
    prost_build::compile_protos(&["proto/unixfs.proto"], &["proto/"])?;

    println!("cargo:rerun-if-changed=proto/unixfs.proto");

    Ok(())
}
