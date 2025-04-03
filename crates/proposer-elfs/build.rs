fn main() {
    prover_elf_utils::ElfInfo::new("elf_info.rs")
        // Verification keys for aggregation proof
        .module("aggregation", op_succinct_elfs::AGG_ELF)
        .emit_vkey_hash()
        .finish()
        // Verification keys for range proof
        .module("range", op_succinct_elfs::RANGE_ELF)
        .emit_vkey_hash()
        .emit_vkey_commitment()
        .finish();
}
