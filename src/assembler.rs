//! `Assembler` permits to build final `Artifact` of a compiled dossier or document

pub mod html_assembler;


use std::fmt::Debug;
use thiserror::Error;
use crate::{compilation::compilation_outcome::CompilationOutcome, dossier::document::chapter::chapter_tag::ChapterTag, resource::ResourceError};
use super::{artifact::{Artifact, ArtifactError}, bibliography::Bibliography, dossier::Dossier, table_of_contents::TableOfContents};


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
    fn assemble_dossier(&self, dossier: &Dossier) -> Result<Artifact, AssemblerError>;

    /// Assemble document
    // fn assemble_document(&self, document: &CompilationOutcome) -> Result<String, AssemblerError>;

    /// Assemble a standalone document, so `page_title`, `styles_references`, `toc` and `bibliography` are needed
    fn assemble_document_standalone(&self, document: &CompilationOutcome, _page_title: &String, _toc: Option<&TableOfContents>, _bibliography: Option<&Bibliography>) -> Result<Artifact, AssemblerError>;

    fn assemble_bundle(&self, compiled_preamble: &Vec<CompilationOutcome>, compiled_chapters: &Vec<CompilationOutcome>) -> Result<String, AssemblerError>;

    fn assemble_chapter(&self, chapter_tags: &Vec<ChapterTag>, compiled_heading: &CompilationOutcome, compiled_paragraphs: &Vec<CompilationOutcome>) -> Result<String, AssemblerError>;
}


#[cfg(test)]
mod test {
}