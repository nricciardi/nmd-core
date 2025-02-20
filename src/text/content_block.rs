pub mod list_block;
pub mod focus_block;
pub mod image_block;
pub mod metadata_block_wrapper;
pub mod paragraph;
pub mod replacement_rule_paragraph;
pub mod table_block;
pub mod quote_block;


use std::fmt::Display;
use thiserror::Error;
use crate::{compilation::compilable::Compilable, utility::datastruct::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Error, Debug)]
pub enum ContentBlockError {
    #[error("creation error")]
    Creation,

    #[error("empty content")]
    Empty
}

// pub type ParagraphType = String;     // TODO: remove


pub trait ContentBlock: std::fmt::Debug + Compilable + Sync + Send {

    fn nuid(&self) -> Option<&NmdUniqueIdentifier>;

    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>);
}


impl Display for dyn ContentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_content())
    }
}