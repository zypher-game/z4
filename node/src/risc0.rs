use z4_engine::{DefaultParams, HandleResult};

pub use risc0_vm::LocalProver as Risc0;

impl Executor for Risc0 {
    fn create() -> Self {
        LocalProver::new("local")
    }

    fn execute(
        &self,
        code: &[u8],
        storage: Vec<u8>,
        params: DefaultParams,
    ) -> Result<(Vec<u8>, HandleResult), Error> {
        // TODO limit cycles

        let env = ExecutorEnv::builder()
            .write(storage)
            .unwrap()
            .write(params)
            .unwrap()
            .build()
            .unwrap();

        let info = self.execute(env, code).unwrap();
        info.journal.decode()
    }
}
