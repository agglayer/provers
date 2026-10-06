<div id="top"></div>
<!-- PROJECT LOGO -->
<br />
<div align="center">

  <img src="./.github/assets/agglayer-logo.png#gh-light-mode-only" alt="Logo" width="100">
  <img src="./.github/assets/agglayer-logo.png#gh-dark-mode-only" alt="Logo" width="100">

<br />

<h1>Agglayer provers</h1>

<p align="center">
The <b>Agglayer</b> (<i>Aggregation layer</i>) provides a common language for secure, atomic, interoperability among heterogeneous chains. (WIP)
</p>
</div>

<br />

<div align="center">

[![Test workflow](https://github.com/agglayer/agglayer/actions/workflows/test.yml/badge.svg)](https://github.com/agglayer/agglayer/actions/workflows/test.yml)
[![Quality workflow](https://github.com/agglayer/agglayer/actions/workflows/quality.yml/badge.svg)](https://github.com/agglayer/agglayer/actions/workflows/quality.yml)
[![codecov](https://codecov.io/gh/agglayer/agglayer/graph/badge.svg?token=5TOBZRZ7Q8)](https://codecov.io/gh/agglayer/agglayer)

<hr />

<img src="./.github/assets/agglayer.png" alt="Logo">

</div>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Overview](#overview)
- [Repository Structure](#repository-structure)
- [Prerequisites](#prerequisites)
  - [Succinct Prover Network](#succinct-prover-network)
  - [Software Requirements](#software-requirements)
  - [Hardware Recommendations](#hardware-recommendations)
- [Installation](#installation)
- [Running the Test Suite](#running-the-test-suite)
- [Modifying and building the Aggchain Proof](#modifying-and-building-the-aggchain-proof)
  - [Building Aggchain Proof one-off](#building-aggchain-proof-one-off)
  - [Turning on automatic proof rebuild](#turning-on-automatic-proof-rebuild)
  - [Check the vkey hash from the ELF binary](#check-the-vkey-hash-from-the-elf-binary)
  - [Proof versioning policy](#proof-versioning-policy)
- [Development](#development)
- [Support](#support)
- [License](#license)

## Overview

Agglayer is the Rust-based service designed to: 
1. Receive updates from Agglayer-connected chains 
2. Verify their validity 
3. Send them to the L1 for final settlement. 

To find out more about Agglayer, please visit [the more detailed documentation.](https://docs.polygon.technology/agglayer/overview/)

> [!WARNING]
>    - Some of the content in this section discusses technology in development and not ready for release. As such, all APIs and configuration are subject to change. The code is still being audited, so please contact the Polygon team if you would like to use it in production.

## Repository Structure

The crates and their functions within the Agglayer repo are as follows:

TODO

## Prerequisites

Before working with the repository, you’ll need the following:

### Succinct Prover Network

You’ll need to submit a unique Ethereum address to Succinct for access to their proving network. To get access:

1. Follow the instructions [here](https://docs.succinct.xyz/docs/generating-proofs/prover-network/key-setup) to use Foundry to generate a new private key or retrieve an existing one.
2. Apply for access for the public address associated with your private key to Succinct Network [here](https://docs.google.com/forms/d/e/1FAIpQLSd-X9uH7G0bvXH_kjptnQtNil8L4dumrVPpFE4t8Ci1XT1GaQ/viewform).

### Software Requirements
* [Rustup](https://www.rust-lang.org/tools/install) (stable)
* [protoc](https://grpc.io/docs/protoc-installation/)
* [nextest](https://nexte.st/docs/installation/pre-built-binaries/#with-cargo-binstall)
* [cargo-make](https://github.com/sagiegurari/cargo-make#installation)
* [cargo-insta](https://insta.rs/docs/quickstart/)
* [Go](https://go.dev/doc/install)

### Hardware Recommendations
With SP1, you do not need to generate proofs locally on your machine.

However, if you’d like to run a prover locally (not recommended), you’ll need roughly 40-50GB of available RAM.

## Installation

To install the Agglayer provers repository, please run the following:

```bash
git clone https://github.com/agglayer/provers
cd provers
```

To build Agglayer provers locally, please run:
```bash
cargo build
```

## Running the Test Suite

To execute the test suite, please run the following:

```bash
cargo nextest run --workspace
```

## Modifying and building the Aggchain Proof

By default, the committed pre-compiled ELF binary is used.
Modifications in proof code will not be automatically reflected in the binary.
We use docker-based deterministic build to compile the proof.
Therefore, `docker` has to be present on the system for the build to work if rebuild is enabled.

### Building Aggchain Proof one-off

The following command rebuilds the Aggchain proof and.
It requires `cargo-make` to be installed:

```sh
cargo make ap-elf
```

### Turning on automatic proof rebuild

This option makes the standard commands like `cargo build`, `cargo run` etc. rebuild the proof automatically any time it changes as if it were a normal part of the build.
It is enabled by setting the `AGGLAYER_ELF_BUILD` environment variable to `update`.

```sh
export AGGLAYER_ELF_BUILD=update
```

Note: Rust suppresses the output of build scripts by default.
As a result, the build may appear stuck on the `aggchain-proof-builder` crate while the proof is being rebuilt.

In the `update` mode, the proof will be rebuilt and the cached ELF will be updated.
There is also the `build` mode which leaves the cached ELF intact.
It is mostly useful for debugging, the `update` is more suitable for regular development.

To get automatic rebuilds by default, set the variable in the shell init script.

### Check the vkey hash from the ELF binary

This command helps to retrieve the vkey hash from a given ELF binary:

```
cargo prove vkey --elf <path-to-elf-file>
```

Output example:

```
Verification Key Hash:
0x0077f45ec2258cc98fa879d13a2773190bffb9cafb9f428ce3c5718dc768f03e
```

Which ELFs?

- `aggchain proof` ELF binary
  - lives in this present `provers` repository
- `aggregation` and `range` ELF binaries for the op-succinct proofs
  - live in the `op-succinct` dependency repository

### Proof versioning policy

The proof binary to use is uniquely identified by a vkey selector on the L1.
The selector is derived from the major version of the `aggchain-proof-program` package.
This version must be bumped between releases / deployments.

### Aggchain proof modes

`mode` in `[aggchain-proof-service.aggchain-proof-builder]` selects what the prover does with a request:

- `standard` (default): the standard aggchain program is proven. It verifies the FEP (for an optimistic certificate, the trusted sequencer signature) and the bridge constraints.
- `skip-proof-verification`: executes the standard aggchain program with FEP proof verification disabled, then proves the noop program over the resulting public values. op-succinct-proposer runs in SP1 mock mode and is still asked for the aggregation proof, so it keeps deriving the chain from L1 and decides the end block.
- `recovery`: after an L2 reorg past a settled output. Nothing is executed: the public values are built from L1 (pre-root of the latest L1 output, last settled local exit root, `optimisticMode`, op-succinct config), the L2 bridge root and output at the end block and the request, and the noop program is proven over them. op-succinct-proposer is not used.

| Mode | Selector | Normal request | Optimistic request |
|---|---|---|---|
| `standard` | `0x000C0001` | **Checks:** FEP proof, FEP L1-head inclusion, bridge constraints. **Skipped:** trusted sequencer signature (not required). | **Checks:** trusted sequencer signature, bridge constraints. **Skipped:** FEP proof and FEP L1-head inclusion. |
| `skip-proof-verification` | `0xFFFF0001` | **Checks:** FEP L1-head inclusion and bridge constraints during host-side execution; proposer public values compared with contract data. **Skipped:** FEP proof verification (`deferred_proof_verification(false)`); trusted sequencer signature (not required). | **Checks:** trusted sequencer signature and bridge constraints during host-side execution. **Skipped:** FEP proof and FEP L1-head inclusion. |
| `recovery` | `0xFFFF0001` | **Checks:** host checks the request starts at the latest settled L1 output, uses its pre-root and ends after it. **Skipped:** FEP proof, FEP L1-head inclusion, bridge constraints; trusted sequencer signature (not required). | **Checks:** same L1 anchor check as normal recovery. **Skipped:** FEP proof, FEP L1-head inclusion, bridge constraints, trusted sequencer signature. |

With a real SP1 prover, `standard` proves the standard program's checks. `skip-proof-verification` and `recovery` prove only the noop program's commitment to the supplied public values; their host-side checks are not established by the resulting proof. The selector follows the configured mode for both request types. The standard selector above corresponds to program version 12; the noop selector uses the reserved version `0xFFFF`.

`primary-prover` says how the program of the mode is proven: `network-prover` or `cpu-prover` give a real SP1 proof, `mock-prover` an SP1 mock proof that only a mock verifier accepts.
Use the following settings for each execution path (the proposer settings apply to normal requests):

| Execution path | `mode` setting | `primary-prover` | `proposer-service.mock` | op-succinct-proposer | Result |
|---|---|---|---|---|---|
| Full sp1-mock (Kurtosis) | Omit; defaults to `"standard"` | `mock-prover` | `true` | `OP_SUCCINCT_MOCK=true` | sp1-mock FEP and sp1-mock standard aggchain proof; requires a sp1-mock verifier downstream |
| Full sp1-real | Omit; defaults to `"standard"` | `network-prover` or `cpu-prover` | `false` | `OP_SUCCINCT_MOCK=false` | sp1-real FEP and sp1-real standard aggchain proof |
| Skip proof verification | **Required: `mode = "skip-proof-verification"`** | `network-prover` or `cpu-prover` | Ignored; sp1-mock proposer client is selected | `OP_SUCCINCT_MOCK=true` | sp1-mock FEP, host execution of the standard program, sp1-real noop proof |
| Recovery | **Required: `mode = "recovery"`** | `network-prover` or `cpu-prover` | Ignored; proposer is unused | May be stopped | Public values built from L1/L2 data, sp1-real noop proof |

Only `skip-proof-verification` and `recovery` require the new `mode` field. Existing full mock and full real configurations keep working without it; `mode = "standard"` is also accepted explicitly. Set `mode` under `[aggchain-proof-service.aggchain-proof-builder]` and `mock` under `[aggchain-proof-service.proposer-service]`. The prover types select TOML subtables, for example `[aggchain-proof-service.aggchain-proof-builder.primary-prover.mock-prover]` for Kurtosis or `[aggchain-proof-service.aggchain-proof-builder.primary-prover.network-prover]` for network proving; retain the other required endpoint and contract settings.

The full mock path still executes the standard aggchain program, but skips deferred FEP proof verification and produces no cryptographic proof of its checks. Choosing `cpu-prover` changes only aggchain proving; real FEP generation still uses the proposer network path. If a `fallback-prover` is configured, use a real prover there too when real proofs are required. `skip-proof-verification` and `recovery` also accept `mock-prover` with a warning, but then their noop proofs require a mock verifier as well.

A chain in `optimisticMode` needs no mode of its own: the aggsender sends optimistic requests, which every mode serves without calling op-succinct-proposer. In `recovery` the flag hashed into the aggchain params is read from the contract, so a normal request also matches.

#### Switching to the noop program

`skip-proof-verification` and `recovery` prove `aggchain-proof-noop-program`, a program that commits its public values without verifying them.
It is a real SP1 proof of an empty program, unrelated to the SP1 mock prover.
The certificate carries the selector `0xFFFF0001` instead of the standard one, and the contract only accepts it with the noop vkey registered in the rollup's own `ownedAggchainVKeys`.
Never register it on the `AgglayerGateway`.

1. Print the vkeys and selectors: `aggkit-prover vkey --noop`, `aggkit-prover vkey-selector --noop`, `aggkit-prover vkey`, `aggkit-prover vkey-selector`.
2. As aggchain manager: `addOwnedAggchainVKey(0xFFFF0001, <noop vkey>)` and `addOwnedAggchainVKey(<standard selector>, <standard vkey>)`, then `disableUseDefaultVkeysFlag()`. Owning the standard vkey keeps in-flight certificates and the way back working.
3. Set `mode`, restart the prover.
4. A certificate already InError keeps the aggchain proof the aggsender cached for it: drop that proof (resync the aggsender database) if needed.

To switch back:

1. Set `mode = "standard"`. For normal FEP requests, restore op-succinct-proposer to the mode expected by `proposer-service.mock` (`OP_SUCCINCT_MOCK=false` for real proofs), and restart it if stopped. Start the proposer before restarting aggkit-prover; `standard` and `skip-proof-verification` modes connect to it at startup.
2. Restart aggkit-prover. Wait for any pending noop certificates to settle before calling `enableUseDefaultVkeysFlag()` to follow the gateway defaults again.

#### Skipping FEP proof verification (`skip-proof-verification`)

In `skip-proof-verification` the FEP proof is not verified. Native execution uses the builder's `proving-timeout` and the primary local prover's `max-concurrency-limit` (the default limit for a network prover). A timed-out execution retains its slot until it finishes.
The execution of the L2 is only checked by op-succinct-proposer, which derives and executes the range without proof, and by the prover, which compares the proposer's output roots with the L2 node's.
Everything else in the standard program runs: the L1 head against the L1 info tree, the bridge constraints and the aggchain params, or the trusted sequencer signature for an optimistic certificate.

1. Restart op-succinct-proposer with `OP_SUCCINCT_MOCK=true`.
2. Switch to `mode = "skip-proof-verification"` as above.

The prover still sends every normal request to op-succinct-proposer and fetches its SP1 mock aggregation proof, whatever `proposer-service.mock` says.
Against an op-succinct-proposer in real mode, that request starts a real, paid aggregation proof and no certificate comes out, hence the order of the steps.

#### Recovering a halted FEP chain (`recovery`)

When the L2 reorgs past an output already settled on L1, the `AggchainFEP` contract keeps hashing the orphaned output root as the pre-root while the prover proves from the live L2 pre-root, and every certificate fails on the agglayer with `Aggchain hash mismatch`.
Settled outputs cannot be rewritten, so the way out is one certificate whose pre-root is the L1 one.
Bridge constraints are not verified in this mode, so this is for non-production chains only.

1. Check that the local exit root on L2 at the last settled block matches the one on L1. If not, reconcile it first with the bridge `BackwardLET` / `ForwardLET` tooling.
2. Switch to `mode = "recovery"` as above. op-succinct-proposer can be stopped.
3. Once the certificate is settled, follow the switch-back steps above.

With `mode = "recovery"` the prover refuses a request that is not anchored at the latest L1 output.
It asks nothing from op-succinct-proposer and reads no L2 state at the reorged anchor: the previous local exit root, the pre-root, `optimisticMode` and the op-succinct config come from L1, and only the L2 bridge root and the L2 output at the end block come from the L2 node.
An L2 node that cannot serve historical state at the anchor is therefore not a blocker, but it must serve the end block.

## Development

Contributions are very welcomed, the guidelines are currently not available (WIP)

## Support

Feel free to [open an issue](https://github.com/agglayer/agglayer/issues/new) if you have any feature request or bug report.<br />

## License
Copyright (c) 2024 PT Services DMCC

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

The SPDX license identifier for this project is `MIT OR Apache-2.0`.
