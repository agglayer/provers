# Bridge Constraints SP1 Test

SP1 proof system for bridge constraints verification.

## What This Does

Tests the `bridge_data_input_verify()` function from `aggchain-proof-core` using SP1:
- **Program**: RISC-V program that verifies bridge constraints 
- **Script**: Orchestrates proof generation and verification

## Prerequisites

```bash
# Install SP1 toolchain
curl -L https://sp1up.succinct.xyz | bash
source ~/.bashrc && sp1up
```

## Run Local Test

Quick test (execution only):
```bash
cd crates/aggchain-proof-core/test/bridge-constraints-sp1-script
cargo run --release
```

Full test with proof generation:
```bash
cd crates/aggchain-proof-core/test/bridge-constraints-sp1-script
cargo run --release -- --prove
```

## Run Network Test

Option 1 - Environment variables:
```bash
cd crates/aggchain-proof-core/test/bridge-constraints-sp1-script
export SP1_PROVER=network
export NETWORK_PRIVATE_KEY=your_key_here
export NETWORK_RPC_URL=https://rpc.production.succinct.xyz
cargo run --release -- --prove  # Add --prove for actual proof generation
```

Option 2 - Using `.env` file (recommended):
```bash
cd crates/aggchain-proof-core/test/bridge-constraints-sp1-script
echo "SP1_PROVER=network" >> .env
echo "NETWORK_PRIVATE_KEY=your_key_here" >> .env
echo "NETWORK_RPC_URL=https://rpc.production.succinct.xyz"  >> .env
cargo run --release -- --prove  # Add --prove for actual proof generation
```

## Flags

- **`--prove`**: Generate and verify cryptographic proof (slow)
- **No flag**: Execute SP1 program only (fast, for development)


**Performance**: ~60M cycles, proof time varies by mode (local vs network)
