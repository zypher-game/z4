use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PokerError {
    SignatureError,
    VerifySignatureError,
    BuildPlayEnvParamsError,
    MorphError,
    NoCardError,
    VerifyReVealError,
    UnmaskCardError,
    SerializationError,
    DeserializationError,
}

pub type Result<T> = core::result::Result<T, PokerError>;

impl Display for PokerError {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::SignatureError => "Signature failed",
            Self::VerifySignatureError => "Signature verification failed",
            Self::BuildPlayEnvParamsError => "Incorrect parameters of playerEnv",
            Self::MorphError => "Merph to classic card failed",
            Self::NoCardError => "No card error",
            Self::VerifyReVealError => "Verify reveal failed",
            Self::UnmaskCardError => "Unmask card failed",
            Self::SerializationError => "Serialization error",
            Self::DeserializationError => "Deserialization error",
        })
    }
}
