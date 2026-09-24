use std::collections::HashMap;

use aggchain_proof_types::{
    imported_bridge_exit::ImportedBridgeExitWithBlockNumber, unclaim::UnclaimWithBlockNumber,
};
use alloy_primitives::U256;

use crate::error::Error;

/// A matched (import, unclaim) pair for the same `global_index`.
#[derive(Debug)]
struct ImportUnclaimPair {
    global_index: U256,
    import_block: u64,
    unclaim_block: u64,
}

/// Validates that a proposer's reduced `new_end_block` does not split any
/// (imported_bridge_exit, unclaim) pair.
///
/// A pair is broken when the import falls within the new block range but its
/// matching unclaim does not, meaning the import would appear as valid in the
/// proof even though it was intended to be cancelled.
pub(crate) fn validate_no_broken_pairs(
    imported_bridge_exits: &[ImportedBridgeExitWithBlockNumber],
    unclaims: &[UnclaimWithBlockNumber],
    new_end_block: u64,
) -> Result<(), Error> {
    for ImportUnclaimPair {
        global_index,
        import_block,
        unclaim_block,
    } in pair_imports_with_unclaims(imported_bridge_exits, unclaims)
    {
        if import_block <= new_end_block && unclaim_block > new_end_block {
            return Err(Error::BrokenImportUnclaimPair {
                global_index,
                import_block,
                unclaim_block,
                new_end_block,
            });
        }
    }

    Ok(())
}

