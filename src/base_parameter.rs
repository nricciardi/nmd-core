pub mod output_format;
pub mod parallelization;
pub mod effort;
pub mod theme;


use core::fmt;

use delegate::delegate;
use effort::Effort;
use getset::{Getters, Setters};
use output_format::OutputFormat;
use parallelization::Parallelization;
use theme::Theme;

pub trait HasBaseConfiguration: fmt::Debug + Default {

    fn output_format(&self) -> &OutputFormat;

    fn set_output_format(&mut self, value: OutputFormat);

    fn parallelization(&self) -> &Parallelization;

    fn set_parallelization(&mut self, value: Parallelization);

    fn effort(&self) -> &Effort;

    fn set_effort(&mut self, value: Effort);

    fn theme(&self) -> &Theme;

    fn set_theme(&mut self, value: Theme);
}


#[derive(Debug, Default, Getters, Setters)]
pub struct BaseConfiguration {

    #[getset(get="pub", set="pub")]
    output_format: OutputFormat,

    #[getset(get="pub", set="pub")]
    parallelization: Parallelization,

    #[getset(get="pub", set="pub")]
    effort: Effort,

    #[getset(get = "pub", set = "pub")]
    theme: Theme,
}


impl HasBaseConfiguration for BaseConfiguration {
    
    delegate! {
        to self.output_format {
            fn output_format(&self) -> &OutputFormat;
            fn set_output_format(&mut self, value: OutputFormat);
        }

        to self.output_format {
            fn parallelization(&self) -> &Parallelization;
            fn set_parallelization(&mut self, value: Parallelization);
        }

        to self.effort {
            fn effort(&self) -> &Effort;
            fn set_effort(&mut self, value: Effort);
        }

        to self.theme {
            fn theme(&self) -> &Theme;
            fn set_theme(&mut self, value: Theme);
        }
    }
}

