fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the IPNS protobuf schema
    prost_build::compile_protos(&["proto/ipns.proto"], &["proto/"])?;
    Ok(())
}
