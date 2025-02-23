use std::path::PathBuf;
use getset::{CopyGetters, Getters, Setters};


#[derive(Debug, Getters, CopyGetters, Setters, Clone)]
pub struct LoadConfiguration {
    
    #[getset(get = "pub", set = "pub")]
    input_location: PathBuf,

    #[getset(get_copy = "pub", set = "pub")]
    strict_dossier_configuration_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_focus_block_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_paragraphs_loading_rules_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    parallelization: bool,

    // #[getset(get = "pub", set = "pub")]
    // dossier_name: Option<String>,       // TODO: remove it, place it as method parameter

    // #[getset(get = "pub", set = "pub")]
    // document_name: Option<String>,       // TODO: remove it, place it as method parameter
}

impl Default for LoadConfiguration {
    fn default() -> Self {
        Self {
            input_location: PathBuf::from("."),
            strict_focus_block_check: false,
            strict_dossier_configuration_check: true,
            strict_paragraphs_loading_rules_check: true,
            parallelization: true,
            // document_name: None,
            // dossier_name: None
        }
    }
}