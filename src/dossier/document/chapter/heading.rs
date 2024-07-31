use getset::{CopyGetters, Getters, Setters};
use crate::{compiler::{compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor}, resource::resource_reference::ResourceReference};


pub type HeadingLevel = u32;


#[derive(Debug, Getters, CopyGetters, Setters, Clone)]
pub struct Heading {

    #[getset(get_copy = "pub", set = "pub")]
    level: HeadingLevel,

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(set = "pub")]
    compilation_result: Option<CompilationResult>,

    #[getset(get = "pub", set = "pub")]
    resource_reference: Option<ResourceReference>,
}

impl Heading {
    pub fn new(level: HeadingLevel, title: String) -> Self {

        Self {
            level,
            title,
            compilation_result: None,
            resource_reference: None
        }
    }
}

impl CompilationResultAccessor for Heading {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compilation_result
    }
}
