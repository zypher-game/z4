use clap::{Parser, Args, ValueEnum};

#[derive(Parser)]
#[command(name = "z4")]
#[command(bin_name = "z4")]
enum Z4Cli {
    New(NewArgs),
    Deploy(DeployArgs),
}

#[derive(Args, Debug)]
#[command(version, about = "New game with Z4 & ZK", long_about = None)]
struct NewArgs {
    /// game name
    #[arg(long)]
    name: String,
    /// contract language
    #[arg(long)]
    contract: Option<Contract>,
    /// zk schemes or zkvm
    #[arg(long)]
    zk: Option<Zk>,
}

#[derive(Clone, Debug, ValueEnum)]
enum Contract {
    Solidity,
    RISCV,
    Move
}

#[derive(Clone, Debug, ValueEnum)]
enum Zk {
    Plonk,
    RISC0,
}

#[derive(Args, Debug)]
#[command(version, about = "Deploy contracts to chain", long_about = None)]
struct DeployArgs {
    /// optional contract name
    #[arg(long)]
    contract: Option<String>,
    /// RPC endpoint
    #[arg(long)]
    rpc: String,
    /// Deploy secret key
    #[arg(long)]
    sk: String,
}

fn main() {
    match Z4Cli::parse() {
        Z4Cli::New(args) => {
            println!("{:?}", args);
        }
        Z4Cli::Deploy(args) => {
            println!("{:?}", args);
        }
    }
}
