use super::ParagraphLoadingRule;
use crate::{codex::{modifier::ModifiersBucket, Codex}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::compilation_rule::replacement_rule::ReplacementRule, dossier::document::chapter::paragraph::{replacement_rule_paragraph::ReplacementRuleParagraph, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError}};


#[derive(Debug)]
pub struct ReplacementRuleParagraphLoadingRule {
    replacement_rule: ReplacementRule,
}

impl ReplacementRuleParagraphLoadingRule {
    
    pub fn new(replacement_rule: ReplacementRule,) -> Self {
        Self {
            replacement_rule,
        }
    } 
}

impl ParagraphLoadingRule for ReplacementRuleParagraphLoadingRule {
    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: &LoaderConfiguration, _configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        
        let compilable_text = CompilableText::from(CompilableTextPart::new_compilable(
            raw_content.to_string(),
            ModifiersBucket::None
        ));

        Ok(Box::new(ReplacementRuleParagraph::new(
            raw_content.to_string(),
            compilable_text,
            self.replacement_rule.clone()
        )))
    }
}