use serde::Serialize;




#[derive(Debug, Default, Clone, Serialize)]
pub enum Effort {

    Fast,

    #[default]
    Standard,
    
}

