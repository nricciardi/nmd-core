use std::{convert::Infallible, fmt::Debug, str::FromStr};
use getset::{Getters, Setters};
use crate::utility::nmd_unique_identifier::NmdUniqueIdentifier;


pub type CompilableContent = String;


pub trait Compilable: Debug {
    fn compilable_content(&self) -> &CompilableContent;

    fn nuid(&self) -> Option<&NmdUniqueIdentifier>;
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

impl FromStr for GenericCompilable {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            content: s.to_string(),
            nuid: None
        })
    }
}

impl From<String> for GenericCompilable {
    fn from(value: String) -> Self {
        Self {
            content: value,
            nuid: None
        }
    }
}