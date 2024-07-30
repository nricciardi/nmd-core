use thiserror::Error;

use crate::{compiler::{compilable::compilation_result_accessor::CompilationResultAccessor, compilation_error::CompilationError}, resource::ResourceError};

use self::{html_assembler::HtmlAssembler, assembler_configuration::AssemblerConfiguration};

use super::{artifact::{Artifact, ArtifactError}, bibliography::Bibliography, dossier::{Document, Dossier}, output_format::OutputFormat, table_of_contents::TableOfContents};

pub mod html_assembler;
pub mod assembler_configuration;


#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("too few elements to assemble: {0}")]
    TooFewElements(String),

    #[error(transparent)]
    ArtifactError(#[from] ArtifactError),

    #[error(transparent)]
    CompilationError(#[from] CompilationError),

    #[error("compiled content not found")]
    CompiledContentNotFound,

    #[error(transparent)]
    ResourceError(#[from] ResourceError),
}

pub trait Assembler {

    fn configuration(&self) -> &AssemblerConfiguration;

    fn set_configuration(&mut self, configuration: AssemblerConfiguration);

    fn assemble_dossier(&self, dossier: &Dossier) -> Result<Artifact, AssemblerError>;

    fn assemble_document(&self, document: &Document) -> Result<Artifact, AssemblerError> {

        let mut result = String::new();

        for paragraph in document.preamble() {

            if let Some(r) = paragraph.compilation_result().as_ref() {

                result.push_str(&r.content());

            } else {

                return Err(AssemblerError::CompiledContentNotFound)
            }
        }

        for chapter in document.chapters() {

            if let Some(r) = chapter.heading().compilation_result().as_ref() {

                result.push_str(&r.content());

            } else {

                return Err(AssemblerError::CompiledContentNotFound)
            }

            for paragraph in chapter.paragraphs() {
                if let Some(r) = paragraph.compilation_result().as_ref() {

                    result.push_str(&r.content());
    
                } else {

                    return Err(AssemblerError::CompiledContentNotFound)
                }
            }
        }

        Ok(Artifact::new(result))

    }

    fn assemble_document_standalone(&self, page_title: &String, styles_references: Option<&Vec<String>>, toc: Option<&TableOfContents>, bibliography: Option<&Bibliography>, document: &Document) -> Result<Artifact, AssemblerError> {
        self.assemble_document(document)
    }
}

pub fn from(format: OutputFormat, configuration: AssemblerConfiguration) -> Box<dyn Assembler> {
    match format {
        OutputFormat::Html => Box::new(HtmlAssembler::new(configuration))  
    }
}