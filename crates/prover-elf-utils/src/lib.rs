/// Build time tool to emit information about an zkvm ELF.
pub mod elf_info {
    use std::{env, fs, io::Write, path::Path};

    use sp1_sdk::{CpuProver, HashableKey, Prover as _, SP1VerifyingKey};

    pub struct Context {
        /// Lazily loaded SP1 prover client
        prover: Option<CpuProver>,

        /// Target file
        output: fs::File,
    }

    impl Context {
        pub fn new(file_name: impl AsRef<Path>) -> Self {
            println!("cargo::rerun-if-changed=build.rs");

            let prover = None;

            let dir = env::var_os("OUT_DIR").expect("output directory");
            let path = Path::new(&dir).join(file_name);
            let output = fs::File::create(path).expect("elf info output file");

            Self { prover, output }
        }

        pub fn with_elf_bytes<ElfBytes: AsRef<[u8]>>(
            self,
            name: &'static str,
            elf: ElfBytes,
        ) -> Emitter<ElfBytes> {
            Emitter {
                context: self,
                name,
                elf,
                vkey: None,
            }
        }

        fn prover(&mut self) -> &CpuProver {
            self.prover.get_or_insert_with(|| CpuProver::new())
        }
    }

    pub struct Emitter<ElfBytes> {
        context: Context,
        name: &'static str,
        elf: ElfBytes,
        vkey: Option<SP1VerifyingKey>,
    }

    impl<ElfBytes: AsRef<[u8]>> Emitter<ElfBytes> {
        pub fn emit_elf_bytes(self) -> Self {
            let name = self.name;
            let elf = self.elf.as_ref();
            write!(&self.context.output, "const {name}_ELF: &[u8] = {elf:?};").unwrap();
            self
        }

        pub fn emit_vkey(self) -> Self {
            todo!("emit vkey");
            self
        }

        pub fn emit_vkey_hash(mut self) -> Self {
            let vkey_hash = self.vkey().hash_u32();
            let name = self.name;
            write!(
                &self.context.output,
                "const {name}_VKEY_HASH: &[u32; 8] = {vkey_hash:?};"
            )
            .unwrap();

            self
        }

        pub fn finish(self) -> Context {
            self.context
        }

        fn vkey(&mut self) -> &SP1VerifyingKey {
            self.vkey.get_or_insert_with(|| {
                let (_pkey, vkey) = self.context.prover().setup(self.elf.as_ref());
                vkey
            })
        }
    }
}
