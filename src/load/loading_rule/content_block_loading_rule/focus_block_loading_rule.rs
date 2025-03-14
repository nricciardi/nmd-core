use regex::Regex;
use getset::{Getters, Setters};

use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, dossier::document::chapter::content_block::{focus_block::FocusBlock, ContentBlock}, load::{load_configuration::LoadConfiguration, load_error::LoadError, loading_rule::LoadingRule}, content::Text, utility::datastruct::span::Span};


const DEFAULT_TYPE: &str = "quote";


#[derive(Debug, Getters, Setters, Clone)]
pub struct FocusBlockLoadingRule {

    #[getset(get = "pub", set = "pub")]
    loading_regex: Regex,
}


impl FocusBlockLoadingRule {

    pub fn new(loading_regex: Regex,) -> Self {
        Self {
            loading_regex
        }
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<FocusBlock, LoadError> {

        if let Some(captures) = self.loading_regex.captures(raw_content) {

            let focus_block_type: String;

            if let Some(t) = captures.get(1) {
    
                focus_block_type = t.as_str().to_string().to_lowercase();
    
            } else {
                focus_block_type = String::from(DEFAULT_TYPE);
            }

            if let Some(body) = captures.get(2) {

                Ok(FocusBlock::new(
                    raw_content.to_string(),
                    focus_block_type,
                    Text::load_from_str(body.as_str(), codex, configuration)?,
                ))

            } else {

                return Err(LoadError::ElaborationError(format!("body not found in focus block: {}", raw_content)))
            }

        } else {

            return Err(LoadError::ElaborationError(format!("{} is not a focus block", raw_content)))
        }

    }
}

// TODO
// impl LoadingRule<Box<dyn ContentBlock>> for FocusBlockLoadingRule {

//     fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<dyn Iterator<Item = Span<&'a str>>, LoadError> {
//         Ok(StandardParagraphModifier::FocusBlock.find_spans_iter(raw_str))
//     }

//     fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError> {
        
//         Ok(Box::new(self.inner_load(raw_content, codex, configuration)?))
//     }

// }


#[cfg(test)]
mod test {

    // TODO

    // use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, load::{LoadConfiguration, LoadConfigurationOverLay}};
    // use super::FocusBlockParagraphLoadingRule;


    // #[test]
    // fn load() {
    //     let nmd_text = concat!(
    //         "\n\n",
    //         "::: warning\n",
    //         "new warning\n\n",
    //         "multiline\n",
    //         ":::\n\n",
    //     );

    //     let rule = FocusBlockParagraphLoadingRule::new(StandardParagraphModifier::FocusBlock.modifier_pattern_regex().clone());

    //     let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();    
    
    //     assert_eq!(paragraph.extended_quote_type(), "warning");

    //     assert_eq!(paragraph.content().preamble().len(), 2);
    // }

}