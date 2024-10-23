use getset::{CopyGetters, Getters, Setters};
use serde::Serialize;
use crate::{codex::modifier::ModifiersBucket, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, compilation::{compilation_error::CompilationError, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile}, output_format::OutputFormat, resource::resource_reference::ResourceReference, utility::nmd_unique_identifier::NmdUniqueIdentifier};


pub type HeadingLevel = u32;


#[derive(Debug, Getters, CopyGetters, Setters, Clone, Serialize)]
pub struct Heading {

    #[getset(get_copy = "pub", set = "pub")]
    level: HeadingLevel,

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(set = "pub")]
    compilation_result: Option<CompilableText>,

    #[getset(get = "pub", set = "pub")]
    resource_reference: Option<ResourceReference>,

    #[getset(get = "pub", set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,
}

impl Heading {
    pub fn new(level: HeadingLevel, title: String) -> Self {

        Self {
            level,
            title,
            compilation_result: None,
            resource_reference: None,
            nuid: None,
        }
    }
}

impl CompiledTextAccessor for Heading {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compilation_result.as_ref()
    }
}

impl SelfCompile for Heading {
    fn standard_compile(&mut self, format: &crate::output_format::OutputFormat, codex: &crate::codex::Codex, compilation_configuration: &crate::compilation::compilation_configuration::CompilationConfiguration, compilation_configuration_overlay: crate::compilation::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay) -> Result<(), crate::compilation::compilation_error::CompilationError> {
        
        let document_name = compilation_configuration_overlay.document_name().as_ref();

        if document_name.is_none() {
            return Err(CompilationError::DocumentNameNotFound)
        }

        let document_name = document_name.unwrap();

        let id: ResourceReference = ResourceReference::of_internal_from_without_sharp(&self.title, Some(&document_name))?;

        let mut compiled_title = CompilableText::from(self.title.clone());
        
        compiled_title.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

        let res = match format {
            OutputFormat::Html => {

                let nuid_attr: String;

                if let Some(nuid) = &self.nuid {
                    nuid_attr = format!(r#"data-nuid="{}""#, nuid);
                } else {
                    nuid_attr = String::new();
                }

                let outcome = CompilableText::new(vec![

                    CompilableTextPart::new(
                        format!(r#"<h{} class="heading-{}" id="{}" {}>"#, self.level, self.level, id.build_without_internal_sharp(), nuid_attr),
                        CompilableTextPartType::Fixed
                    ),
                    CompilableTextPart::new(
                        compiled_title.content(),
                        CompilableTextPartType::Compilable{ incompatible_modifiers: ModifiersBucket::None }
                    ),
                    CompilableTextPart::new(
                        format!(r#"</h{}>"#, self.level),
                        CompilableTextPartType::Fixed
                    ),
                ]);

                outcome
            },
        };

        self.set_compilation_result(Some(res));
        self.set_resource_reference(Some(id));

        Ok(())
    }
}