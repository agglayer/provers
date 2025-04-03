use std::{env, fs, io::Write, path::Path};

use sp1_sdk::{CpuProver, HashableKey, Prover as _, SP1VerifyingKey};

/// Build time tool to emit information about a zkvm ELF.
pub struct ElfInfo {
    /// Lazily loaded SP1 prover client
    prover: Option<CpuProver>,

    /// Target file
    output: fs::File,
}

impl ElfInfo {
    pub fn new(file_name: impl AsRef<Path>) -> Self {
        println!("cargo::rerun-if-changed=build.rs");

        let prover = None;

        let dir = env::var_os("OUT_DIR").expect("output directory");
        let path = Path::new(&dir).join(file_name);
        let output = fs::File::create(path).expect("elf info output file");

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

    fn prover(&mut self) -> &CpuProver {
        self.prover.get_or_insert_with(|| CpuProver::new())
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
        self.output().write(b"}\n").unwrap();
        self.context
    }

    fn output(&self) -> &fs::File {
        &self.context.output
    }
}

impl<ElfBytes: AsRef<[u8]>> Emitter<ElfBytes> {
    /// Emit an attribute to be added to the next item.
    pub fn emit_attr(self, attr: &str) -> Self {
        writeln!(self.output(), "    #[{attr}]").unwrap();
        self
    }

    /// Emit bincode-encoded vkey for given proof.
    pub fn emit_vkey(self) -> Self {
        todo!("emit vkey encoded with bincode")
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
            let (_pkey, vkey) = self.context.prover().setup(self.elf.as_ref());
            vkey
        })
    }
}