/// Pairs the k-th import for each `global_index` with the k-th unclaim for the
/// same `global_index`, both ordered by `(block_number, log_index)`.
///
/// Excess events on either side are dropped: they correspond to imports that
/// were not cancelled, or to unclaims targeting imports claimed outside the
/// block range covered by these inputs (e.g. in an already-proven range).
fn pair_imports_with_unclaims(
    imported_bridge_exits: &[ImportedBridgeExitWithBlockNumber],
    unclaims: &[UnclaimWithBlockNumber],
) -> Vec<ImportUnclaimPair> {
    #[derive(Default)]
    struct EventLists {
        imports: Vec<(u64, u64)>,
        unclaims: Vec<(u64, u64)>,
    }

    let mut events_by_global_index: HashMap<U256, EventLists> = HashMap::new();

    for import in imported_bridge_exits {
        events_by_global_index
            .entry(import.global_index.into())
            .or_default()
            .imports
            .push((import.block_number, import.log_index));
    }

    for unclaim in unclaims {
        events_by_global_index
            .entry(unclaim.global_index)
            .or_default()
            .unclaims
            .push((unclaim.block_number, unclaim.log_index));
    }

    events_by_global_index
        .into_iter()
        .flat_map(|(global_index, mut events)| {
            events.imports.sort_unstable();
            events.unclaims.sort_unstable();

            events.imports.into_iter().zip(events.unclaims).map(
                move |((import_block, _), (unclaim_block, _))| ImportUnclaimPair {
                    global_index,
                    import_block,
                    unclaim_block,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use agglayer_interop::types::GlobalIndex;

    use super::*;
    use crate::error::Error;

    fn make_import(
        global_index: GlobalIndex,
        block_number: u64,
    ) -> ImportedBridgeExitWithBlockNumber {
        make_import_at(global_index, block_number, 0)
    }

    fn make_import_at(
        global_index: GlobalIndex,
        block_number: u64,
        log_index: u64,
    ) -> ImportedBridgeExitWithBlockNumber {
        use aggchain_proof_types::imported_bridge_exit::BridgeExitHash;
        use agglayer_interop::types::Digest;
        ImportedBridgeExitWithBlockNumber {
            block_number,
            bridge_exit_hash: BridgeExitHash(Digest::default()),
            global_index,
            log_index,
        }
    }

    fn make_unclaim(global_index: GlobalIndex, block_number: u64) -> UnclaimWithBlockNumber {
        make_unclaim_at(global_index, block_number, 0)
    }

    fn make_unclaim_at(
        global_index: GlobalIndex,
        block_number: u64,
        log_index: u64,
    ) -> UnclaimWithBlockNumber {
        UnclaimWithBlockNumber {
            global_index: global_index.into(),
            block_number,
            log_index,
        }
    }

    fn gi(leaf_index: u32) -> GlobalIndex {
        use agglayer_interop::types::NetworkId;
        GlobalIndex::new(NetworkId::new(1), leaf_index)
    }

    // No data at all → always valid.
    #[test]
    fn empty_inputs_ok() {
        assert!(validate_no_broken_pairs(&[], &[], 100).is_ok());
    }

    // Import without unclaim → no pair to break.
    #[test]
    fn import_without_unclaim_ok() {
        let imports = [make_import(gi(1), 50)];
        assert!(validate_no_broken_pairs(&imports, &[], 60).is_ok());
    }

    // Unclaim without matching import → no split.
    #[test]
    fn unclaim_without_import_ok() {
        let unclaims = [make_unclaim(gi(1), 75)];
        assert!(validate_no_broken_pairs(&[], &unclaims, 60).is_ok());
    }

    // Both import and unclaim inside new_end_block → OK.
    #[test]
    fn pair_both_in_range_ok() {
        let imports = [make_import(gi(1), 50)];
        let unclaims = [make_unclaim(gi(1), 75)];
        assert!(validate_no_broken_pairs(&imports, &unclaims, 80).is_ok());
    }

    // Both import and unclaim outside new_end_block → OK (no split).
    #[test]
    fn pair_both_outside_range_ok() {
        let imports = [make_import(gi(1), 100)];
        let unclaims = [make_unclaim(gi(1), 120)];
        assert!(validate_no_broken_pairs(&imports, &unclaims, 80).is_ok());
    }

    // Import inside range, unclaim outside → BROKEN.
    #[test]
    fn pair_import_in_unclaim_out_broken() {
        let imports = [make_import(gi(1), 50)];
        let unclaims = [make_unclaim(gi(1), 75)];
        let result = validate_no_broken_pairs(&imports, &unclaims, 60);
        assert!(
            matches!(
                result,
                Err(Error::BrokenImportUnclaimPair {
                    import_block: 50,
                    unclaim_block: 75,
                    new_end_block: 60,
                    ..
                })
            ),
            "unexpected result: {result:?}"
        );
    }

    // Multiple imports same global_index: smallest block_number is matched first.
    // new_end_block splits only the second pair → BROKEN.
    #[test]
    fn multiple_imports_second_pair_broken() {
        let imports = [make_import(gi(1), 50), make_import(gi(1), 100)];
        let unclaims = [make_unclaim(gi(1), 75), make_unclaim(gi(1), 130)];
        // Pairs: (50,75) and (100,130). new_end_block=120 keeps 100 in range but
        // not 130.
        let result = validate_no_broken_pairs(&imports, &unclaims, 120);
        assert!(
            matches!(
                result,
                Err(Error::BrokenImportUnclaimPair {
                    import_block: 100,
                    unclaim_block: 130,
                    new_end_block: 120,
                    ..
                })
            ),
            "unexpected result: {result:?}"
        );
    }

    // Multiple imports same global_index, new_end_block keeps all pairs intact.
    #[test]
    fn multiple_imports_all_pairs_ok() {
        let imports = [make_import(gi(1), 50), make_import(gi(1), 100)];
        let unclaims = [make_unclaim(gi(1), 75), make_unclaim(gi(1), 130)];
        assert!(validate_no_broken_pairs(&imports, &unclaims, 200).is_ok());
    }

    // Multiple unclaims same global_index: k-th unclaim matches k-th import.
    // new_end_block splits the first pair.
    #[test]
    fn multiple_unclaims_first_pair_broken() {
        // Three imports, three unclaims. new_end_block=55 includes import at 50
        // but not its unclaim at 75.
        let imports = [
            make_import(gi(1), 50),
            make_import(gi(1), 60),
            make_import(gi(1), 100),
        ];
        let unclaims = [
            make_unclaim(gi(1), 75),
            make_unclaim(gi(1), 110),
            make_unclaim(gi(1), 150),
        ];
        let result = validate_no_broken_pairs(&imports, &unclaims, 55);
        assert!(
            matches!(
                result,
                Err(Error::BrokenImportUnclaimPair {
                    import_block: 50,
                    unclaim_block: 75,
                    new_end_block: 55,
                    ..
                })
            ),
            "unexpected result: {result:?}"
        );
    }

    // Independent global_indexes, each with a clean pair → OK.
    #[test]
    fn independent_global_indexes_ok() {
        let imports = [make_import(gi(1), 50), make_import(gi(2), 55)];
        let unclaims = [make_unclaim(gi(1), 70), make_unclaim(gi(2), 80)];
        assert!(validate_no_broken_pairs(&imports, &unclaims, 90).is_ok());
    }

    // Independent global_indexes, one broken.
    #[test]
    fn independent_global_indexes_one_broken() {
        let imports = [make_import(gi(1), 50), make_import(gi(2), 55)];
        let unclaims = [make_unclaim(gi(1), 70), make_unclaim(gi(2), 80)];
        // new_end_block=75: gi(1) pair OK, gi(2) pair broken (55 in range, 80 not).
        let result = validate_no_broken_pairs(&imports, &unclaims, 75);
        assert!(
            matches!(
                result,
                Err(Error::BrokenImportUnclaimPair {
                    unclaim_block: 80,
                    new_end_block: 75,
                    ..
                })
            ),
            "unexpected result: {result:?}"
        );
    }

    // More unclaims than imports: extra unclaim has no matching import → not
    // treated as a split by this function.
    #[test]
    fn more_unclaims_than_imports_not_a_split() {
        let imports = [make_import(gi(1), 50)];
        let unclaims = [make_unclaim(gi(1), 75), make_unclaim(gi(1), 90)];
        // The second unclaim has no matching import at position 1, so we skip it.
        assert!(validate_no_broken_pairs(&imports, &unclaims, 80).is_ok());
    }

    // Two imports in the same block, different log_index: ordering must use
    // (block_number, log_index) so that the k-th import is paired correctly.
    // Import(50, log=2) → Unclaim(60), Import(50, log=5) → Unclaim(120).
    // new_end_block=100: second pair broken (import at 50 in range, unclaim at
    // 120 not).
    #[test]
    fn same_block_imports_log_index_ordering_broken() {
        let imports = [
            make_import_at(gi(1), 50, 5), // second in log order
            make_import_at(gi(1), 50, 2), // first in log order
        ];
        let unclaims = [
            make_unclaim_at(gi(1), 60, 0),  // matches import log=2
            make_unclaim_at(gi(1), 120, 0), // matches import log=5
        ];
        let result = validate_no_broken_pairs(&imports, &unclaims, 100);
        assert!(
            matches!(
                result,
                Err(Error::BrokenImportUnclaimPair {
                    import_block: 50,
                    unclaim_block: 120,
                    new_end_block: 100,
                    ..
                })
            ),
            "unexpected result: {result:?}"
        );
    }

    // Same scenario but new_end_block covers both unclaims → OK.
    #[test]
    fn same_block_imports_log_index_ordering_ok() {
        let imports = [make_import_at(gi(1), 50, 5), make_import_at(gi(1), 50, 2)];
        let unclaims = [
            make_unclaim_at(gi(1), 60, 0),
            make_unclaim_at(gi(1), 120, 0),
        ];
        assert!(validate_no_broken_pairs(&imports, &unclaims, 200).is_ok());
    }
}
