use getset::{CopyGetters, Getters, Setters};
use serde::Serialize;
use crate::{compilable_text::CompilableText, compiler::compiled_text_accessor::CompiledTextAccessor, resource::resource_reference::ResourceReference, utility::nmd_unique_identifier::NmdUniqueIdentifier};


pub type HeadingLevel = u32;


#[derive(Debug, Getters, CopyGetters, Setters, Clone, Serialize)]
pub struct Heading {

    #[getset(get_copy = "pub", set = "pub")]
    level: HeadingLevel,

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(set = "pub")]
    compilation_result: Option<CompilableText>,

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

impl CompiledTextAccessor for Heading {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compilation_result.as_ref()
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