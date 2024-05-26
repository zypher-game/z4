use clap::{Subcommand, Parser, Args, ValueEnum};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Z4 {
        #[command(subcommand)]
        sub: Z4SubCli
    }
}

#[derive(Subcommand)]
enum Z4SubCli {
    New(Z4NewArgs),
    Deploy(Z4DeployArgs),
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct Z4NewArgs {
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
struct Z4DeployArgs {
    #[arg(long)]
    rpc: String,
    #[arg(long)]
    sk: String,
}

fn main() {
    let CargoCli::Z4 { sub } = CargoCli::parse();
    match sub {
        Z4SubCli::New(args) => {
            println!("{:?}", args);
        }
        Z4SubCli::Deploy(args) => {
            println!("{:?}", args);
        }
    }
}
