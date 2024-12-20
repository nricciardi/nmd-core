use getset::{CopyGetters, Setters};
use serde::{Deserialize, Serialize};


fn yes() -> bool {
    true
}

fn no() -> bool {
    false
}


#[derive(Debug, Clone, Deserialize, Serialize, CopyGetters, Setters)]
pub struct DossierConfigurationCompilation {

    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    embed_local_image: bool,

    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    embed_remote_image: bool,
    
    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    compress_embed_image: bool,
    
    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    strict_image_src_check: bool,
    
    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    parallelization: bool,
    
    #[serde(default = "no")]
    #[getset(get_copy = "pub", set = "pub")]
    use_remote_addons: bool,

    #[serde(default = "no")]
    #[getset(get_copy = "pub", set = "pub")]
    strict_list_check: bool,

    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    strict_greek_letters_check: bool,

    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    strict_cite_check: bool,

    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    strict_reference_check: bool,

    #[serde(default = "yes")]
    #[getset(get_copy = "pub", set = "pub")]
    strict_paragraph_loading_rules_check: bool,
}

impl Default for DossierConfigurationCompilation {
    fn default() -> Self {
        Self {
            embed_local_image: false,
            embed_remote_image: false,
            compress_embed_image: false,
            strict_image_src_check: true,
            parallelization: true,
            use_remote_addons: false,
            strict_list_check: false,
            strict_cite_check: true,
            strict_greek_letters_check: true,
            strict_reference_check: true,
            strict_paragraph_loading_rules_check: true,
        }
    }
}