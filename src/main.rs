// use std::{path::PathBuf, sync::{Arc, RwLock}};

// use nmd_core::{assembler::{html_assembler::{html_assembler_configuration::HtmlAssemblerConfiguration, HtmlAssembler}, Assembler}, codex::{codex_configuration::CodexConfiguration, Codex}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, Compiler}, dossier::Dossier, dumpable::{DumpConfiguration, Dumpable}, loader::{loader_configuration::LoaderConfiguration, Loader}, output_format::OutputFormat};

// fn load_dossier(dossier_path: &PathBuf) -> Dossier {

//     let codex = Codex::of_html(CodexConfiguration::default());

//     let loader_configuration = LoaderConfiguration::default();

//     let dossier = Loader::load_dossier_from_path_buf(dossier_path, &codex, &loader_configuration).unwrap();

//     dossier
// }

// fn build_dossier(dossier: &mut Dossier) {
//     Compiler::compile_dossier(
//         dossier,
//         &OutputFormat::Html,
//         &Codex::of_html(CodexConfiguration::default()),
//         &CompilationConfiguration::default(),
//         Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))
//     ).unwrap();

//     let artifact = HtmlAssembler::assemble_dossier(&dossier, &HtmlAssemblerConfiguration::default()).unwrap();
// }

fn main() {
    
    // let dossier_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("nmd-test-dossier-from-md");

    // let dossier = load_dossier(&dossier_path);


}