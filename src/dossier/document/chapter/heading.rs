use getset::{Getters, Setters};
use serde::Serialize;
use crate::{codex::{modifier::ModifiersBucket, Codex}, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, mmo::uri::NUri, output_format::OutputFormat, text::compilable_string::{compilable_string_part::CompilableStringPart, CompilableString}, utility::datastruct::nmd_unique_identifier::NmdUniqueIdentifier};


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

        let mut compiled_title = CompilableString::from(self.title.clone());
        
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

                let outcome = CompilableString::new(vec![

                    CompilableStringPart::Fixed(
                        format!(r#"<h{} class="heading-{}" id="{}" {}>"#, level, level, id.build_without_internal_sharp(), nuid_attr),
                    ),
                    CompilableStringPart::Compilable(
                        compiled_title.content(),
                        ModifiersBucket::None
                    ),
                    CompilableStringPart::Fixed(
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