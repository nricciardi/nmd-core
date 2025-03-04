pub mod output_format;
pub mod parallelization;
pub mod effort;
pub mod theme;


use effort::Effort;
use getset::{Getters, Setters};
use output_format::OutputFormat;
use parallelization::Parallelization;
use theme::Theme;


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

