# Rust coding guidelines

* Prioritize code correctness and clarity. Speed and efficiency are secondary priorities unless otherwise specified.
* Do not write comments that summarize the code. Comments should only be written in order to explain "why" the code is written in some way in the case there is a reason that is tricky / non-obvious.
* Prefer implementing functionality in existing files unless it is a new logical component. Avoid creating many small files.
* Avoid using functions that panic like `unwrap()`, instead use mechanisms like `?` to propagate errors.
* Be careful with operations like indexing which may panic if the indexes are out of bounds.
* Never silently discard errors with `let _ =` on fallible operations. Always handle errors appropriately:
  - Propagate errors with `?` when the calling function should handle them
  - Use `.log_err()` or similar when you need to ignore errors but want visibility
  - Use explicit error handling with `match` or `if let Err(...)` when you need custom logic
  - Example: avoid `let _ = client.request(...).await;` - use `client.request(...).await?;` instead
* When implementing async operations that may fail, ensure errors propagate to the UI layer so users get meaningful feedback.

* Avoid creative additions unless explicitly requested
* Use full words for variable names (no abbreviations like "q" for "queue")
* Use variable shadowing to scope clones in async contexts for clarity, minimizing the lifetime of borrowed references.
  Example:
  ```rust
  executor.spawn({
      let task_ran = task_ran.clone();
      async move {
          *task_ran.borrow_mut() = true;
      }
  });
  ```

## Project Overview

Agglayer provers is a Rust-based collection of zero-knowledge provers for the Agglayer (Aggregation Layer), which provides interoperability among heterogeneous blockchains. The project uses SP1 (Succinct's proving system) for generating zero-knowledge proofs.
## Architecture

The project is structured as a Rust workspace with multiple interconnected crates:

### Core Proof Systems
- **aggchain-proof-***: Core aggchain proof (full execution proof for the l2 chain) system components
  - `aggchain-proof-builder`: Builds and manages ELF binaries for proofs
  - `aggchain-proof-core`: Core proof logic and types
  - `aggchain-proof-service`: Service layer for proof generation
  - `aggchain-proof-contracts`: L1/L2 smart contract integration
  - `aggchain-proof-types`: Type definitions

### Aggkit Prover
- **aggkit-prover**: Aggregation kit prover implementation
- **aggkit-prover-config**: Configuration for aggkit
- **aggkit-prover-types**: Type definitions

### Proposer System
- **proposer-***: Components for the proposer system
  - `proposer-client`: Client for proposer interactions
  - `proposer-service`: Service layer
  - `proposer-elfs`: ELF management for proposers

### Shared Infrastructure
- **prover-***: Shared prover utilities and infrastructure
  - `prover-engine`: Core proving engine
  - `prover-executor`: Execution layer
  - `prover-alloy`: Ethereum integration via Alloy
  - `prover-config`: Configuration utilities
  - `prover-elf-utils`: ELF file utilities
  - `prover-logger`: Logging infrastructure
  - `prover-utils`: General utilities

## Development Commands

### Building
```bash
# Standard build
cargo build

# Build with automatic ELF rebuild (for proof modifications)
export AGGLAYER_ELF_BUILD=update
cargo build

cargo build --quiet
cargo make ap-elf
```

### Testing
```bash
# Run all tests with nextest
cargo nextest run --workspace

# Run doc tests
cargo test --doc --workspace

# Run single test crate
cargo nextest run -p <crate-name>
```

### Code Quality
```bash
# Format code (uses nightly toolchain)
cargo make ci-format

# Run clippy lints
cargo make ci-clippy

# Run all CI checks
cargo make ci-all
```

### ELF Management
```bash
# Build dummy program ELF
cargo make dummy-elf

# Interactive SP1 toolchain shell
cargo make sp1-toolchain-shell
```

## Key Dependencies

- **SP1**: Succinct's zero-knowledge proof system
- **Alloy**: Modern Ethereum library
- **Interop**: Agglayer interoperability libraries
- **Protocol Buffers**: For service definitions and serialization

## ELF Binary Management

The project uses pre-compiled ELF binaries for SP1 proofs stored in crate-specific `elf/` directories. ELF binaries are built using deterministic Docker builds to ensure reproducibility.

### Automatic ELF Rebuild
Set `AGGLAYER_ELF_BUILD=update` environment variable to automatically rebuild ELF binaries when proof code changes. This requires Docker for the deterministic build process.

## SP1 Integration

The project integrates with SP1's proving network rather than generating proofs locally. This requires:
1. Succinct Prover Network access (Ethereum address registration)
2. SP1 toolchain installation for local development

## Workspace Configuration

- Default workspace members: `aggkit-prover`
- Excluded from workspace: `prover-dummy-program`, `aggchain-proof-program` (SP1 programs)
- Uses Rust 2021 edition with resolver v2

## Testing Strategy

The project uses `cargo-nextest` for parallel test execution and includes:
- Unit tests across all workspace crates
- Integration tests with Anvil (local Ethereum node)
- ELF binary integrity checks
- Feature isolation testing with `cargo-hack`
