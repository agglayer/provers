use std::{env, fs, io::Write, path::Path};

use bincode::Options;
use sp1_prover::{HashableKey, SP1Prover, SP1VerifyingKey};

pub fn bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

/// Build time tool to emit information about a zkvm ELF.
pub struct ElfInfo {
    /// Lazily loaded SP1 prover client.
    prover: Option<SP1Prover>,

    /// Target file.
    output: fs::File,
}

impl ElfInfo {
    /// Write the modules into a file relative to the source directory.
    pub fn writing_to_src(src_path: impl AsRef<Path>) -> Self {
        let dir = env::var_os("CARGO_MANIFEST_DIR").expect("output directory");
        let path = Path::new(&dir).join("src").join(src_path);
        Self::writing_to_custom(path)
    }

    /// Write the modules into a file, specifying the full path.
    pub fn writing_to_custom(path: impl AsRef<Path>) -> Self {
        println!("cargo::rerun-if-changed=build.rs");

        let prover = None;

        //let dir = env::var_os("OUT_DIR").expect("output directory");
        //let path = Path::new(&dir).join(file_name);
        let output = fs::File::create(path).expect("elf info output file");

        writeln!(&output, "// AUTO-GENERATED FILE. DO NOT EDIT.").unwrap();

        Self { prover, output }
    }

    /// Emit module corresponding to given ELF binary.
    pub fn module<EB>(self, module_name: &str, elf_bytes: EB) -> Emitter<EB> {
        Emitter::new(self, module_name, elf_bytes)
    }

    /// Like [Self::module] but the ELF is taken from a file.
    pub fn module_from_file(
        self,
        module_name: &str,
        elf_path: impl AsRef<Path>,
    ) -> Emitter<Box<[u8]>> {
        let path_string = elf_path.as_ref().to_string_lossy();
        println!("cargo::rerun-if-changed={path_string}");

        let elf_bytes = fs::read(elf_path).unwrap().into_boxed_slice();
        self.module(module_name, elf_bytes)
    }

    fn prover(&mut self) -> &SP1Prover {
        self.prover.get_or_insert_with(SP1Prover::new)
    }
}

/// Takes care of emitting code for one proof binary.
#[must_use = "Please finalize the sequence with a .finish() call"]
pub struct Emitter<ElfBytes> {
    context: ElfInfo,
    elf: ElfBytes,
    vkey: Option<SP1VerifyingKey>,
}

impl<ElfBytes> Emitter<ElfBytes> {
    fn new(context: ElfInfo, name: &str, elf: ElfBytes) -> Self {
        writeln!(&context.output, "pub mod {name} {{").unwrap();
        let vkey = None;
        Emitter { context, elf, vkey }
    }

    pub fn finish(self) -> ElfInfo {
        self.output().write_all(b"}\n").unwrap();
        self.context
    }

    fn output(&self) -> &fs::File {
        &self.context.output
    }
}

impl<ElfBytes: AsRef<[u8]>> Emitter<ElfBytes> {
    /// Emit bincode-encoded vkey for given proof.
    pub fn emit_vkey_bytes(mut self) -> Self {
        let bytes = bincode_options().serialize(self.vkey()).unwrap();
        writeln!(
            self.output(),
            "    pub const VKEY_BYTES: &[u8] = &{bytes:?};"
        )
        .unwrap();
        self
    }

    /// Emit vkey hash for given proof.
    pub fn emit_vkey_hash(mut self) -> Self {
        let hash = self.vkey().hash_u32();
        writeln!(
            self.output(),
            "    pub const VKEY_HASH: [u32; 8] = {hash:?};"
        )
        .unwrap();
        self
    }

    /// Emit vkey hash commitment.
    pub fn emit_vkey_commitment(mut self) -> Self {
        let commitment = self.vkey().hash_bytes();
        writeln!(
            self.output(),
            "    pub const VKEY_COMMITMENT: [u8; 32] = {commitment:?};"
        )
        .unwrap();
        self
    }

    fn vkey(&mut self) -> &SP1VerifyingKey {
        self.vkey.get_or_insert_with(|| {
            let (_sp1_pkey, _stark_pkey, _program, vkey) =
                self.context.prover().setup(self.elf.as_ref());
            vkey
        })
    }
}
