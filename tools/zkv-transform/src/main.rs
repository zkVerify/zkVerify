use clap::Parser;
use codec::{Encode, Decode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "zkv-transform")]
#[command(about = "Convert zk-proof artifacts to on-chain SCALE format")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
    
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    #[arg(long, default_value = "stwo")]
    verifier: String,
}

#[derive(Serialize, Deserialize)]
struct StwoArtifacts {
    verification_key: Vec<u8>,
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
}

#[derive(Encode, Decode)]
struct OnChainProof {
    vk: Vec<u8>,
    proof: Vec<u8>,
    inputs: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.verifier.as_str() {
        "stwo" => {
            let content = fs::read_to_string(&cli.input)?;
            let artifacts: StwoArtifacts = serde_json::from_str(&content)?;
            
            let onchain = OnChainProof {
                vk: artifacts.verification_key,
                proof: artifacts.proof,
                inputs: artifacts.public_inputs,
            };
            
            let encoded = onchain.encode();
            let hex = hex::encode(&encoded);
            
            if let Some(output) = cli.output {
                fs::write(&output, hex)?;
                println!("Converted artifacts written to {:?}", output);
            } else {
                println!("SCALE-encoded hex:\n{}", hex);
            }
        }
        _ => {
            eprintln!("Unsupported verifier: {}", cli.verifier);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

