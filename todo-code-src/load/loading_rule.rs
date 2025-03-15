pub mod content_block_loading_rule;
pub mod chapter_header_loading_rule;

use crate::{codex::Codex, utility::datastruct::span::Span};
use super::{load_configuration::LoadConfiguration, load_error::LoadError};


/// Contain the logic to find all own content block type in a raw text or actual load a content block
pub trait LoadingRule<T>: std::fmt::Debug + Send + Sync {

    /// Find all slices in raw str
    fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<dyn Iterator<Item = Span<&'a str>>, LoadError>;


    /// Actual load a single element from raw str 
    fn load(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<T, LoadError>;

    // TODO
    // fn find_and_load(&self, raw_text: &str, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<Box<dyn ContentBlock>>, LoadError>;

}