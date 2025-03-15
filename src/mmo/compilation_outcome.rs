use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

use crate::text_compiler::text::Text;



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

impl From<Text> for CompilationOutcome {
    fn from(text: Text) -> Self {
        Self {
            content: text.content()
        }
    }
}


impl From<&str> for CompilationOutcome {
    fn from(content: &str) -> Self {
        Self::from(content.to_string())
    }
}