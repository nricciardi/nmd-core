use getset::{Getters, Setters};
use crate::{codex::Codex, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilable::Compilable}, output_format::OutputFormat, utility::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};
use super::Paragraph;



#[derive(Debug, Getters, Setters)]
pub struct FocusBlockParagraph {

    #[getset(get = "pub", set = "pub")]
    paragraphs: Vec<Box<dyn Paragraph>>,
    
    #[getset(get = "pub", set = "pub")]
    extended_quote_type: String,

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    #[getset(set = "pub")]
    compiled_content: Option<CompilableText>,
}

impl FocusBlockParagraph {
    
    pub fn new(raw_content: String, extended_quote_type: String, paragraphs: Vec<Box<dyn Paragraph>>) -> Self {
        Self {
            raw_content,
            paragraphs,
            extended_quote_type,
            nuid: None,
            compiled_content: None
        }
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        let mut compilation_result = CompilableText::new_empty();

        let mut content = format!(r#"<div class="focus-block focus-block-{}" {}>"#, self.extended_quote_type, text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref()));
        content.push_str(&format!(r#"<div class="focus-block-title focus-block-{}-title"></div>"#, self.extended_quote_type));
        content.push_str(&format!(r#"<div class="focus-block-description focus-block-{}-description">"#, self.extended_quote_type));

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(content));

        for paragraph in self.paragraphs.iter_mut() {
            paragraph.standard_compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

            compilation_result.parts_mut().append(&mut paragraph.compiled_text().unwrap().clone().parts_mut());
        }

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from("</div></div>")));

        self.compiled_content = Some(compilation_result);

        Ok(())
    }
}

impl Compilable for FocusBlockParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay),
        }
    }
}


impl CompiledTextAccessor for FocusBlockParagraph {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compiled_content.as_ref()
    }
}

impl Paragraph for FocusBlockParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}


#[cfg(test)]
mod test {

    use crate::{codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, dossier::document::chapter::paragraph::Paragraph, load::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, Loader}, output_format::OutputFormat};

    fn load_and_compile_html(content: &str, expected_n: usize) -> String {
        
        let codex = Codex::of_html();
    
        let paragraphs = Loader::load_paragraphs_from_str_with_workaround(content, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();

        assert_eq!(paragraphs.len(), expected_n);

        let mut compiled_content = String::new();

        let cc = CompilationConfiguration::default();
        let cco = CompilationConfigurationOverLay::default();

        for paragraph in paragraphs {

            let mut paragraph: Box<dyn Paragraph> = paragraph.try_into().unwrap();

            paragraph.compile(&OutputFormat::Html, &codex, &cc, cco.clone()).unwrap();
        
            compiled_content.push_str(&paragraph.compiled_text().unwrap().content());
        }

        compiled_content
    }

    #[test]
    fn two_focus_block() {

        let nmd_text = concat!(
            "\n\n",
            "::: warning\n",
            "new warning\n\n",
            "multiline\n",
            ":::\n\n",
            "\n",
            "::: important\n",
            "new important\n\n",
            "multiline\n",
            ":::\n\n",
        );
        
        let compiled_content = load_and_compile_html(nmd_text, 2);

        assert_eq!(compiled_content, concat!(
            r#"<div class="focus-block focus-block-warning" ><div class="focus-block-title focus-block-warning-title"></div><div class="focus-block-description focus-block-warning-description"><p class="paragraph">new warning</p><p class="paragraph">multiline</p></div></div>"#,
            r#"<div class="focus-block focus-block-important" ><div class="focus-block-title focus-block-important-title"></div><div class="focus-block-description focus-block-important-description"><p class="paragraph">new important</p><p class="paragraph">multiline</p></div></div>"#,
        ));
    }

}


