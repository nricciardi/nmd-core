use getset::{Getters, Setters};
use serde::Serialize;
use crate::{base_parameter::output_format::OutputFormat, codex::{modifier::ModifiersBucket, Codex}, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, mmo::{nmd_unique_identifier::NmdUniqueIdentifier, uri::NUri}, text_compiler::text::{text_part::TextPart, Text}};


#[derive(Debug, Clone, Serialize)]
pub enum HeadingLevel {
    Explicit(u32),
    Minor,
    Major,
    Same
}


#[derive(Debug, Getters, Setters, Clone, Serialize)]
pub struct Heading {

    #[getset(get = "pub", set = "pub")]
    level: HeadingLevel,

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get = "pub", set = "pub")]
    nuri: Option<NUri>,

    #[getset(get = "pub", set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,
}

impl Heading {
    pub fn new(level: HeadingLevel, title: String) -> Self {

        Self {
            level,
            title,
            nuri: None,
            nuid: None,
        }
    }
}

impl Compilable for Heading {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let document_name = compilation_configuration_overlay.document_name().as_ref();

        if document_name.is_none() {
            return Err(CompilationError::DocumentNameNotFound)
        }

        let document_name = document_name.unwrap();

        let id: NUri = NUri::of_internal_from_without_sharp(&self.title, Some(&document_name))?;

        let mut compiled_title = Text::from(self.title.clone());
        
        compiled_title.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

        let res = match format {
            OutputFormat::Html => {

                let nuid_attr: String;

                if let Some(nuid) = &self.nuid {
                    nuid_attr = format!(r#"data-nuid="{}""#, nuid);
                } else {
                    nuid_attr = String::new();
                }

                let level = match self.level {
                    HeadingLevel::Explicit(l) => l,
                    _ => return Err(CompilationError::HeadingLevelNotInferable(self.title.to_string()))
                };

                let outcome = Text::new(vec![

                    TextPart::Fixed(
                        format!(r#"<h{} class="heading-{}" id="{}" {}>"#, level, level, id.build_without_internal_sharp(), nuid_attr),
                    ),
                    TextPart::Compilable(
                        compiled_title.content(),
                        ModifiersBucket::None
                    ),
                    TextPart::Fixed(
                        format!(r#"</h{}>"#, level),
                    ),
                ]);

                outcome
            },
        };

        self.set_resource_reference(Some(id));      // TODO: is pointless?

        Ok(CompilationOutcome::from(&res))
    }
}