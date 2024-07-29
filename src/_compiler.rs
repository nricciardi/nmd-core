pub mod compilation_configuration;
pub mod dossier;
pub mod output_format;
mod assembler;
pub mod dumpable;
pub mod artifact;
pub mod theme;
pub mod parser;
pub mod loader;
pub mod codex;
pub mod compilable;
pub mod parsing;
pub mod table_of_contents;
pub mod bibliography;
pub mod preview;

use std::collections::HashSet;
use std::path::{self, PathBuf};
use std::sync::RwLock;
use std::{sync::Arc, time::Instant};

use dossier::{dossier_configuration::DossierConfiguration, Document, Dossier};
use dumpable::DumpConfiguration;
use parsing::compilation_configuration::parsing_configuration_overlay::ParsingConfigurationOverLay;
use preview::html_preview::PREVIEW_URL;
use preview::{html_preview::HtmlPreview, Preview, PreviewError};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use theme::Theme;
use thiserror::Error;
use tokio::join;
use tokio::task::{JoinError, JoinSet};
use tokio::sync::RwLock as TokioRwLock;
use crate::{_compiler::{dumpable::{DumpError, Dumpable}, loader::Loader, compilable::Compilable}, constants::{DOSSIER_CONFIGURATION_JSON_FILE_NAME, DOSSIER_CONFIGURATION_YAML_FILE_NAME}, utility::file_utility};
use self::{assembler::{assembler_configuration::AssemblerConfiguration, AssemblerError}, compilation_configuration::CompilationConfiguration, loader::LoadError, parsing::compilation_error::ParsingError};


#[derive(Error, Debug)]
pub enum CompilationError {

    #[error("unknown error")]
    Unknown(String),

