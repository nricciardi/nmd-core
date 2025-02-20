use thiserror::Error;


#[derive(Error, Debug)]
pub enum CompilableStringError {
    #[error("compilable content {0} has an overflow using {1} -> {2}")]
    ContentOverflow(String, usize, usize),
}