mod internal {
    include!(concat!(env!("OUT_DIR"), "/elf_info.rs"));
}

pub mod aggregation {
    pub use super::internal::aggregation::VKEY_HASH;
    pub use op_succinct_elfs::AGG_ELF as ELF;
}

pub mod range {
    pub use super::internal::range::VKEY_HASH;
    pub use op_succinct_elfs::RANGE_ELF as ELF;
}
