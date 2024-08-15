pub mod paragraph_content;
pub mod common_paragraph;
pub mod table_paragraph;


use std::fmt::Display;
use getset::{Getters, Setters};
use paragraph_content::ParagraphContent;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::{codex::modifier::standard_paragraph_modifier::StandardParagraphModifier, compiler::{compilable::{Compilable, CompilableContent}, compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor, self_compile::SelfCompile}, utility::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Error, Debug)]
pub enum ParagraphError {
    #[error("creation error")]
    Creation,

    #[error("empty content")]
    Empty
}

pub type ParagraphType = String;


pub trait ParagraphTrait: std::fmt::Debug + SelfCompile + CompilationResultAccessor + Sync + Send {
    
    fn raw_content(&self) -> &String;

    fn set_raw_content(&mut self, raw_content: String);

    fn nuid(&self) -> &Option<NmdUniqueIdentifier>;

    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>);

    fn is_empty(&self) -> bool {
        todo!()     // TODO: check if there is only new lines or tabs or spaces
    }
}




/// # Paragraph
/// 
/// `Paragraph` represents a NMD paragraph, i.e. a portion of text between two blank lines.
/// 
/// Each `Paragraph` has a `raw_content` (which is the raw NMD string), 
/// the `content` (which is the loaded content and it is a dynamic content) and
/// a `compiled_content` (which is the corresponding compilation result) of the paragraph.
/// 
/// In addiction, each `Paragraph` has `paragraph_type` field to perform fast compilation and an optional `nuid`.    
#[derive(Debug, Getters, Setters, Serialize)]
pub struct Paragraph {

    #[getset(get = "pub", set = "pub")]
    raw_content: String,

    #[getset(get = "pub", set = "pub")]
    #[serde(skip)]
    content: Box<dyn ParagraphContent>,

    #[getset(set = "pub")]
    compiled_content: Option<CompilationResult>,

    #[getset(get = "pub", set = "pub")]
    paragraph_type: ParagraphType,

    #[getset(get = "pub", set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,
}

impl Paragraph {

    pub fn new(raw_content: String, content: Box<dyn ParagraphContent>, paragraph_type: ParagraphType) -> Self {
        Self {
            raw_content,
            content,
            paragraph_type,
            compiled_content: None,
            nuid: None,
        }
    }

    pub fn contains_only_newlines(&self) -> bool {
        self.raw_content.chars().all(|c| c == '\n' || c == '\r')
    }
}

impl CompilationResultAccessor for Paragraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compiled_content
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_content)
    }
}

// impl Compilable for Paragraph {
//     fn compilable_content(&self) -> &CompilableContent {
//         &self.content
//     }

//     fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
//         self.nuid.as_ref()
//     }
// }