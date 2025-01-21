use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_package = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = current_package.join("../../").canonicalize()?;
    let proto_path = root.join("proto/agglayer/prover");
    let descriptor_path = current_package
        .join("src/generated")
        .join("agglayer.prover.bin");

    let proof_generation_proto = proto_path.join("v1/proof_generation.proto");

    tonic_build::configure()
        .type_attribute(
            "GenerateProofError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .file_descriptor_set_path(descriptor_path)
        .out_dir("src/generated")
        .compile_protos(&[proof_generation_proto], &[proto_path])?;
    Ok(())
}
