use getset::{CopyGetters, Getters};
use serde::Serialize;

use crate::{mmo::{bucket::Bucket}, text_compiler::transformation_rule::TextTransformationRuleIdentifier};

use super::text_error::TextError;


#[derive(Debug, Clone, Serialize)]
pub enum TextPart {
    Fixed{ content: String },
    Compilable{ content: String, incompatible_rules: Bucket<TextTransformationRuleIdentifier> },
}

impl TextPart {
    
    pub fn content(&self) -> &String {
        match &self {
            Self::Fixed { content } => content,
            Self::Compilable { content, incompatible_rules: _ } => content,
        }
    }

    pub fn content_mut(&mut self) -> &mut String {
        match self {
            Self::Fixed { content } => content,
            Self::Compilable { content, incompatible_rules: _ } => content,
        }
    }
}


#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct TextPartRef<'a> {

    #[getset(get_copy = "pub")]
    index: usize,
    
    #[getset(get = "pub")]
    part: &'a TextPart,
}

impl<'a> TextPartRef<'a> {
    pub fn new(index: usize, part: &'a TextPart) -> Self {
        Self {
            index,
            part,
        }
    }
}


#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct CompatibleTextParts<'a> {

    #[getset(get = "pub")]
    parts: Vec<TextPartRef<'a>>,
}


impl<'a> FromIterator<TextPartRef<'a>> for CompatibleTextParts<'a> {
    fn from_iter<T: IntoIterator<Item = TextPartRef<'a>>>(iter: T) -> Self {

        Self {
            parts: iter.into_iter().collect()
        }
    }
}

impl<'a> ToString for CompatibleTextParts<'a> {
    fn to_string(&self) -> String {
        self.parts.iter()
            .map(|part_ref| part_ref.part().content().as_str())
            .collect()
    }
}

