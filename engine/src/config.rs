use crate::types::Account;

#[derive(Default)]
pub struct Config {
    pub account: Account,
    pub signers: Vec<Account>,
    pub chain_rpcs: Vec<String>,
}
