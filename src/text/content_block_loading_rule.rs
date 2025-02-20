pub mod focus_block_loading_rule;
pub mod image_block_loading_rule;
pub mod list_block_loading_rule;
pub mod metadata_block_wrapper_loading_rule;
pub mod paragraph_loading_rule;
pub mod quote_block_loading_rule;
pub mod replacement_rule_paragraph_loading_rule;
pub mod table_block_loading_rule;


use crate::{codex::Codex, load::{LoadConfiguration, LoadError}};
use std::fmt::Debug;

use super::content_block::ContentBlock;


pub trait ContentBlockLoadingRule: Debug + Send + Sync {

    fn load(&self, raw_content: &str, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<Box<dyn ContentBlock>>, LoadError>;

}