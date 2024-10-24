use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use getset::{Getters, Setters};
use super::ParagraphLoadingRule;
use crate::{codex::Codex, dossier::document::chapter::paragraph::{focus_block_paragraph::FocusBlockParagraph, Paragraph}, load::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError, Loader}};


const DEFAULT_TYPE: &str = "quote";


#[derive(Debug, Getters, Setters, Clone)]
pub struct FocusBlockParagraphLoadingRule {

    #[getset(get = "pub", set = "pub")]
    loading_regex: Regex,
}


impl FocusBlockParagraphLoadingRule {

    pub fn new(loading_regex: Regex,) -> Self {
        Self {
            loading_regex
        }
    }

    fn inner_load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<FocusBlockParagraph, LoadError> {

        if let Some(captures) = self.loading_regex.captures(raw_content) {

            let focus_block_type: String;

            if let Some(t) = captures.get(1) {
    
                focus_block_type = t.as_str().to_string().to_lowercase();
    
            } else {
                focus_block_type = String::from(DEFAULT_TYPE);
            }

            if let Some(body) = captures.get(2) {

                let paragraph_blocks = Loader::load_paragraphs_from_str_with_workaround(body.as_str(), codex, configuration, configuration_overlay.clone())?;
        
                let paragraphs: Vec<Box<dyn Paragraph>> = paragraph_blocks.into_par_iter().map(|block| TryInto::<Box<dyn Paragraph>>::try_into(block).unwrap()).collect();
                
                Ok(FocusBlockParagraph::new(
                    raw_content.to_string(),
                    focus_block_type,
                    paragraphs,
                ))

            } else {

                return Err(LoadError::ElaborationError(format!("body not found in focus block: {}", raw_content)))
            }

        } else {

            return Err(LoadError::ElaborationError(format!("{} is not a focus block", raw_content)))
        }

    }
}


impl ParagraphLoadingRule for FocusBlockParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        
        Ok(Box::new(self.inner_load(raw_content, codex, configuration, configuration_overlay.clone())?))
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, load::loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}};
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

        let rule = FocusBlockParagraphLoadingRule::new(StandardParagraphModifier::FocusBlock.modifier_pattern_regex().clone());

        let paragraph = rule.inner_load(&nmd_text, &Codex::of_html(), &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();    
    
        assert_eq!(paragraph.extended_quote_type(), "warning");

        assert_eq!(paragraph.paragraphs().len(), 2);
    }

}