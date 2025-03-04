use getset::{Getters, Setters};

use crate::base_parameter::BaseConfiguration;



#[derive(Debug, Default, Getters, Setters)]
pub struct TransformationConfiguration {

    #[getset(get="pub", set="pub")]
    base_params: BaseConfiguration
}