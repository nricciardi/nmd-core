pub mod focus_block_loading_rule;
pub mod image_block_loading_rule;
pub mod list_block_loading_rule;
pub mod metadata_block_wrapper_loading_rule;
pub mod paragraph_loading_rule;
pub mod quote_block_loading_rule;
pub mod replacement_rule_paragraph_loading_rule;
pub mod table_block_loading_rule;


use crate::{codex::Codex, load::{LoadConfiguration, LoadError}, utility::datastruct::span::Span};
use std::fmt::Debug;

use super::content_block::ContentBlock;


/// Contain the logic to find all own content block type in a raw text or actual load a content block
pub trait ContentBlockLoadingRule: Debug + Send + Sync {

    /// Find all content blocks contained in the input raw text 
    fn find(&self, raw_text: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Vec<Span<&str>>, LoadError>;

    // TODO
    fn load(&self, raw_text: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError>;

    // TODO
    // fn find_and_load(&self, raw_text: &str, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<Box<dyn ContentBlock>>, LoadError>;

}