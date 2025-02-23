use std::collections::HashSet;

use crate::{codex::Codex, utility::datastruct::span::Span};

use super::{load_configuration::LoadConfiguration, load_error::LoadError};

/// Contain the logic to find all own content block type in a raw text or actual load a content block
pub trait LoadingRule<T>: std::fmt::Debug + Send + Sync {

    /// Find all slices in raw str
    fn find(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<HashSet<Span<&str>>, LoadError>;

    /// Actual load a single element from raw str 
    fn load(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<T, LoadError>;

    // TODO
    // fn find_and_load(&self, raw_text: &str, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<Box<dyn ContentBlock>>, LoadError>;

}