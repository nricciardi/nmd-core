pub mod replacement_rule_paragraph_loading_rule;
pub mod table_paragraph_loading_rule;
pub mod image_paragraph_loading_rule;
pub mod list_paragraph_loading_rule;
pub mod block_quote_paragraph_loading_rule;
pub mod focus_block_paragraph_loading_rule;
pub mod metadata_wrapper_paragraph_loading_rule;
pub mod common_paragraph_loading_rule;


use crate::{codex::Codex, dossier::document::chapter::paragraph::Paragraph, load::{LoadConfiguration, LoadConfigurationOverLay, LoadError}};
use std::fmt::Debug;


pub trait ParagraphLoadingRule: Debug + Send + Sync {

    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError>;

}

pub trait MultiParagraphLoadingRule: Debug + Send + Sync {

    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Vec<Box<dyn Paragraph>>, LoadError>;

}