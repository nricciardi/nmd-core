use once_cell::sync::Lazy;
use regex::Regex;

use super::ParagraphLoadingRule;
use crate::{codex::{modifier::constants::NEW_LINE, Codex}, dossier::document::chapter::paragraph::{block_quote_paragraph::ExtendedBlockQuoteParagraph, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError, Loader}};


static CHECK_EXTENDED_BLOCK_QUOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^(?m:^> \[!(.*)\]))").unwrap());

const DEFAULT_TYPE: &str = "quote";


#[derive(Debug)]
pub struct BlockQuoteParagraphLoadingRule {
}


impl BlockQuoteParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<ExtendedBlockQuoteParagraph, LoadError> {
        let mut lines: Vec<&str> = raw_content.trim().lines().collect();

        let extended_block_quote_type: String;

        if let Some(t) = CHECK_EXTENDED_BLOCK_QUOTE_REGEX.captures(raw_content) {

            extended_block_quote_type = t.get(1).unwrap().as_str().to_string().to_lowercase();

            lines.remove(0);

        } else {
            extended_block_quote_type = String::from(DEFAULT_TYPE);
        }
        
        let mut block_quote_body_content = String::new(); 

        for line in lines {
            if !line.starts_with(">") {
                if configuration.strict_focus_block_check() {

                    log::warn!("invalid line in focus (quote) block: {}", line);
                    continue;

                } else {

                    // log::error!("invalid line in focus (quote) block: {}", line);
                    return Err(LoadError::ElaborationError("invalid line in focus (quote) block".to_string()))
                }
            }

            let mut c = String::from(line[1..].trim_start());

            if c.is_empty() {
                c = format!("{}{}", NEW_LINE, NEW_LINE);
            }

            block_quote_body_content.push_str(c.as_str());
        }
        
        let paragraphs: Vec<Box<dyn Paragraph>> = Loader::load_paragraphs_from_str(&block_quote_body_content, codex, configuration, configuration_overlay.clone())?;
        
        Ok(ExtendedBlockQuoteParagraph::new(
            raw_content.to_string(),
            extended_block_quote_type,
            paragraphs,
        ))
    }
}


impl ParagraphLoadingRule for BlockQuoteParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        
        Ok(Box::new(self.inner_load(raw_content, codex, configuration, configuration_overlay.clone())?))
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::Codex, loader::loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}};
    use super::{BlockQuoteParagraphLoadingRule, DEFAULT_TYPE};


    #[test]
    fn load() {
        let nmd_text = concat!(
            "> p1a\n",
            "> p1b\n",
            ">\n",
            "> p2a\n"
        );

        let rule = BlockQuoteParagraphLoadingRule::new();

        let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();    
    
        assert_eq!(paragraph.extended_quote_type(), DEFAULT_TYPE);

        assert_eq!(paragraph.paragraphs().len(), 2);
    }

}