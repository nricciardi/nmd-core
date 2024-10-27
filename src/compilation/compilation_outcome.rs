use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

use crate::compilable_text::CompilableText;



#[derive(Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
pub struct CompilationOutcome {

    #[getset(get="pub", get_mut="pub")]
    content: String
}

impl CompilationOutcome {
    pub fn empty() -> Self {
        Self {
            content: String::new()
        }
    }
}

impl From<String> for CompilationOutcome {
    fn from(content: String) -> Self {
        Self {
            content
        }
    }
}


impl From<&str> for CompilationOutcome {
    fn from(content: &str) -> Self {
        Self::from(content.to_string())
    }
}

impl From<&CompilableText> for CompilationOutcome {
    fn from(value: &CompilableText) -> Self {
        Self::from(value.content())
    }
}
