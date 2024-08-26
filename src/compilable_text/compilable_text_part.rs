use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use crate::codex::modifier::ModifiersBucket;


#[derive(Debug, Clone, Serialize)]
pub enum CompilableTextPartType {
    Fixed,
    Compilable{ incompatible_modifiers: ModifiersBucket },
}

#[derive(Debug, Getters, Setters, MutGetters, Clone, Serialize)]
pub struct CompilableTextPart {

    #[getset(get_mut = "pub", get = "pub", set = "pub")]
    content: String,

    #[getset(get = "pub", set = "pub")]
    part_type: CompilableTextPartType,
}

impl CompilableTextPart {
    
    pub fn new(content: String, part_type: CompilableTextPartType) -> Self {
        Self {
            content,
            part_type,
        }
    }

    pub fn new_fixed(content: String) -> Self {
        Self::new(content, CompilableTextPartType::Fixed)
    }
}
