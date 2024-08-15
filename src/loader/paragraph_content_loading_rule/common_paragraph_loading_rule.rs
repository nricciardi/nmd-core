use super::ParagraphContentLoadingRule;
use crate::{codex::Codex, dossier::document::chapter::paragraph::{common_paragraph::CommonParagraph, paragraph_content::ParagraphContent, ParagraphTrait}, loader::{loader_configuration::LoaderConfiguration, LoadError}};


#[derive(Debug)]
pub struct CommonParagraphLoadingRule {

}

impl ParagraphContentLoadingRule for CommonParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, _configuration: &LoaderConfiguration) -> Result<Box<dyn ParagraphTrait>, LoadError> {
        Ok(Box::new(CommonParagraph::new(raw_content.to_string())))
    }
}