use serde::Serialize;

use crate::{mmo::HashSet, text_compiler::transformation_rule::TextTransformationRuleIdentifier};


#[derive(Debug, Clone, Serialize)]
pub enum TextPart {
    Fixed{ content: String },
    Compilable{ content: String, incompatible_rules: HashSet<TextTransformationRuleIdentifier> },
}

impl TextPart {
    
    pub fn content(&self) -> &String {
        match &self {
            Self::Fixed { content } => content,
            Self::Compilable { content, incompatible_rules: _ } => content,
        }
    }
}