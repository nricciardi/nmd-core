use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, load::{load_configuration::LoadConfiguration, load_error::LoadError, loading_rule::{Finder, Loader, LoadingRule}}, text::content_block::{list_block::ListBlock, ContentBlock}, utility::datastruct::span::Span};


#[derive(Debug)]
pub struct ListBlockLoadingRule {
    strict_check_if_only_one_list: bool
}


impl ListBlockLoadingRule {

    pub fn new(strict_check_if_only_one_list: bool) -> Self {
        Self {
            strict_check_if_only_one_list
        }
    }
}

impl Finder for ListBlockLoadingRule {

    fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<impl Iterator<Item = Span<&'a str>>, LoadError> {
        Ok(StandardParagraphModifier::List.find_spans_iter(raw_str))
    }
    
}

impl Loader<Box<dyn ContentBlock>> for ListBlockLoadingRule {

    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: &LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError> {
        Ok(Box::new(ListBlock::new(raw_content.to_string())))       // TODO: move logic of loading
    }

}


impl LoadingRule<Box<dyn ContentBlock>> for ListBlockLoadingRule {

}