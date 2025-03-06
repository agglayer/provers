use std::fs;
use std::path::Path;

use semver::Version;
use toml::Value;

fn main() {
    let cargo_toml_path = Path::new("../aggchain-proof-program/Cargo.toml");
    let cargo_toml = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");
    let parsed_toml: Value = cargo_toml.parse().expect("Failed to parse Cargo.toml");

    let version: Version = parsed_toml
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| {
            v.as_str()
                .map(Version::parse)
                .transpose()
                .expect("Unable to extract version")
        })
        .expect("Unable to extract version");

    let major_version = version.major.to_string();

    let out_dir = "./src/";
    let dest_path = Path::new(&out_dir).join("version.rs");
    fs::write(
        &dest_path,
        format!(
            "pub const AGGCHAIN_PROOF_PROGRAM_VERSION: u16 = {};\n",
            major_version
        ),
    )
    .expect("Failed to write aggchain_proof_program_version.rs");

    println!("cargo:rerun-if-changed=Cargo.toml");
}
