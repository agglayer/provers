pub mod error;
pub(crate) mod rpc;
pub(crate) mod service;

pub use rpc::GrpcService;

#[cfg(test)]
mod tests;
