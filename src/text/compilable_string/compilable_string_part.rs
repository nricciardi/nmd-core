use serde::Serialize;
use crate::codex::modifier::ModifiersBucket;


#[derive(Debug, Clone, Serialize)]
pub enum CompilableStringPart {
    Fixed{ content: String },
    Compilable{ content: String, incompatible_modifiers: ModifiersBucket },
}

impl CompilableStringPart {
    
    pub fn content(&self) -> &String {
        match &self {
            Self::Fixed { content } => content,
            Self::Compilable { content, incompatible_modifiers: _ } => content,
        }
    }
}
