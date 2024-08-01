//! `Assembler` permits to build final `Artifact` of a compiled dossier or document

pub mod html_assembler;
pub mod assembler_configuration;


use std::path::PathBuf;

use thiserror::Error;
use crate::{compiler::{compilation_result_accessor::CompilationResultAccessor, compilation_error::CompilationError}, resource::ResourceError};
use self::assembler_configuration::AssemblerConfiguration;
use super::{artifact::{Artifact, ArtifactError}, bibliography::Bibliography, dossier::{Document, Dossier}, table_of_contents::TableOfContents};


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

    /// Assemble dossier
    fn assemble_dossier(dossier: &Dossier, configuration: &AssemblerConfiguration) -> Result<Artifact, AssemblerError> where Self: Sized;

    /// Assemble document
    fn assemble_document(document: &Document, _configuration: &AssemblerConfiguration) -> Result<Artifact, AssemblerError> where Self: Sized {

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

    /// Assemble a standalone document, so `page_title`, `styles_references`, `toc` and `bibliography` are needed
    fn assemble_document_standalone(document: &Document, _page_title: &String, _external_styles_paths: Option<&Vec<PathBuf>>, _external_styles: Option<&Vec<String>>, _toc: Option<&TableOfContents>, _bibliography: Option<&Bibliography>, configuration: &AssemblerConfiguration) -> Result<Artifact, AssemblerError> where Self: Sized{
        Self::assemble_document(document, configuration)
    }
}


#[cfg(test)]
mod test {
}