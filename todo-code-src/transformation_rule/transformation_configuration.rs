use std::any::Any;

use getset::{Getters, Setters};
use delegate::delegate;
use crate::{base_parameter::{effort::Effort, output_format::OutputFormat, parallelization::Parallelization, theme::Theme, BaseConfiguration, BaseConfigurationParameters, CustomableConfiguration}, text_compiler::text_compilation_configuration::{TextCompilationConfiguration, TextCompilationConfigurationParameters}, mmo::HashMap};


#[derive(Debug, Default, Getters, Setters)]
pub struct TextTransformationConfigurationParameters {

    #[getset(get="pub", set="pub")]
    base_params: BaseConfigurationParameters,

    #[getset(get="pub", get_mut="pub", set="pub")]
    others: HashMap<String, Box<dyn Any>>
}


pub trait TextTransformationConfiguration: BaseConfiguration + CustomableConfiguration {
}

impl CustomableConfiguration for TextTransformationConfigurationParameters {
    delegate! {
        to self.others {

            fn others(&self) -> &HashMap<String, Box<dyn Any>>;
            fn others_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>>;
        }
    }
}


impl BaseConfiguration for TextTransformationConfigurationParameters {
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

impl TextTransformationConfiguration for TextTransformationConfigurationParameters {
}


impl From<TextCompilationConfigurationParameters> for TextTransformationConfigurationParameters {
    fn from(tccp: TextCompilationConfigurationParameters) -> Self {
        Self {
            base_params: tccp.base_params(),
            others: tccp.others()
        }
    }
}