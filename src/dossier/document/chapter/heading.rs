use getset::{CopyGetters, Getters, Setters};
use serde::Serialize;
use crate::{codex::modifier::ModifiersBucket, compiler::{compilable::Compilable, compilation_result::{CompilationResult, CompilationResultPart, CompilationResultPartType, CompilationResultParts}, compilation_result_accessor::CompilationResultAccessor}, resource::resource_reference::ResourceReference, utility::nmd_unique_identifier::NmdUniqueIdentifier};


pub type HeadingLevel = u32;


#[derive(Debug, Getters, CopyGetters, Setters, Clone, Serialize)]
pub struct Heading {

    #[getset(get_copy = "pub", set = "pub")]
    level: HeadingLevel,

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(set = "pub")]
    compilation_result: Option<CompilationResult>,

    #[getset(get = "pub", set = "pub")]
    resource_reference: Option<ResourceReference>,

    #[getset(get = "pub", set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    // compilable_content: CompilableContent,
}

impl Heading {
    pub fn new(level: HeadingLevel, title: String) -> Self {

        Self {
            level,
            title: title.clone(),
            compilation_result: None,
            resource_reference: None,
            nuid: None,
            // compilable_content: CompilationResultParts::from([
            //     CompilationResultPart::new(
            //         title,
            //         CompilationResultPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
            //     )
            // ])
        }
    }
}

impl CompilationResultAccessor for Heading {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compilation_result
    }
}

// impl Compilable for Heading {
//     fn compilable_content(&self) -> &CompilableContent {
//         &self.compilable_content
//     }

//     fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
//         self.nuid.as_ref()
//     }
// }