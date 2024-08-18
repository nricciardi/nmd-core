use std::path::PathBuf;

use getset::{Getters, Setters};


#[derive(Debug, Getters, Setters, Default)]
pub struct LoaderConfiguration {
    
    #[getset(get = "pub", set = "pub")]
    input_location: PathBuf,
}


#[derive(Debug, Getters, Setters, Default)]
pub struct LoaderConfigurationOverLay {

    #[getset(get = "pub", set = "pub")]
    dossier_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    document_name: Option<String>,
}