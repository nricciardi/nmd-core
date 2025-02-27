use std::collections::HashSet;

use crate::{codex::Codex, load::{loading_rule::LoadingRule, LoadConfiguration, LoadError}, text::content_block::{list_block::ListBlock, ContentBlock}, utility::datastruct::span::Span};

use super::ContentBlockLoadingRule;


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


impl LoadingRule<Box<dyn ContentBlock>> for ListBlockLoadingRule {

    fn find(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<HashSet<Span<&str>>, LoadError> {
        todo!()
    }

    fn load(&self, raw_content: &str, _codex: &Codex, _configuration: LoadConfiguration) -> Result<Vec<Box<dyn ContentBlock>>, LoadError> {
        Ok(Box::new(ListBlock::new(raw_content.to_string())))       // TODO: move logic of loading
    }
}