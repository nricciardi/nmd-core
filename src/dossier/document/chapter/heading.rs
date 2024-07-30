use std::{num::ParseIntError, str::FromStr, sync::{Arc, RwLock}};

use getset::{CopyGetters, Getters, Setters};

use crate::{codex::Codex, compiler::{compilable::{compilation_result_accessor::CompilationResultAccessor, Compilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::{CompilationResult, CompilationResultPart}, Compiler}, output_format::OutputFormat, resource::resource_reference::ResourceReference};

use super::chapter_builder::ChapterBuilderError;


pub type HeadingLevel = u32;


#[derive(Debug, Getters, CopyGetters, Setters, Clone)]
pub struct Heading {

    #[getset(get_copy = "pub", set = "pub")]
    level: HeadingLevel,

    #[getset(get = "pub", set = "pub")]
    title: String,

    compilation_result: Option<CompilationResult>,

    #[getset(get = "pub", set = "pub")]
    resource_reference: Option<ResourceReference>,
}

impl Heading {
    pub fn new(level: HeadingLevel, title: String) -> Self {

        Self {
            level,
            title,
            compilation_result: None,
            resource_reference: None
        }
    }
}

impl Compilable for Heading {
    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        let pc = compilation_configuration.read().unwrap();

        let document_name = pc.metadata().document_name().as_ref().unwrap();

        let id: ResourceReference = ResourceReference::of_internal_from_without_sharp(&self.title, Some(&document_name))?;

        let parsed_title = Compiler::compile_str(&*codex, &self.title, Arc::clone(&compilation_configuration), compilation_configuration_overlay.clone())?;

        let outcome = CompilationResult::new(vec![
            CompilationResultPart::Fixed { content: format!(r#"<h{} class="heading-{}" id="{}">"#, self.level, self.level, id.build_without_internal_sharp()) },
            CompilationResultPart::Mutable { content: parsed_title.content() },
            CompilationResultPart::Fixed { content: format!(r#"</h{}>"#, self.level) },
        ]);
        
        self.compilation_result = Some(outcome);
        self.resource_reference = Some(id);

        Ok(())
    }
}


impl CompilationResultAccessor for Heading {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compilation_result
    }
}
