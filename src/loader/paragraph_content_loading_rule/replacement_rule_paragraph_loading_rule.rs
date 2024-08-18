use std::sync::{Arc, RwLock};

use super::ParagraphContentLoadingRule;
use crate::{codex::Codex, compiler::compilation_rule::{replacement_rule::ReplacementRule, CompilationRule}, dossier::document::chapter::paragraph::{paragraph_content::ParagraphContent, replacement_rule_paragraph::ReplacementRuleParagraph, ParagraphTrait}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError}};


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

impl ParagraphContentLoadingRule for ReplacementRuleParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, _configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Box<dyn ParagraphTrait>, LoadError> {
        Ok(Box::new(ReplacementRuleParagraph::new(raw_content.to_string(), Box::new(self.compilation_rule.clone()))))
    }
}