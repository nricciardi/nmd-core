use crate::{codex::Codex, utility::datastruct::span::Span};

use super::{load_configuration::LoadConfiguration, load_error::LoadError};

pub trait Finder: std::fmt::Debug + Send + Sync {

    /// Find all slices in raw str
    fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<impl Iterator<Item = Span<&'a str>>, LoadError>;

}

pub trait Loader<T>: std::fmt::Debug + Send + Sync {

    /// Actual load a single element from raw str 
    fn load(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<T, LoadError>;

}


/// Contain the logic to find all own content block type in a raw text or actual load a content block
pub trait LoadingRule<T>: Finder + Loader<T> + std::fmt::Debug + Send + Sync {

    // TODO
    // fn find_and_load(&self, raw_text: &str, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<Box<dyn ContentBlock>>, LoadError>;

}