//! `Assembler` permits to build final `Artifact` of a compiled dossier or document

pub mod html_assembler;


use std::fmt::Debug;

use thiserror::Error;
use crate::{compilation::{compilation_error::CompilationError, compiled_text_accessor::CompiledTextAccessor}, dossier::document::{Chapter, Document}, resource::ResourceError};
use super::{artifact::{Artifact, ArtifactError}, bibliography::Bibliography, dossier::Dossier, table_of_contents::TableOfContents};


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

pub trait ChapterAssembler: Debug {

    fn assemble(&self, chapter: &Chapter) -> Result<Artifact, AssemblerError>;
}


pub trait Assembler {

    type Configuration;

    /// Assemble dossier
    fn assemble_dossier(dossier: &Dossier, configuration: &Self::Configuration) -> Result<Artifact, AssemblerError> where Self: Sized;

    /// Assemble document
    fn assemble_document(document: &Document, _configuration: &Self::Configuration) -> Result<Artifact, AssemblerError> where Self: Sized {

        let mut result = String::new();

        for paragraph in document.content().preamble() {

            if let Some(r) = paragraph.compiled_text().as_ref() {

                result.push_str(&r.content());

            } else {

                return Err(AssemblerError::CompiledContentNotFound)
            }
        }

        for chapter in document.content().chapters() {

            if let Some(r) = chapter.header().heading().compiled_text().as_ref() {

                result.push_str(&r.content());

            } else {

                return Err(AssemblerError::CompiledContentNotFound)
            }

            for paragraph in chapter.paragraphs() {
                if let Some(r) = paragraph.compiled_text().as_ref() {

                    result.push_str(&r.content());
    
                } else {

                    return Err(AssemblerError::CompiledContentNotFound)
                }
            }
        }

        Ok(Artifact::new(result))

    }

    /// Assemble a standalone document, so `page_title`, `styles_references`, `toc` and `bibliography` are needed
    fn assemble_document_standalone(document: &Document, _page_title: &String, _toc: Option<&TableOfContents>, _bibliography: Option<&Bibliography>, configuration: &Self::Configuration) -> Result<Artifact, AssemblerError> where Self: Sized{
        Self::assemble_document(document, configuration)
    }
}


#[cfg(test)]
mod test {
}