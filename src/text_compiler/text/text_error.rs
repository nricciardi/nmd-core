use thiserror::Error;


#[derive(Error, Debug)]
pub enum TextError {
    // #[error("cut operation can not be performed on a fixed part")]
    // OutOfBound,
}