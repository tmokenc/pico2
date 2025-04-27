use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum Error {
    #[error("The file is too large for the operation")]
    FileTooLarge,

    #[error("Invalid address: {0:#X}")]
    InvalidAddress(u32),

    #[error("Invalid target address")]
    MemoryError(#[from] crate::memory::MemoryOutOfBoundsError),

    #[error("Invalid UF2 file")]
    UF2Error(#[from] uf2::Error),
}
