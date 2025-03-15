pub mod list_block;
pub mod focus_block;
pub mod image_block;
pub mod metadata_block_wrapper;
pub mod paragraph;
pub mod replacement_rule_paragraph;
pub mod table_block;
pub mod quote_block;


use thiserror::Error;

use crate::{compilation::compilable::Compilable, mmo::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Error, Debug)]
pub enum ContentBlockError {
    #[error("creation error")]
    Creation,

    #[error("empty content")]
    Empty
}


/// Whatever block of content in the text, e.g. images, textual paragraphs, tables and so on.
pub trait ContentBlock: Compilable + Sync + Send + std::fmt::Debug {

    fn nuid(&self) -> Option<&NmdUniqueIdentifier>;

    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>);
}