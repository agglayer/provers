pub mod aggregation {
    pub use op_succinct_elfs::AGG_ELF as ELF;
    pub use proposer_vkeys_raw::aggregation::*;
}

pub mod range {
    pub use op_succinct_elfs::RANGE_ELF as ELF;
    pub use proposer_vkeys_raw::range::*;
}
