use once_cell::sync::Lazy;
use regex::Regex;
use crate::{codex::{modifier::{constants::{MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, NEW_LINE_PATTERN}, standard_paragraph_modifier::StandardParagraphModifier}, Codex}, load::{load_configuration::LoadConfiguration, load_error::LoadError, loading_rule::{Finder, Loader, LoadingRule}}, text::content_block::{paragraph::Paragraph, ContentBlock}, utility::datastruct::span::Span};


static EXTRACT_PARAGRAPH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("{}{}{}", MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, NEW_LINE_PATTERN, NEW_LINE_PATTERN)).unwrap());


#[derive(Debug)]
pub struct ParagraphLoadingRule {
}


impl ParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }

    fn inner_load(&self, raw_content: &str, _codex: &Codex, _configuration: LoadConfiguration) -> Vec<Paragraph> {
        
        let mut raw_content = String::from(raw_content);

        while !raw_content.ends_with("\n\n") {
            raw_content.push_str("\n");
        }

        let mut paragraphs: Vec<Paragraph> = Vec::new(); 

        for m in EXTRACT_PARAGRAPH_REGEX.find_iter(&raw_content) {
            paragraphs.push(
                Paragraph::new(m.as_str().to_string())
            );
        }

        paragraphs
    }
}

impl Finder for ParagraphLoadingRule {

    fn find<'a>(&self, raw_str: &'a str, codex: &Codex, configuration: &LoadConfiguration) -> Result<impl Iterator<Item = Span<&'a str>>, LoadError> {
        Ok(StandardParagraphModifier::CommonParagraph.find_spans_iter(raw_str))
    }
    
}

impl Loader<Box<dyn ContentBlock>> for ParagraphLoadingRule {

    // TODO
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Box<dyn ContentBlock>, LoadError> {
            
        // Ok(self.inner_load(raw_content, codex, configuration).into_iter().map(|p| {
        //     Box::new(p) as Box<dyn ContentBlock>
        // }).collect())
        
        todo!()
    }

}

impl LoadingRule<Box<dyn ContentBlock>> for ParagraphLoadingRule {
    
}


#[cfg(test)]
mod test {

    // TODO

    // use crate::{codex::Codex, load::{LoadConfiguration, LoadConfigurationOverLay}};
    // use super::CommonParagraphLoadingRule;


    // #[test]
    // fn load_common_paragraph() {
    //     let nmd_text = concat!(
    //         "a\n",
    //         "b\n",
    //         "\n",
    //         "c\n",
    //         "\n\n\n\n",
    //         "d",
    //     );

    //     let rule = CommonParagraphLoadingRule::new();

    //     let paragraphs = rule.inner_load(&nmd_text, &Codex::of_html(), &LoadConfiguration::default(), LoadConfigurationOverLay::default());    
    
    //     assert_eq!(paragraphs.len(), 3);
    // }


}