pub mod replacement_rule_paragraph;
pub mod table_paragraph;
pub mod list_paragraph;
pub mod image_paragraph;
pub mod block_quote_paragraph;
pub mod focus_block_paragraph;


use std::fmt::Display;
use thiserror::Error;
use crate::{compiler::{compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile}, utility::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Error, Debug)]
pub enum ParagraphError {
    #[error("creation error")]
    Creation,

    #[error("empty content")]
    Empty
}

pub type ParagraphType = String;


/// # Paragraph
/// 
/// `Paragraph` represents a NMD paragraph, i.e. a portion of text between two blank lines.
/// 
/// Each `Paragraph` has a `raw_content` (which is the raw NMD string) and
/// a `compiled_content` (which is the corresponding compilation result) of the paragraph.
/// 
/// In addiction, each `Paragraph` has an optional `nuid`.    
pub trait Paragraph: std::fmt::Debug + SelfCompile + CompiledTextAccessor + Sync + Send {
    
    fn raw_content(&self) -> &String;

    fn set_raw_content(&mut self, raw_content: String);

    fn nuid(&self) -> Option<&NmdUniqueIdentifier>;

    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>);

    fn is_empty(&self) -> bool {
        self.raw_content().chars().all(|c| c.is_control() || c.is_whitespace())
    }
}


impl Display for dyn Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_content())
    }
}