use std::path::PathBuf;
use getset::{CopyGetters, Getters, Setters};


#[derive(Debug, Getters, CopyGetters, Setters)]
pub struct LoaderConfiguration {
    
    #[getset(get = "pub", set = "pub")]
    input_location: PathBuf,

    #[getset(get_copy = "pub", set = "pub")]
    strict_focus_block_check: bool,
}

impl Default for LoaderConfiguration {
    fn default() -> Self {
        Self {
            input_location: PathBuf::from("."),
            strict_focus_block_check: false
        }
    }
}


#[derive(Debug, Getters, Setters, Default, Clone)]
pub struct LoaderConfigurationOverLay {

    #[getset(get = "pub", set = "pub")]
    dossier_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    document_name: Option<String>,
}