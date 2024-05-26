use clap::{Parser, Args, ValueEnum};

#[derive(Parser)]
#[command(name = "z4")]
#[command(bin_name = "z4")]
enum Z4Cli {
    New(NewArgs),
    Deploy(DeployArgs),
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct NewArgs {
    #[arg(long)]
    contract: Option<Contract>,
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
#[command(version, about, long_about = None)]
struct DeployArgs {
    #[arg(long)]
    rpc: String,
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
