fn main() {
    prover_elf_utils::ElfInfo::writing_to_src("generated.rs")
        // Verification keys for aggregation proof
        .module("aggregation", op_succinct_elfs::AGGREGATION_ELF)
        .emit_vkey_bytes()
        .emit_vkey_hash()
        .finish()
        // Verification keys for range proof
        .module("range", op_succinct_elfs::RANGE_ELF_EMBEDDED)
        .emit_vkey_bytes()
        .emit_vkey_hash()
        .emit_vkey_commitment()
        .finish();
}
