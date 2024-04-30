fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(
            "User",
            "#[derive(Eq, Hash, serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            "FileInfo",
            "#[derive(Eq, serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            "HoldersResponse",
            "#[derive(Eq, serde::Deserialize, serde::Serialize)]",
        )
        .compile(&["market.proto"], &["."])?;

    println!("cargo:rerun-if-changed=market.proto");
    println!("cargo:rerun-if-changed=.");
    Ok(())
}
