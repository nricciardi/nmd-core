use std::collections::HashSet;

use getset::{Getters, Setters};



#[derive(Debug, Getters, Setters, Default)]
pub struct CompilationConfigurationOverLay {

    #[getset(get = "pub", set = "pub")]
    compile_only_documents: Option<HashSet<String>>,
    
    #[getset(get = "pub", set = "pub")]
    additional_style: Option<String>,

    #[getset(get = "pub", set = "pub")]
    dossier_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    document_name: Option<String>,
}