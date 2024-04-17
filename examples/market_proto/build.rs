use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_files = &["./proto/market/market.proto"];
    let dirs = &["./proto"];
    tonic_build::configure().compile(proto_files, dirs)?;
    for file in proto_files {
        println!("cargo:rerun-if-changed={}", file);
    }
    Ok(())
}
