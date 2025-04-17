use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq, FromPrimitive)]
pub enum MTreeError {
    #[error("unimplemented")]
    Test,
}

impl From<MTreeError> for ProgramError {
    fn from(e: MTreeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MTreeError {
    fn type_of() -> &'static str {
        "MTreeError"
    }
}

impl PrintProgramError for MTreeError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            MTreeError::Test => msg!("Error: Test error"),
        }
    }
}
