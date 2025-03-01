use thiserror::Error;

use crate::{assembler::AssemblerError, compilable_string::compilable_string_error::CompilableStringError, mmo::{uri::UriError, MultiMediaObjectError}};


#[derive(Error, Debug)]
pub enum CompilationError {

    #[error("bucket of errors: '{0:#?}'")]
    BucketOfErrors(Vec<CompilationError>),

    #[error(transparent)]
    AssemblerError(#[from] AssemblerError),

    #[error("pattern provided '{0}' is invalid")]
    InvalidPattern(String),

    #[error("'{0}' is an invalid source")]
    InvalidSource(String),

    #[error("failed during elaboration")]
    ElaborationError,

    #[error("failed during elaboration: {0}")]
    ElaborationErrorVerbose(String),

    #[error("document name not found")]
    DocumentNameNotFound,

    #[error("'{0}' is an invalid parameter")]
    InvalidParameter(String),

    #[error(transparent)]
    UriError(#[from] UriError),

    #[error(transparent)]
    MmoError(#[from] MultiMediaObjectError),

    #[error(transparent)]
    CompilableStringError(#[from] CompilableStringError),

    #[error("heading level not inferable: {0}")]
    HeadingLevelNotInferable(String),

    #[error("unknown error occurs")]
    Unknown,
}