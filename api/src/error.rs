use steel::*;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum EntropyError {
    #[error("Incomplete digest")]
    IncompleteDigest = 0,

    #[error("Invalid seed")]
    InvalidSeed = 1,
}

error!(EntropyError);
