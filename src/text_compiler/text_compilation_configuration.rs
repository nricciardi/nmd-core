use std::any::Any;
use delegate::delegate;
use getset::{CopyGetters, Getters, Setters};
use crate::utility::datastruct::HashMap;
use crate::{base_parameter::{effort::Effort, output_format::OutputFormat, parallelization::Parallelization, theme::Theme, BaseConfiguration, BaseConfigurationParameters, CustomableConfiguration}, utility::datastruct::bucket::Bucket};
use super::transformation_rule::{TextTransformationRule, TextTransformationRuleIdentifier};


pub trait TextCompilationConfiguration {
    
    fn dossier_name(&self) -> &Option<String>;
    fn set_dossier_name(&mut self, value: Option<String>);

    fn document_name(&self) -> &Option<String>;
    fn set_document_name(&mut self, value: Option<String>);

    /// Bucket of rules which will not be tested
    fn excluded_rules(&self) -> &Bucket<TextTransformationRuleIdentifier>;
    fn set_excluded_rules(&mut self, value: Bucket<TextTransformationRuleIdentifier>);

    // TODO: move Vec to Iterator
    fn transformation_rules(&self) -> &Vec<Box<dyn TextTransformationRule>>;
    fn set_transformation_rules(&mut self, value: Vec<Box<dyn TextTransformationRule>>);
}

#[derive(Debug, Getters, CopyGetters, Setters, Clone)]
pub struct TextCompilationConfigurationParameters  {

    #[getset(get = "pub", set = "pub")]
    dossier_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    document_name: Option<String>,

    #[getset(get = "pub", set = "pub")]
    excluded_rules: Bucket<TextTransformationRuleIdentifier>,

    #[getset(get = "pub", set = "pub")]
    transformation_rules: Vec<Box<dyn TextTransformationRule>>,
    
    #[getset(get="pub", set="pub")]
    base_params: BaseConfigurationParameters,

    #[getset(get="pub", get_mut="pub", set="pub")]
    others: HashMap<String, Box<dyn Any>>
}

impl Default for TextCompilationConfigurationParameters {
    fn default() -> Self {
        Self {
            dossier_name: None,
            document_name: None,
            excluded_rules: Bucket::None,
            transformation_rules: Default::default(),
            base_params: Default::default(),
            others: Default::default()
        }
    }
}

impl BaseConfiguration for TextCompilationConfigurationParameters {
    delegate! {
        to self.base_params {

            fn output_format(&self) -> &OutputFormat;
            fn set_output_format(&mut self, value: OutputFormat);

            fn parallelization(&self) -> &Parallelization;
            fn set_parallelization(&mut self, value: Parallelization);

            fn effort(&self) -> &Effort;
            fn set_effort(&mut self, value: Effort);

            fn theme(&self) -> &Theme;
            fn set_theme(&mut self, value: Theme);
        }
    }
}

impl CustomableConfiguration for TextCompilationConfigurationParameters {
    delegate! {
        to self {

            fn others(&self) -> &HashMap<String, Box<dyn Any>>;
            fn others_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>>;
        }
    }
}

impl TextCompilationConfiguration for TextCompilationConfigurationParameters {
    
    delegate! {
        to self {

            fn dossier_name(&self) -> &Option<String>;
            fn set_dossier_name(&mut self, value: Option<String>);
            
            fn document_name(&self) -> &Option<String>;
            fn set_document_name(&mut self, value: Option<String>);

            fn excluded_rules(&self) -> &Bucket<TextTransformationRuleIdentifier>;
            fn set_excluded_rules(&mut self, value: Bucket<TextTransformationRuleIdentifier>);

            fn transformation_rules(&self) -> &Vec<Box<dyn TextTransformationRule>>;
            fn set_transformation_rules(&mut self, value: Vec<Box<dyn TextTransformationRule>>);
        }
    }
    
}

