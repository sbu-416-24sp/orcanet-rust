// use serde::{Deserialize, Serialize};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "market/market.proto";

    tonic_build::configure()
        .type_attribute("User", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(&[proto], &["."])?;

    Ok(())
}
