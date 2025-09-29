# Proposer Client

This crate provides a client for interacting with the proposer service and managing aggregation proofs.

## gRPC Code Generation

The gRPC client code for `op-succinct-grpc` is generated using [buf](https://buf.build) from the [`buf.build/agglayer/op-succinct-grpc`](https://buf.build/agglayer/op-succinct-grpc) module.


### Regenerating gRPC Code with cargo make

To regenerate the gRPC code after proto changes (it would also regenerate all other proto files), use:

```bash
# From the project root
cargo make generate-proto
```

### Regenerating gRPC Code without cargo make

To regenerate the gRPC code after proto changes:

```bash
# From the project root
buf generate --template buf.op-succinct-grpc.gen.yaml
```

The generated files are located in `src/generated/`:
- `mod.rs` - Module declarations
- `proofs.rs` - Generated message types
- `proofs.serde.rs` - Serde implementations
- `proofs.tonic.rs` - gRPC client/server code

### Dependencies

The generated code requires:
- `prost` - Protocol Buffers implementation
- `tonic` - gRPC implementation
- `pbjson` - JSON serialization support
- `serde` - Serialization framework

### Migration from op-succinct-grpc

This crate previously used the [op-succinct-grpc](https://github.com/agglayer/op-succinct-grpc) Git dependency. It has been replaced with buf build generation for:
- Better dependency management
- Consistent code generation
- Version pinning via buf.lock
- Integration with the buf ecosystem

The API remains the same - the generated `proofs` module provides the same types and client interfaces as the previous dependency. 