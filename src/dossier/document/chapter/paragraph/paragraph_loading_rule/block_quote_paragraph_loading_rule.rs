use once_cell::sync::Lazy;
use regex::Regex;
use super::ParagraphLoadingRule;
use crate::{codex::Codex, content_bundle::ContentBundle, dossier::document::chapter::paragraph::{block_quote_paragraph::ExtendedBlockQuoteParagraph, Paragraph}, load::{LoadConfiguration, LoadConfigurationOverLay, LoadError}, load_block::LoadBlock};


static CHECK_EXTENDED_BLOCK_QUOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m:> \[!(\w*)\])").unwrap());

const DEFAULT_TYPE: &str = "quote";


#[derive(Debug)]
pub struct BlockQuoteParagraphLoadingRule {
}


impl BlockQuoteParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<ExtendedBlockQuoteParagraph, LoadError> {
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

            let c = line[1..].trim_start();

            block_quote_body_content.push_str(c);
            block_quote_body_content.push_str("\n");
        }
        
        let blocks = LoadBlock::load_from_str(&block_quote_body_content, codex, configuration, configuration_overlay.clone())?;

        Ok(ExtendedBlockQuoteParagraph::new(
            raw_content.to_string(),
            extended_block_quote_type,
            ContentBundle::from(blocks),
        ))
    }
}


impl ParagraphLoadingRule for BlockQuoteParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        
        Ok(Box::new(self.inner_load(raw_content, codex, configuration, configuration_overlay.clone())?))
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::Codex, load::{LoadConfiguration, LoadConfigurationOverLay}};

    use super::{BlockQuoteParagraphLoadingRule, DEFAULT_TYPE};


    #[test]
    fn load_implicit_quote() {
        let nmd_text = concat!(
            "> p1a\n",
            "> p1b\n",
            ">\n",
            "> p2a\n"
        );

        let rule = BlockQuoteParagraphLoadingRule::new();

        let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();    
    
        assert_eq!(paragraph.extended_quote_type(), DEFAULT_TYPE);

        assert_eq!(paragraph.content().preamble().len(), 2);
    }

    #[test]
    fn load_explicit_quote() {
        let nmd_text = concat!(
            "> [!IMPORTANT]\n",
            "> p1a\n",
            "> p1b\n",
            ">\n",
            "> p2a\n",
        );

        let rule = BlockQuoteParagraphLoadingRule::new();

        let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();    
    
        assert_eq!(paragraph.extended_quote_type(), "important");

        assert_eq!(paragraph.content().preamble().len(), 2);
    }

}