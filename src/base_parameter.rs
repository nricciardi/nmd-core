pub mod output_format;
pub mod parallelization;
pub mod effort;
pub mod theme;


use core::fmt;
use std::{any::Any, sync::Arc};

use delegate::delegate;
use effort::Effort;
use getset::{Getters, Setters};
use output_format::OutputFormat;
use parallelization::Parallelization;
use theme::Theme;

use crate::mmo::HashMap;

pub trait BaseConfiguration: fmt::Debug {

    fn output_format(&self) -> &OutputFormat;

    fn set_output_format(&mut self, value: OutputFormat);

    fn parallelization(&self) -> &Parallelization;

    fn set_parallelization(&mut self, value: Parallelization);

    fn effort(&self) -> &Effort;

    fn set_effort(&mut self, value: Effort);

    fn theme(&self) -> &Theme;

    fn set_theme(&mut self, value: Theme);
}


pub trait CustomableConfiguration: fmt::Debug {
    
    fn others(&self) -> &HashMap<String, Arc<dyn Any>>;

    fn others_mut(&mut self) -> &mut HashMap<String, Arc<dyn Any>>;
}


#[derive(Debug, Default, Getters, Setters, Clone)]
pub struct BaseConfigurationParameters {

    #[getset(get="pub", set="pub")]
    output_format: OutputFormat,

    #[getset(get="pub", set="pub")]
    parallelization: Parallelization,

    #[getset(get="pub", set="pub")]
    effort: Effort,

    #[getset(get = "pub", set = "pub")]
    theme: Theme,
}


impl BaseConfiguration for BaseConfigurationParameters {
    
    delegate! {
        to self {
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