    #[error(transparent)]
    LoadError(#[from] LoadError),

    #[error(transparent)]
    ParsingError(#[from] ParsingError),

    #[error(transparent)]
    AssemblerError(#[from] AssemblerError),

    #[error(transparent)]
    DumpError(#[from] DumpError),

    #[error(transparent)]
    PreviewError(#[from] PreviewError),

    #[error(transparent)]
    JoinError(#[from] JoinError),
}

pub struct Compiler {
}

impl Compiler {

    /// Standard dossier compilation based on CompilationConfiguration.
    /// It loads, parses and dumps dossier
    pub async fn load_and_compile_dossier(compilation_configuration: CompilationConfiguration) -> Result<(), CompilationError> {

        let mut dossier = Self::load_dossier(&compilation_configuration).await?;

        Self::compile_dossier(compilation_configuration, &mut dossier).await
    }

    async fn load_dossier(compilation_configuration: &CompilationConfiguration) -> Result<Dossier, CompilationError> {
        log::info!("start to load dossier");

        let loading_start = Instant::now();

        log::debug!("compilation configuration (this will override dossier compilation configuration):\n\n{:#?}\n", compilation_configuration);

        let codex = Arc::new(compilation_configuration.codex());

        let dossier: Dossier;

        let loader = Loader::new();

        if let Some(dstc) = compilation_configuration.documents_subset_to_compile() {

            dossier = loader.load_dossier_from_path_buf_only_documents(&codex, compilation_configuration.input_location(), dstc)?;

        } else {

            dossier = loader.load_dossier_from_path_buf(&codex, compilation_configuration.input_location())?;
        }

        log::info!("dossier loaded in {} ms", loading_start.elapsed().as_millis());

        Ok(dossier)
    }

    async fn compile_dossier(compilation_configuration: CompilationConfiguration, dossier: &mut Dossier) -> Result<(), CompilationError> {
        Self::compile_dossier_parsing_subset(compilation_configuration, dossier, None).await
    }

    async fn compile_dossier_parsing_subset(mut compilation_configuration: CompilationConfiguration, dossier: &mut Dossier, subset_documents_to_parse: Option<HashSet<String>>) -> Result<(), CompilationError> {
        
        log::info!("start to compile dossier");

        let compilation_start = Instant::now();

        let codex = Arc::new(compilation_configuration.codex());
        
        let dossier_configuration = dossier.configuration();

        compilation_configuration.merge_dossier_configuration(dossier_configuration);

        let mut parsing_configuration = compilation_configuration.parsing_configuration();
        parsing_configuration.set_list_bullets_configuration(dossier_configuration.style().list_bullets_configuration().clone());
        parsing_configuration.set_strict_list_check(dossier_configuration.compilation().strict_list_check());

        if parsing_configuration.compress_embed_image() || parsing_configuration.embed_local_image() || parsing_configuration.embed_remote_image() {

            log::warn!("embedding or compressing images is a time consuming task! Consider not using this feature unless strictly necessary");
        }

        log::info!("will use dossier configuration: {:?}", compilation_configuration.input_location());
        log::debug!("will use dossier configuration:\n\n{:#?}\n", dossier_configuration);

        let mut assembler_configuration = AssemblerConfiguration::from(dossier_configuration.clone());

        let dossier_theme = dossier_configuration.style().theme().clone();
        
        log::info!("parsing using theme: {}", parsing_configuration.theme());
        log::debug!("parsing configuration:\n{:#?}\n", parsing_configuration);
        
        if parsing_configuration.fast_draft() {
            log::info!("fast draft mode on!")
        }

        let parsing_configuration_overlay: Option<ParsingConfigurationOverLay>;

        if let Some(subset) = subset_documents_to_parse {

            let mut pco = ParsingConfigurationOverLay::default();

            pco.set_parse_only_documents(Some(subset));

            parsing_configuration_overlay = Some(pco);

        } else {

            parsing_configuration_overlay = None;
        }

        dossier.compile(compilation_configuration.format(), Arc::clone(&codex), Arc::new(RwLock::new(parsing_configuration)), Arc::new(parsing_configuration_overlay))?;

        log::info!("dossier parsed in {} ms", compilation_start.elapsed().as_millis());

        assembler_configuration.set_theme(compilation_configuration.theme().as_ref().unwrap_or(&dossier_theme).clone());
        assembler_configuration.set_preview(compilation_configuration.preview());
        assembler_configuration.set_watching(compilation_configuration.watching());

        log::info!("assembling...");

        let assembly_time = Instant::now();

        let assembler = assembler::from(compilation_configuration.format().clone(), assembler_configuration);

        let mut artifact = assembler.assemble_dossier(&dossier)?;

        log::info!("end to assembly (assembly time {} ms)", assembly_time.elapsed().as_millis());

        let mut output_location = compilation_configuration.output_location().clone();

        if output_location.is_dir() {
            output_location = output_location.join(file_utility::build_output_file_name(
                &dossier.name(),
            Some(&compilation_configuration.format().get_extension())
            ));
        }

        let dump_configuration = DumpConfiguration::new(output_location, compilation_configuration.force_output());

        artifact.dump(&dump_configuration)?;

        log::info!("end to compile dossier (compile time: {} ms)", compilation_start.elapsed().as_millis());

        Ok(())
    }

    /// Standard file compilation based on CompilationConfiguration.
    /// It loads, parses and dumps dossier
    pub async fn load_and_compile_file(mut compilation_configuration: CompilationConfiguration) -> Result<(), CompilationError> {

        log::info!("start to compile dossier");

        let compilation_start = Instant::now();

        log::debug!("compilation configuration (this will override dossier compilation configuration):\n\n{:#?}\n", compilation_configuration);

        let codex = compilation_configuration.codex();

        let loader = Loader::new();

        let mut document: Document = loader.load_document_from_path(&codex, compilation_configuration.input_location())?;

        log::info!("document loaded in {} ms", compilation_start.elapsed().as_millis());

        compilation_configuration.fill_with_default();

        let parsing_configuration = compilation_configuration.parsing_configuration();

        if parsing_configuration.compress_embed_image() || parsing_configuration.embed_local_image() || parsing_configuration.embed_remote_image() {

            log::warn!("embedding or compressing images is a time consuming task! Consider not using this feature unless strictly necessary");
        }

        log::info!("will use dossier configuration: {:?}", compilation_configuration.input_location());

        let mut assembler_configuration = AssemblerConfiguration::default();
        
        log::info!("parsing using theme: {}", parsing_configuration.theme());
        log::debug!("parsing configuration:\n{:#?}\n", parsing_configuration);
        
        if parsing_configuration.fast_draft() {
            log::info!("fast draft mode on!")
        }

        let codex = Arc::new(codex);

        document.compile(compilation_configuration.format(), Arc::clone(&codex), Arc::new(RwLock::new(parsing_configuration)), Arc::new(None))?;

        log::info!("document parsed in {} ms", compilation_start.elapsed().as_millis());

        assembler_configuration.set_theme(compilation_configuration.theme().clone().unwrap_or(Theme::default()));
        assembler_configuration.set_preview(compilation_configuration.preview());
        assembler_configuration.set_watching(compilation_configuration.watching());

        log::info!("assembling...");

        let output_location = compilation_configuration.output_location().clone();

        let assembly_time = Instant::now();

        let assembler = assembler::from(compilation_configuration.format().clone(), assembler_configuration);

        let mut artifact = assembler.assemble_document_standalone(&output_location.file_name().unwrap().to_string_lossy().to_string(), Some(compilation_configuration.styles_raw_path()), None, None, &document)?;

        log::info!("end to assembly (assembly time {} ms)", assembly_time.elapsed().as_millis());

        let dump_configuration = DumpConfiguration::new(output_location, compilation_configuration.force_output());

        artifact.dump(&dump_configuration)?;

        log::info!("end to compile document (compile time: {} ms)", compilation_start.elapsed().as_millis());

        Ok(())
    }

    pub async fn watch_compile_file(compilation_configuration: CompilationConfiguration, min_elapsed_time_between_events_in_secs: u64) -> Result<(), CompilationError> {
        unimplemented!("watch compile file will be added in a next version")
    }

}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use super::*;

    #[tokio::test]
    async fn compile_dossier() {

        let project_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dossier_dir = "nmd-test-dossier-1";
        let nmd_dossier_path = project_directory.join("test-resources").join(dossier_dir);

        assert!(nmd_dossier_path.is_dir());

        let compilation_configuration = CompilationConfiguration::new(nmd_dossier_path.clone(), nmd_dossier_path.clone());

        Compiler::load_and_compile_dossier(compilation_configuration).await.unwrap();
    }
}