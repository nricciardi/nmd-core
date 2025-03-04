use thiserror::Error;

use super::transformation_rule::transformation_error::TransformationError;


#[derive(Error, Debug)]
pub enum TextCompilationError {
    #[error(transparent)]
    TransformationError(#[from] TransformationError),
}