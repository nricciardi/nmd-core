pub mod replacement_rule_paragraph_loading_rule;
pub mod table_paragraph_loading_rule;
pub mod pass_through_paragraph_loading_rule;


use crate::{codex::Codex, dossier::document::chapter::paragraph::{paragraph_content::ParagraphContent, ParagraphTrait}};
use super::{loader_configuration::LoaderConfiguration, LoadError};
use std::fmt::Debug;


pub trait ParagraphContentLoadingRule: Debug + Send + Sync {

    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Box<dyn ParagraphTrait>, LoadError>;

}