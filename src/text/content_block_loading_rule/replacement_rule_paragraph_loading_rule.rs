use crate::{codex::{modifier::ModifiersBucket, Codex}, compilation::compilation_rule::replacement_rule::ReplacementRule, load::{LoadConfiguration, LoadError}, text::{compilable_string::{compilable_string_part::CompilableStringPart, CompilableString}, content_block::ContentBlock}};
use super::ContentBlockLoadingRule;


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

impl ContentBlockLoadingRule for ReplacementRuleParagraphLoadingRule {
    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError> {
        
        let compilable_text = CompilableString::from(CompilableStringPart::new_compilable(
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