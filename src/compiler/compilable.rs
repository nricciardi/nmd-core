use std::{convert::Infallible, fmt::Debug, str::FromStr};
use getset::{Getters, Setters};
use crate::{codex::modifier::ModifiersBucket, utility::nmd_unique_identifier::NmdUniqueIdentifier};
use super::compilation_result::{CompilationResultPart, CompilationResultParts};


pub type CompilableContent = CompilationResultParts;


pub trait Compilable: Debug {
    fn compilable_content(&self) -> &CompilableContent;

    fn nuid(&self) -> Option<&NmdUniqueIdentifier>;
}


impl Compilable for CompilationResultParts {
    fn compilable_content(&self) -> &CompilableContent {
        &self
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        None
    }
}


#[derive(Debug, Getters, Setters)]
pub struct GenericCompilable {
    content: CompilableContent,

    nuid: Option<NmdUniqueIdentifier>
}

impl GenericCompilable {
    pub fn new(content: CompilableContent, nuid: Option<NmdUniqueIdentifier>) -> Self {
        Self {
            content,
            nuid
        }
    }
}

impl Compilable for GenericCompilable {
    fn compilable_content(&self) -> &CompilableContent {
        &self.content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
    }
}

// impl FromStr for GenericCompilable {
//     type Err = Infallible;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Ok(Self {
//             content: s.to_string(),
//             nuid: None
//         })
//     }
// }

// impl From<String> for GenericCompilable {
//     fn from(value: String) -> Self {
//         Self {
//             content: value,
//             nuid: None
//         }
//     }
// }