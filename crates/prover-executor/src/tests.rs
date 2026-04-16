use std::{sync::Arc, time::Duration};

use prover_config::MockProverConfig;
use sp1_sdk::{
    MockProver, Prover, ProvingKey as _, SP1ProofMode, SP1ProofWithPublicValues, SP1ProvingKey,
    SP1Stdin, SP1VerifyingKey, SP1_CIRCUIT_VERSION,
};
use tokio::sync::OnceCell;
use tower::{service_fn, timeout::TimeoutLayer, Service, ServiceBuilder, ServiceExt};

use crate::{Executor, LocalExecutor, LocalProver, ProofType, Request, Response};
const ELF: &[u8] = proposer_elfs::aggregation::ELF;

async fn mock_prover() -> &'static MockProver {
    static RES: OnceCell<MockProver> = OnceCell::const_new();
    RES.get_or_init(|| async { MockProver::new().await }).await
}

async fn pkey_vkey() -> &'static (Arc<SP1ProvingKey>, Arc<SP1VerifyingKey>) {
    static RES: OnceCell<(Arc<SP1ProvingKey>, Arc<SP1VerifyingKey>)> = OnceCell::const_new();
    RES.get_or_init(|| async {
        let pkey = mock_prover()
            .await
            .setup(ELF.into())
            .await
            .expect("setting up proving key");
        let vkey = pkey.verifying_key().clone();
        (Arc::new(pkey), Arc::new(vkey))
    })
    .await
}

async fn pkey() -> &'static Arc<SP1ProvingKey> {
    &pkey_vkey().await.0
}

async fn vkey() -> &'static Arc<SP1VerifyingKey> {
    &pkey_vkey().await.1
}

async fn mock_proof(stdin: SP1Stdin) -> SP1ProofWithPublicValues {
    let proving_key = pkey().await;
    let (public_values, _) = mock_prover()
        .await
        .execute(proving_key.elf().clone(), stdin)
        .await
        .expect("executing prover input");

    // Create a mock Plonk proof.
    SP1ProofWithPublicValues::create_mock_proof(
        proving_key.verifying_key(),
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
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|_: Request| async { panic!("Shouldn't be called") }),
    );

    let mut executor = Executor::new_with_services(vkey().await.clone(), network, Some(local));
    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
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
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(vkey().await.clone(), network, None);
    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
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
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(vkey().await.clone(), network, Some(local));
    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
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
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|r: Request| async {
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(vkey().await.clone(), network, Some(local));

    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
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
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_millis(100),
        1,
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_millis(100)))
        .service(Executor::new_with_services(
            vkey().await.clone(),
            network,
            Some(local),
        ));

    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
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
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(20),
        1,
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let mut proof = mock_proof(r.stdin).await;
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(1)))
        .service(Executor::new_with_services(
            vkey().await.clone(),
            network,
            Some(local),
        ));

    let mut executor2 = executor.clone();

    tokio::spawn(async move {
        executor
            .ready()
            .await
            .unwrap()
            .call(Request {
                stdin: SP1Stdin::new(),
                proof_type: ProofType::Plonk,
            })
            .await
    });

    let result = executor2
        .ready()
        .await
        .unwrap()
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
        })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn executor_normal_behavior_mock_prover() {
    let prover = MockProver::new().await;
    let proving_key = prover
        .setup(ELF.into())
        .await
        .expect("setting up proving key");
    let verification_key = proving_key.verifying_key().clone();

    let mock_prover_config = MockProverConfig::default();
    let mut executor = Executor::build_local_service(
        mock_prover_config.proving_timeout,
        mock_prover_config.max_concurrency_limit,
        LocalExecutor {
            prover: Arc::new(LocalProver::Mock(prover.clone())),
            proving_key,
            verification_key: verification_key.clone(),
        },
    );
    let executor = executor.ready().await.expect("valid executor");

    let result = executor
        .call(Request {
            stdin: SP1Stdin::new(),
            proof_type: ProofType::Plonk,
        })
        .await;

    assert!(result.is_ok());
    assert!(prover
        .verify(&result.unwrap().proof, &verification_key, None)
        .is_ok());
}
