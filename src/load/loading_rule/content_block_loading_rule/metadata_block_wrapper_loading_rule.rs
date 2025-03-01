use std::sync::Arc;
use getset::{Getters, Setters};
use regex::Regex;

use crate::{codex::Codex, dossier::document::chapter::content_block::{metadata_block_wrapper::MetadataBlockWrapper, ContentBlock}, load::{load_configuration::LoadConfiguration, load_error::LoadError, loading_rule::LoadingRule}, text::Text, utility::datastruct::span::Span};


pub type StyleElaborationFn = Arc<dyn Sync + Send + Fn(&str, bool) -> (Option<String>, Option<String>)>;


// TODO: maybe more general, this allows only style and id

#[derive(Getters, Setters, Clone)]
pub struct MetadataBlockWrapperLoadingRule {
    
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

impl std::fmt::Debug for MetadataBlockWrapperLoadingRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataWrapperParagraphLoadingRule").field("loading_regex", &self.loading_regex).field("id_group", &self.id_group).field("style_group", &self.style_group).finish()
    }
}


impl MetadataBlockWrapperLoadingRule {

    pub fn new(loading_regex: Regex, content_group: usize, id_group: Option<usize>, style_group: Option<usize>, style_elaboration_fn: Option<StyleElaborationFn>,) -> Self {
        Self {
            loading_regex,
            content_group,
            id_group,
            style_group,
            style_elaboration_fn,
        }
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<MetadataBlockWrapper, LoadError> {

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

                Ok(MetadataBlockWrapper::new(
                    raw_content.to_string(),
                    Text::load_from_str(body.as_str(), codex, configuration)?,
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


impl LoadingRule<Box<dyn ContentBlock>> for MetadataBlockWrapperLoadingRule {

    fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<impl Iterator<Item = Span<&'a str>>, LoadError> {
        Ok(self.loading_regex.find_iter(raw_str).map(|m| Span::from(m)))
    }
    
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError> {
        
        Ok(Box::new(self.inner_load(raw_content, codex, configuration)?))
    }
}


#[cfg(test)]
mod test {

    // TODO

    // use std::sync::Arc;

    // use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, load::{LoadConfiguration, LoadConfigurationOverLay}, utility::text_utility};
    // use super::MetadataBlockWrapperLoadingRule;


    // #[test]
    // fn load() {
    //     let nmd_text = concat!(
    //         "\n\n",
    //         "[[\n",
    //         "this is a paragraphs\n\n",
    //         "::: warning\n",
    //         "this is another paragraph\n",
    //         ":::\n",
    //         "]]\n",
    //         "{{\n",
    //         ".red\n",
    //         "}}\n"
    //     );

    //     let rule = MetadataBlockWrapperLoadingRule::new(
    //         StandardParagraphModifier::EmbeddedParagraphStyle.modifier_pattern_regex().clone(),
    //         1,
    //         Some(2),
    //         Some(3),
    //         Some(Arc::new(|style, _| {
    //             text_utility::split_styles_and_classes(style)
    //         }))
    //     );

    //     let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();    
    
    //     assert_eq!(paragraph.raw_id().as_ref(), None);

    //     assert_eq!(paragraph.styles().as_ref(), None);

    //     assert_eq!(paragraph.classes().as_ref().unwrap(), "red");

    //     assert_eq!(paragraph.content().preamble().len(), 2);
    // }

}