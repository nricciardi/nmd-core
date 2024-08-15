use super::ParagraphContentLoadingRule;
use crate::{codex::Codex, compiler::compilation_rule::{replacement_rule::ReplacementRule, CompilationRule}, dossier::document::chapter::paragraph::{replacement_rule_paragraph::ReplacementRuleParagraph, paragraph_content::ParagraphContent, ParagraphTrait}, loader::{loader_configuration::LoaderConfiguration, LoadError}};


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
    fn load(&self, raw_content: &str, codex: &Codex, _configuration: &LoaderConfiguration) -> Result<Box<dyn ParagraphTrait>, LoadError> {
        Ok(Box::new(ReplacementRuleParagraph::new(raw_content.to_string(), Box::new(self.compilation_rule.clone()))))
    }
}