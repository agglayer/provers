use std::sync::Arc;
use std::time::Duration;

use prover_config::MockProverConfig;
use sp1_sdk::{
    CpuProver, Prover, ProverClient, SP1ProofMode, SP1ProofWithPublicValues, SP1Stdin,
    SP1_CIRCUIT_VERSION,
};
use tower::timeout::TimeoutLayer;
use tower::{service_fn, Service, ServiceBuilder, ServiceExt};

use crate::{Executor, LocalExecutor, Request, Response};
const ELF: &[u8] = include_bytes!("../../prover-dummy-program/elf/riscv32im-succinct-zkvm-elf");

fn mock_proof(stdin: SP1Stdin) -> SP1ProofWithPublicValues {
    let client = ProverClient::builder().cpu().build();
    let (pk, _vk) = client.setup(ELF);
    let (public_values, _) = client.execute(&pk.elf, &stdin).run().unwrap();

    // Create a mock Plonk proof.
    SP1ProofWithPublicValues::create_mock_proof(
        &pk,
        public_values,
        SP1ProofMode::Plonk,
        SP1_CIRCUIT_VERSION,
    )
}

#[tokio::test]
async fn executor_normal_behavior() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|r: Request| async move {
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|_: Request| async { panic!("Shouldn't be called") }),
    );

    let mut executor = Executor::new_with_services(network, Some(local));
    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_network");
}

#[tokio::test]
async fn executor_normal_behavior_only_network() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|r: Request| async move {
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(network, None);
    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_network");
}

#[tokio::test]
async fn executor_fallback_behavior_cpu() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|_: Request| async { Err(crate::Error::ProverFailed("failure".to_string())) }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|r: Request| async move {
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(network, Some(local));
    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_local");
}

#[tokio::test]
async fn executor_fallback_because_of_timeout_cpu() {
    let network = Executor::build_network_service(
        Duration::from_millis(100),
        service_fn(|r: Request| async {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|r: Request| async {
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(network, Some(local));

    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_local");
}

#[tokio::test]
async fn executor_fails_because_of_timeout_cpu() {
    let network = Executor::build_network_service(
        Duration::from_millis(100),
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_millis(100),
        1,
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_millis(100)))
        .service(Executor::new_with_services(network, Some(local)));

    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn executor_fails_because_of_concurrency_cpu() {
    let network = Executor::build_network_service(
        Duration::from_millis(100),
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(20),
        1,
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(1)))
        .service(Executor::new_with_services(network, Some(local)));

    let mut executor2 = executor.clone();

    tokio::spawn(async move {
        executor
            .ready()
            .await
            .unwrap()
            .call(Request {
                stdin: SP1Stdin::new(),
            })
            .await
    });

    let result = executor2
        .ready()
        .await
        .unwrap()
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn executor_normal_behavior_mock_prover() {
    let prover = Arc::new(CpuProver::mock());
    let (proving_key, verification_key) = prover.setup(ELF);

    let mock_prover_config = MockProverConfig::default();
    let mut executor = Executor::build_local_service(
        mock_prover_config.proving_timeout,
        mock_prover_config.max_concurrency_limit,
        LocalExecutor {
            prover: prover.clone(),
            proving_key,
            verification_key: verification_key.clone(),
        },
    );
    let executor = executor.ready().await.expect("valid executor");

    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
        })
        .await;

    assert!(result.is_ok());
    assert!(prover
        .verify(&result.unwrap().proof, &verification_key)
        .is_ok());
}
