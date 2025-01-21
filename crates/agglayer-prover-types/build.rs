use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = PathBuf::from("src/generated").join("agglayer.prover.bin");

    tonic_build::configure()
        .type_attribute(
            "ProofGenerationError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .file_descriptor_set_path(descriptor_path)
        .out_dir("src/generated")
        .compile_protos(&["proto/v1/proof_generation.proto"], &["proto"])?;
    Ok(())
}
