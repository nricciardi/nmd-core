//! `Assembler` permits to build final `Artifact` of a compiled dossier or document

pub mod assembler_configuration;
pub mod html_assembler;


use std::fmt::Debug;
use assembler_configuration::AssemblerConfiguration;
use thiserror::Error;
use crate::{compilation::compilation_outcome::CompilationOutcome, dossier::{document::chapter::chapter_tag::ChapterTag, dossier_configuration::DossierConfiguration}, resource::ResourceError};
use super::artifact::ArtifactError;


#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("too few elements to assemble: {0}")]
    TooFewElements(String),

    #[error(transparent)]
    ArtifactError(#[from] ArtifactError),

    #[error("compiled content not found")]
    CompiledContentNotFound,

    #[error(transparent)]
    ResourceError(#[from] ResourceError),
}


pub trait Assembler: Debug + Sync + Send {

    /// Assemble dossier
    fn assemble_dossier(&self, compiled_documents: &Vec<CompilationOutcome>, compiled_toc: Option<&CompilationOutcome>, compiled_bib: Option<&CompilationOutcome>, dossier_configuration: &DossierConfiguration, configuration: &AssemblerConfiguration) -> Result<String, AssemblerError>;

    /// Assemble document
    // fn assemble_document(&self, document: &CompilationOutcome) -> Result<String, AssemblerError>;

    /// Assemble a standalone document, so `page_title`, `styles_references`, `toc` and `bibliography` are needed
    fn assemble_document_standalone(&self, page_title: &str, complied_document: &CompilationOutcome, compiled_toc: Option<&CompilationOutcome>, compiled_bib: Option<&CompilationOutcome>, configuration: &AssemblerConfiguration) -> Result<String, AssemblerError>;

    fn assemble_bundle(&self, compiled_document: &Vec<CompilationOutcome>, compiled_chapters: &Vec<CompilationOutcome>, configuration: &AssemblerConfiguration) -> Result<String, AssemblerError>;

    fn assemble_chapter(&self, chapter_tags: &Vec<ChapterTag>, compiled_heading: &CompilationOutcome, compiled_paragraphs: &Vec<CompilationOutcome>, configuration: &AssemblerConfiguration) -> Result<String, AssemblerError>;
}


#[cfg(test)]
mod test {
}