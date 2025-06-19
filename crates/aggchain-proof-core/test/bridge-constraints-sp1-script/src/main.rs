use std::{fs::File, io::BufReader, path::Path};

use aggchain_proof_core::bridge::BridgeConstraintsInput;
use anyhow::Result;
use sp1_sdk::{ProverClient, SP1Stdin};
use clap::Parser;

#[derive(Parser)]
#[command(name = "bridge-constraints-sp1-script")]
#[command(about = "Bridge Constraints SP1 Proof Test")]
struct Args {
    #[arg(long, help = "Generate and verify cryptographic proof")]
    prove: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Parse command line arguments
    let args = Args::parse();
    
    println!("🌉 Bridge Constraints SP1 Proof Test");
    println!("=====================================");
    if args.prove {
        println!("🔐 Proof generation: ENABLED");
    } else {
        println!("⚡ Proof generation: DISABLED (use --prove to enable)");
    }

    // Load ELF file (built by sp1_build)
    let elf = std::fs::read("elf/bridge-constraints-elf")?;

    // Load bridge constraints input
    let input_path = "../../src/test_input/bridge_constraints_input.json";
    if !Path::new(input_path).exists() {
        return Err(anyhow::anyhow!("Input file not found: {}", input_path));
    }

    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let bridge_input: BridgeConstraintsInput = serde_json::from_reader(reader)?;
    println!("✓ Loaded bridge constraints input");

    // Test verification locally first
    println!("Testing verification locally...");
    bridge_input.verify()?;
    println!("✓ Local verification passed");

    // Set up SP1 client
    let client = ProverClient::from_env();
    
    // Check if using network prover
    if std::env::var("SP1_PROVER").unwrap_or_default() == "network" {
        println!("🌐 Using SP1 Network Prover");
        if std::env::var("NETWORK_PRIVATE_KEY").is_err() {
            return Err(anyhow::anyhow!(
                "❌ NETWORK_PRIVATE_KEY not found. Set: export NETWORK_PRIVATE_KEY=your_key"
            ));
        }
    } else {
        println!("💻 Using Local CPU Prover");
    }

    // Set up SP1 input
    let mut stdin = SP1Stdin::new();
    stdin.write(&bridge_input);

    // Generate keys
    let (pk, vk) = client.setup(&elf);
    println!("✓ Generated proving keys");

    // Execute program
    println!("Executing SP1 program...");
    let (mut output, report) = client.execute(&elf, &stdin).run()?;
    println!("✓ Executed ({} cycles)", report.total_instruction_count());

    // Check output
    let result: bool = output.read::<bool>();
    if !result {
        return Err(anyhow::anyhow!("SP1 program verification failed"));
    }
    println!("✓ Program output: {}", result);

    if args.prove {
        // Generate proof
        println!("Generating proof...");
        let start = std::time::Instant::now();
        let proof = client.prove(&pk, &stdin).compressed().run()?;
        println!("✓ Generated proof in {:.2?}", start.elapsed());

        // Verify proof
        println!("Verifying proof...");
        client.verify(&proof, &vk)?;
        println!("✓ Proof verified successfully!");
    } else {
        println!("⚡ Skipping proof generation (use --prove to enable)");
    }

    Ok(())
} 