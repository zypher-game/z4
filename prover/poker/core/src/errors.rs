use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PokerError {
    SignatureError,
    VerifySignatureError,
    BuildPlayEnvParasError,
    MorphError,
    NoCardError,
    VerifyReVealError,
    UnmaskCardError,
}

pub type Result<T> = core::result::Result<T, PokerError>;

impl Display for PokerError {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::SignatureError => "Signature failed",
            Self::VerifySignatureError => "Signature verification failed",
            Self::BuildPlayEnvParasError => "Incorrect parameters of playerEnv",
            Self::MorphError => "Merph to classic card failed",
            Self::NoCardError => "No card error",
            Self::VerifyReVealError => "Verify reveal failed",
            Self::UnmaskCardError => "Unmask card failed",
        })
    }
}
