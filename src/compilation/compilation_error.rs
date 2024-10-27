use thiserror::Error;
use crate::{assembler::AssemblerError, compilable_text::CompilableError, resource::{resource_reference::ResourceReferenceError, ResourceError}};


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
    ReferenceError(#[from] ResourceReferenceError),

    #[error(transparent)]
    ResourceError(#[from] ResourceError),

    #[error(transparent)]
    CompilableError(#[from] CompilableError),

    #[error("unknown error occurs")]
    Unknown,
}