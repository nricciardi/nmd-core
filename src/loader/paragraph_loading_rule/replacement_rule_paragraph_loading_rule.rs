use super::ParagraphLoadingRule;
use crate::{codex::Codex, compiler::compilation_rule::replacement_rule::ReplacementRule, dossier::document::chapter::paragraph::{replacement_rule_paragraph::ReplacementRuleParagraph, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError}};


#[derive(Debug)]
pub struct ReplacementRuleParagraphLoadingRule {
    compilation_rule: ReplacementRule<String>,
}

impl ReplacementRuleParagraphLoadingRule {
    
    pub fn new(compilation_rule: ReplacementRule<String>,) -> Self {
        Self {
            compilation_rule,
        }
    } 
}

impl ParagraphLoadingRule for ReplacementRuleParagraphLoadingRule {
    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: &LoaderConfiguration, _configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        Ok(Box::new(ReplacementRuleParagraph::new(raw_content.to_string(), Box::new(self.compilation_rule.clone()))))
    }
}