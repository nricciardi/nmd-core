use super::ParagraphLoadingRule;
use crate::{codex::Codex, dossier::document::chapter::paragraph::{list_paragraph::ListParagraph, Paragraph}, load::{LoadConfiguration, LoadConfigurationOverLay, LoadError}};


#[derive(Debug)]
pub struct ListParagraphLoadingRule {
}


impl ListParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }
}


impl ParagraphLoadingRule for ListParagraphLoadingRule {
    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: &LoadConfiguration, _configuration_overlay: LoadConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        Ok(Box::new(ListParagraph::new(raw_content.to_string())))
    }
}