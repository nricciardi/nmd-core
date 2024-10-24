//! `Loader` permits to create `Dossier` or `Document` reading filesystem


pub mod loader_configuration;
pub mod paragraph_loading_rule;
pub mod load_block;


use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use load_block::{LoadBlock, LoadBlockContent};
use getset::{Getters, Setters};
use loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay};
use once_cell::sync::Lazy;
use paragraph_loading_rule::ParagraphLoadingRule;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use thiserror::Error;
use crate::codex::modifier::base_modifier::BaseModifier;
use crate::codex::modifier::standard_heading_modifier::StandardHeading;
use crate::codex::modifier::Modifier;
use crate::content_bundle::ContentBundle;
use crate::dossier::document::Document;
use crate::resource::disk_resource::DiskResource;
use crate::resource::resource_reference::ResourceReferenceError;
use crate::resource::{Resource, ResourceError};
use super::codex::Codex;
use super::dossier::document::chapter::chapter_tag::ChapterTag;
use super::dossier::dossier_configuration::DossierConfiguration;
use super::dossier::Dossier;
use super::dossier::document::chapter::heading::{Heading, HeadingLevel};




#[derive(Error, Debug)]
pub enum LoadError {
    #[error(transparent)]
    ResourceError(#[from] ResourceError),

    #[error(transparent)]
    ResourceReferenceError(#[from] ResourceReferenceError),

    #[error("elaboration error: {0}")]
    ElaborationError(String),
    
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("block error: {0}")]
    BlockError(String)
}

impl Clone for LoadError {
    fn clone(&self) -> Self {
        match self {
            Self::IoError(e) => Self::ElaborationError(e.to_string()),
            other => other.clone()
        }
    }
}


#[derive(Debug, Getters, Setters)]
pub struct Loader {
}

impl Loader {


}


