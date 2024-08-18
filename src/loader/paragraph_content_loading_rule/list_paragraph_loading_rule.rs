use std::sync::{Arc, RwLock};
use super::ParagraphContentLoadingRule;
use crate::{codex::Codex, compiler::compilation_rule::{replacement_rule::ReplacementRule, CompilationRule}, dossier::document::chapter::paragraph::{list_paragraph::ListParagraph, paragraph_content::ParagraphContent, replacement_rule_paragraph::ReplacementRuleParagraph, ParagraphTrait, SimpleParagraphConstructor}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError}};


#[derive(Debug)]
pub struct ListParagraphLoadingRule {
}


impl ListParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }
}


impl ParagraphContentLoadingRule for ListParagraphLoadingRule {
    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Box<dyn ParagraphTrait>, LoadError> {
        Ok(Box::new(ListParagraph::new(raw_content.to_string())))
    }
}