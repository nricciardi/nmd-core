use thiserror::Error;

use crate::{assembler::AssemblerError, mmo::{uri::UriError, MultiMediaObjectError}, text_compiler::text_compilation_error::TextCompilationError};


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
    TextCompilationError(#[from] TextCompilationError),

    #[error("heading level not inferable: {0}")]
    HeadingLevelNotInferable(String),

    #[error("unknown error occurs")]
    Unknown,
}