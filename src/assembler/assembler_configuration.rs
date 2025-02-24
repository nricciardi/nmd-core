use std::path::PathBuf;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use crate::{dossier::dossier_configuration::DossierConfiguration, theme::Theme};


#[derive(Debug, Clone, Getters, CopyGetters, MutGetters, Setters)]
pub struct AssemblerConfiguration {

    #[getset(get = "pub", set = "pub")]
    theme: Theme,

    #[getset(get_copy = "pub", set = "pub")]
    use_remote_addons: bool,

    #[getset(get_copy = "pub", set = "pub")]
    parallelization: bool,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    external_styles_paths: Vec<PathBuf>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    external_styles: Vec<String>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    external_scripts_paths: Vec<PathBuf>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    external_scripts: Vec<String>,
}

impl AssemblerConfiguration {
    
    pub fn new(theme: Theme, use_remote_addons: bool, parallelization: bool) -> Self {
        Self {
            theme,
            use_remote_addons,
            parallelization,
            
            ..Default::default()
        }
    }
}

impl Default for AssemblerConfiguration {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            use_remote_addons: false,
            parallelization: false,
            external_styles_paths: Vec::new(),
            external_styles: Vec::new(),
            external_scripts_paths: Vec::new(),
            external_scripts: Vec::new(),
        }
    }
}

impl From<&DossierConfiguration> for AssemblerConfiguration {
    fn from(dossier_configuration: &DossierConfiguration) -> Self {
        Self {
            theme: dossier_configuration.style().theme().clone(),
            use_remote_addons: dossier_configuration.compilation().use_remote_addons(),
            parallelization: dossier_configuration.compilation().parallelization(),

            ..Default::default()
        }
    }
}