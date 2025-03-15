use getset::{Getters, Setters};
use crate::{base_parameter::output_format::OutputFormat, codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, dossier::document::content::Content, mmo::{nmd_unique_identifier::NmdUniqueIdentifier, uri::NUri}, utility::text_utility};

use super::ContentBlock;



#[derive(Debug, Getters, Setters)]
pub struct MetadataBlockWrapper {

    // TODO: remove
    #[getset(set = "pub")]
    raw_content: String,

    #[getset(get = "pub", set = "pub")]
    content: Content,

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(get = "pub", set = "pub")]
    raw_id: Option<String>,

    #[getset(get = "pub", set = "pub")]
    styles: Option<String>,

    #[getset(get = "pub", set = "pub")]
    classes: Option<String>,
}

impl MetadataBlockWrapper {
    
    pub fn new(raw_content: String, content: Content, raw_id: Option<String>, styles: Option<String>, classes: Option<String>,) -> Self {
        Self {
            raw_content,
            content,
            raw_id,
            styles,
            classes,
            nuid: None,
        }
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let nuid_attr = text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref());

        let id_attr;
        if let Some(ref id) = self.raw_id {
            id_attr = format!(
                r#"id="{}""#,
                NUri::of_internal_from_without_sharp(id, compilation_configuration_overlay.document_name().as_ref())?.build()
            );

        } else {

            id_attr = String::new();
        }

        let mut outcome = format!(r#"<div class="{}" style="{}" {} {}>"#, self.classes.as_ref().unwrap_or(&String::new()), self.styles.as_ref().unwrap_or(&String::new()), nuid_attr, id_attr);

        outcome.push_str(&self.content.standard_compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?.content());

        outcome.push_str("</div>");

        Ok(CompilationOutcome::from(outcome))
    }
}

impl Compilable for MetadataBlockWrapper {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
    }
}



impl ContentBlock for MetadataBlockWrapper {

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
    }

    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}


#[cfg(test)]
mod test {
    use crate::{base_parameter::output_format::OutputFormat, codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, load::{load_configuration::LoadConfiguration, loading_rule::content_block_loading_rule::quote_block_loading_rule::QuoteBlockLoadingRule}};



    #[test]
    fn compile() {
        let nmd_text = concat!(
            "> p1a\n",
            "> p1b\n",
            ">\n",
            "> p2a\n"
        ).to_string();
        
        let codex = Codex::of_html();
        let rule = QuoteBlockLoadingRule::new();

        let mut configuration = LoadConfiguration::default();
        configuration.set_document_name(Some(String::from("test")));

        let mut paragraph = rule.load(&nmd_text, &codex, configuration).unwrap();

        let mut cco = CompilationConfigurationOverLay::default();

        cco.set_document_name(Some(String::from("test")));

        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), cco).unwrap();
    }

}


