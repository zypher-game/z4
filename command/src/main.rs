use clap::{Args, Parser};
use std::process::Command;

// const REPO: &str = "https://github.com/zypher-game/z4-templates.git"; // must use https git
const CONTRACT_MODES: [&str; 1] = ["solidity"];
const ZK_MODES: [&str; 2] = ["custom", "risc0"];

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
    contract: Option<String>,
    /// zk schemes or zkvm
    #[arg(long)]
    zk: Option<String>,
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
            let contract_mode = if let Some(contract) = args.contract {
                if CONTRACT_MODES.contains(&contract.to_lowercase().as_str()) {
                    contract.to_lowercase()
                } else {
                    panic!("NO contract mode: {}", contract);
                }
            } else {
                "solidity".to_owned()
            };

            let zk_mode = if let Some(zk) = args.zk {
                if ZK_MODES.contains(&zk.to_lowercase().as_str()) {
                    zk.to_lowercase()
                } else {
                    panic!("NO zk mode: {}", zk);
                }
            } else {
                "custom".to_owned()
            };

            // git clone template
            clone_template(&args.name);

            // reorganize contracts & code
            clean_mode(&args.name, &zk_mode, &contract_mode);

            // TODO rename cargo & contract name

            println!(
                "{} with {}-{} ready. Happy hacking!",
                args.name, contract_mode, zk_mode
            );
        }
        Z4Cli::Deploy(args) => {
            println!("{:?}", args);
        }
    }
}

fn clone_template(name: &str) {
    println!("Cloning Z4 template");
    let out = Command::new("git")
        .args(&["clone", REPO, &format!("{}_tmp", name)])
        .output()
        .expect("Failed to execute command");

    if !out.status.success() {
        println!("{}", String::from_utf8(out.stderr).unwrap());
        panic!("git clone template failure, please check git and network!");
    }

    println!("Cloned Z4 template");
}

fn clean_mode(name: &str, z_mode: &str, c_mode: &str) {
    println!("Cleaning Z4 modes");
    let out1 = Command::new("rm")
        .args(&[&format!("{}_tmp/{}/contracts", name, z_mode)])
        .output()
        .expect("Failed to execute command");
    if !out1.status.success() {
        println!("{}", String::from_utf8(out1.stderr).unwrap());
        panic!("Unable delete softlink contraact in zk mode!");
    }

    let out2 = Command::new("mv")
        .args(&[&format!("{}_tmp/{}", name, z_mode), name])
        .output()
        .expect("Failed to execute command");
    if !out2.status.success() {
        println!("{}", String::from_utf8(out2.stderr).unwrap());
        panic!("Unable move zk mode to current directory!");
    }
    println!("Cleaned Z4 zk");

    let out3 = Command::new("mv")
        .args(&[
            &format!("{}_tmp/contracts/{}", name, c_mode),
            &format!("{}/contracts", name),
        ])
        .output()
        .expect("Failed to execute command");
    if !out3.status.success() {
        println!("{}", String::from_utf8(out3.stderr).unwrap());
        panic!("Unable move contract mode to current directory!");
    }
    println!("Cleaned Z4 contracts");

    let out4 = Command::new("rm")
        .args(&["-rf", &format!("{}_tmp", name)])
        .output()
        .expect("Failed to execute command");
    if !out4.status.success() {
        println!("{}", String::from_utf8(out3.stderr).unwrap());
        println!("Unable delete {}_tmp, you need delete it!", name);
    }
}
