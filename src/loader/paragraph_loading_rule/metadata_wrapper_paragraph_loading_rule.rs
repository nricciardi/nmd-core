use std::sync::Arc;
use getset::{Getters, Setters};
use regex::Regex;
use super::ParagraphLoadingRule;
use crate::{codex::Codex, dossier::document::chapter::paragraph::{metadata_wrapper_paragraph::MetadataWrapperParagraph, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError, Loader}};


pub type StyleElaborationFn = Arc<dyn Sync + Send + Fn(&str, bool) -> (Option<String>, Option<String>)>;


#[derive(Getters, Setters, Clone)]
pub struct MetadataWrapperParagraphLoadingRule {
    
    #[getset(get = "pub", set = "pub")]
    loading_regex: Regex,

    #[getset(get = "pub", set = "pub")]
    content_group: usize,

    #[getset(get = "pub", set = "pub")]
    id_group: Option<usize>,

    #[getset(get = "pub", set = "pub")]
    style_group: Option<usize>,

    #[getset(get = "pub", set = "pub")]
    style_elaboration_fn: Option<StyleElaborationFn>,
}

impl std::fmt::Debug for MetadataWrapperParagraphLoadingRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataWrapperParagraphLoadingRule").field("loading_regex", &self.loading_regex).field("id_group", &self.id_group).field("style_group", &self.style_group).finish()
    }
}


impl MetadataWrapperParagraphLoadingRule {

    pub fn new(loading_regex: Regex, content_group: usize, id_group: Option<usize>, style_group: Option<usize>, style_elaboration_fn: Option<StyleElaborationFn>,) -> Self {
        Self {
            loading_regex,
            content_group,
            id_group,
            style_group,
            style_elaboration_fn,
        }
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<MetadataWrapperParagraph, LoadError> {

        if let Some(captures) = self.loading_regex.captures(raw_content) {

            let mut raw_id: Option<String> = None;
            let mut there_is_id = false;

            if let Some(id_group) = self.id_group {
                if let Some(id) = captures.get(id_group) {
                
                    raw_id = Some(id.as_str().to_string());
                    there_is_id = true;
    
                }
            }

            let mut styles: Option<String> = None;
            let mut classes: Option<String> = None;

            if let Some(style_group) = self.style_group {
                if let Some(style) = captures.get(style_group) {
                
                    (styles, classes) = (self.style_elaboration_fn.as_ref().unwrap())(style.as_str(), there_is_id);
                }
            }

            

            if let Some(body) = captures.get(self.content_group) {

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

    use std::sync::Arc;

    use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, loader::loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, utility::text_utility};
    use super::MetadataWrapperParagraphLoadingRule;


    #[test]
    fn load() {
        let nmd_text = concat!(
            "\n\n",
            "[[\n",
            "this is a paragraphs\n\n",
            "::: warning\n",
            "this is another paragraph\n",
            ":::\n",
            "]]\n",
            "{{\n",
            ".red\n",
            "}}\n"
        );

        let rule = MetadataWrapperParagraphLoadingRule::new(
            StandardParagraphModifier::EmbeddedParagraphStyle.modifier_pattern_regex().clone(),
            1,
            Some(2),
            Some(3),
            Some(Arc::new(|style, _| {
                text_utility::split_styles_and_classes(style)
            }))
        );

        let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();    
    
        assert_eq!(paragraph.raw_id().as_ref(), None);

        assert_eq!(paragraph.styles().as_ref(), None);

        assert_eq!(paragraph.classes().as_ref().unwrap(), "red");

        assert_eq!(paragraph.paragraphs().len(), 2);
    }

}