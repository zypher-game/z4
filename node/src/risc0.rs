use risc0_zkvm::{Executor as RiscExecutor, ExecutorEnv, LocalProver};
use z4_engine::{DefaultParams, Error, HandleResult, Result};

use crate::Executor;

pub struct Risc0(LocalProver);

impl Executor for Risc0 {
    fn create() -> Self {
        Risc0(LocalProver::new("local"))
    }

    fn execute(
        &self,
        code: &[u8],
        storage: &[u8],
        params: &DefaultParams,
    ) -> Result<(Vec<u8>, HandleResult<DefaultParams>)> {
        // TODO limit cycles

        let env = ExecutorEnv::builder()
            .write(&hex::encode(storage))
            .unwrap()
            .write(params)
            .unwrap()
            .build()
            .unwrap();

        let info = self.0.execute(env, code).unwrap();
        // Ok(info.journal.decode()?)

        todo!()
    }
}
