use crate::text_compiler::transformation_rule::replacement_rule::ReplacementRule;



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

// TODO
// impl LoadingRule<Box<dyn ContentBlock>> for ReplacementRuleParagraphLoadingRule {

//     fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<impl Iterator<Item = Span<&'a str>>, LoadError> {
//         todo!()
//     }

//     fn load(&self, raw_content: &str, _codex: &Codex, _configuration: LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError> {
        
//         let compilable_text = CompilableString::from(CompilableStringPart::new_compilable(
//             raw_content.to_string(),
//             ModifiersBucket::None
//         ));

//         Ok(Box::new(ReplacementRuleParagraph::new(
//             raw_content.to_string(),
//             compilable_text,
//             self.replacement_rule.clone()
//         )))
//     }
// }