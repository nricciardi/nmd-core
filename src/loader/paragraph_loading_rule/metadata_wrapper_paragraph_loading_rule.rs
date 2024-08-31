use std::sync::Arc;

use regex::Regex;
use super::ParagraphLoadingRule;
use crate::{codex::Codex, dossier::document::chapter::paragraph::{metadata_wrapper_paragraph::MetadataWrapperParagraph, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError, Loader}};


const DEFAULT_TYPE: &str = "quote";


pub type StyleElaborationFn = Arc<dyn Sync + Send + Fn(&str) -> (Option<String>, Option<String>)>;


pub struct MetadataWrapperParagraphLoadingRule {
    
    loading_regex: Regex,

    id_group: Option<usize>,

    style_group: Option<usize>,

    style_elaboration_fn: StyleElaborationFn,
}

impl std::fmt::Debug for MetadataWrapperParagraphLoadingRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataWrapperParagraphLoadingRule").field("loading_regex", &self.loading_regex).field("id_group", &self.id_group).field("style_group", &self.style_group).finish()
    }
}


impl MetadataWrapperParagraphLoadingRule {

    pub fn new(loading_regex: Regex, id_group: Option<usize>, style_group: Option<usize>, style_elaboration_fn: StyleElaborationFn,) -> Self {
        Self {
            loading_regex,
            id_group,
            style_group,
            style_elaboration_fn,
        }
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<MetadataWrapperParagraph, LoadError> {

        if let Some(captures) = self.loading_regex.captures(raw_content) {

            let mut raw_id: Option<String> = None;

            if let Some(id_group) = self.id_group {
                if let Some(id) = captures.get(id_group) {
                
                    raw_id = Some(id.as_str().to_string());
    
                }
            }

            let mut styles: Option<String> = None;
            let mut classes: Option<String> = None;

            if let Some(style_group) = self.id_group {
                if let Some(style) = captures.get(style_group) {
                
                    (styles, classes) = (self.style_elaboration_fn)(style.as_str());
                }
            }

            

            if let Some(body) = captures.get(2) {

                let paragraphs: Vec<Box<dyn Paragraph>> = Loader::load_paragraphs_from_str(body.as_str(), codex, configuration, configuration_overlay.clone())?;
        
                Ok(MetadataWrapperParagraph::new(
                    raw_content.to_string(),
                    paragraphs,
                    raw_id,
                    styles,
                    classes,
                ))

            } else {

                return Err(LoadError::ElaborationError(format!("body not found in focus block: {}", raw_content)))
            }

        } else {

            return Err(LoadError::ElaborationError(format!("{} doesn't have metadata", raw_content)))
        }

    }
}


impl ParagraphLoadingRule for MetadataWrapperParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        
        Ok(Box::new(self.inner_load(raw_content, codex, configuration, configuration_overlay.clone())?))
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, loader::loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}};
    use super::FocusBlockParagraphLoadingRule;


    #[test]
    fn load() {
        let nmd_text = concat!(
            "\n\n",
            "::: warning\n",
            "new warning\n\n",
            "multiline\n",
            ":::\n\n",
        );

        let rule = FocusBlockParagraphLoadingRule::new(StandardParagraphModifier::FocusBlock.modifier_pattern_regex_with_paragraph_separator().clone());

        let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();    
    
        assert_eq!(paragraph.extended_quote_type(), "warning");

        assert_eq!(paragraph.paragraphs().len(), 2);
    }

}