use std::collections::HashMap;

use getset::{CopyGetters, Getters, Setters};

use crate::{base_parameter::BaseConfiguration, utility::datastruct::bucket::Bucket};

use super::transformation_rule::{TextTransformationRule, TextTransformationRuleIdentifier};

#[derive(Debug, Getters, CopyGetters, Setters, Clone)]
pub struct TextCompilationConfiguration  {

    #[getset(get = "pub", set = "pub")]
    dossier_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    document_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    excluded_rules: Bucket<TextTransformationRuleIdentifier>,

    #[getset(get = "pub", set = "pub")]
    transformation_rules: Vec<Box<dyn TextTransformationRule>>,
    
    #[getset(get = "pub", set = "pub")]
    base_params: BaseConfiguration,

    #[getset(get = "pub", set = "pub")]
    external_params: HashMap<String, String>
}

