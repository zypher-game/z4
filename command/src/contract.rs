// use ethers::prelude::*;

async fn compile(_contract: Option<String>) {
    //
}

pub async fn deploy(_rpc: String, _sk: String, contract: Option<String>) {
    compile(contract).await;
}
