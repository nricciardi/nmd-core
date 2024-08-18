pub mod replacement_rule_paragraph_loading_rule;
pub mod table_paragraph_loading_rule;
pub mod image_paragraph_loading_rule;
pub mod list_paragraph_loading_rule;


use crate::{codex::Codex, dossier::document::chapter::paragraph::{paragraph_content::ParagraphContent, ParagraphTrait}};
use super::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError};
use std::{fmt::Debug, sync::{Arc, RwLock}};


pub trait ParagraphContentLoadingRule: Debug + Send + Sync {

    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Box<dyn ParagraphTrait>, LoadError>;

}