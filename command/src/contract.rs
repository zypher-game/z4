use ethers::prelude::*;

async fn compile(contract: Option<String>) {
    //
}

async fn deploy(rpc: String, sk: String, contract: Option<String>) {
    compile(contract).await;
}
