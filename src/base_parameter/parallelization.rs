use serde::Serialize;



#[derive(Debug, Default, Clone, Serialize)]
pub enum Parallelization {

    #[default]
    No,
    Auto,
    Max
}