use std::path::PathBuf;

use nmd_core::{codex::{codex_configuration::CodexConfiguration, Codex}, dossier::Dossier, loader::{loader_configuration::LoaderConfiguration, Loader}};

fn load_dossier(dossier_path: &PathBuf) -> Dossier {

    let codex = Codex::of_html(CodexConfiguration::default());

    let loader_configuration = LoaderConfiguration::default();

    let dossier = Loader::load_dossier_from_path_buf(dossier_path, &codex, &loader_configuration).unwrap();

    dossier
}


fn main() {
    
    let dossier_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("nmd-test-dossier-from-md");

    let dossier = load_dossier(&dossier_path);
}