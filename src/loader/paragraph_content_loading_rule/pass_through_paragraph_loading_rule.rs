use super::ParagraphContentLoadingRule;
use crate::{codex::Codex, compiler::compilation_rule::{replacement_rule::ReplacementRule, CompilationRule}, dossier::document::chapter::paragraph::{list_paragraph::ListParagraph, paragraph_content::ParagraphContent, replacement_rule_paragraph::ReplacementRuleParagraph, ParagraphTrait, SimpleParagraphConstructor}, loader::{loader_configuration::LoaderConfiguration, LoadError}};


#[derive(Debug)]
pub enum PassThroughParagraphLoadingRule {
    ListParagraphLoadingRule,
    ExtendedBlockQuoteParagraphLoadingRule,
    ImageParagraphLoadingRule,
    AbridgedImageParagraphLoadingRule,
    MultiImageParagraphLoadingRule,
}

// TODO

// impl PassThroughParagraphLoadingRule {
    
//     pub fn new(compilation_rule: ReplacementRule<String>,) -> Self {
//         Self {
//             compilation_rule,
//         }
//     }
// }



// pub trait PassThroughParagraphLoadingRule {
//     type ParagraphType: SimpleParagraphConstructor;

//     fn generate_paragraph(raw_content: &str) -> Box<dyn ParagraphTrait> where Self: Sized {
//         Self::ParagraphType::new(raw_content)
//     }
// }

impl ParagraphContentLoadingRule for PassThroughParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, _configuration: &LoaderConfiguration) -> Result<Box<dyn ParagraphTrait>, LoadError> {
        match *self {
            Self::ListParagraphLoadingRule => Ok(Box::new(ListParagraph::new(raw_content.to_string()))),
            Self::ExtendedBlockQuoteParagraphLoadingRule => Ok(Box::new(ListParagraph::new(raw_content.to_string()))),
            Self::ImageParagraphLoadingRule => Ok(Box::new(ListParagraph::new(raw_content.to_string()))),
            Self::AbridgedImageParagraphLoadingRule => Ok(Box::new(ListParagraph::new(raw_content.to_string()))),
            Self::MultiImageParagraphLoadingRule => Ok(Box::new(ListParagraph::new(raw_content.to_string()))),
        }
    }
}