use once_cell::sync::Lazy;
use regex::Regex;
use super::MultiParagraphLoadingRule;
use crate::{codex::{modifier::constants::{MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, NEW_LINE_PATTERN}, Codex}, dossier::document::chapter::paragraph::{common_paragraph::CommonParagraph, Paragraph}, load::{LoadConfiguration, LoadConfigurationOverLay, LoadError}};


static EXTRACT_PARAGRAPH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("{}{}{}", MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, NEW_LINE_PATTERN, NEW_LINE_PATTERN)).unwrap());


#[derive(Debug)]
pub struct CommonParagraphLoadingRule {
}


impl CommonParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }

    fn inner_load(&self, raw_content: &str, _codex: &Codex, _configuration: &LoadConfiguration, _configuration_overlay: LoadConfigurationOverLay) -> Vec<CommonParagraph> {
        
        let mut raw_content = String::from(raw_content);

        while !raw_content.ends_with("\n\n") {
            raw_content.push_str("\n");
        }

        let mut paragraphs: Vec<CommonParagraph> = Vec::new(); 

        for m in EXTRACT_PARAGRAPH_REGEX.find_iter(&raw_content) {
            paragraphs.push(
                CommonParagraph::new(m.as_str().to_string())
            );
        }

        paragraphs
    }
}


impl MultiParagraphLoadingRule for CommonParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Vec<Box<dyn Paragraph>>, LoadError> {
        
        Ok(self.inner_load(raw_content, codex, configuration, configuration_overlay).into_iter().map(|p| {
            Box::new(p) as Box<dyn Paragraph>
        }).collect())
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::Codex, load::{LoadConfiguration, LoadConfigurationOverLay}};
    use super::CommonParagraphLoadingRule;


    #[test]
    fn load_common_paragraph() {
        let nmd_text = concat!(
            "a\n",
            "b\n",
            "\n",
            "c\n",
            "\n\n\n\n",
            "d",
        );

        let rule = CommonParagraphLoadingRule::new();

        let paragraphs = rule.inner_load(&nmd_text, &Codex::of_html(), &LoadConfiguration::default(), LoadConfigurationOverLay::default());    
    
        assert_eq!(paragraphs.len(), 2);
    }


}