use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompilationResultPart {
    Fixed{ content: String },
    Compilable{ content: String },
}

impl CompilationResultPart {
    pub fn content(&self) -> &String {
        match self {
            CompilationResultPart::Fixed { content } => content,
            CompilationResultPart::Compilable { content } => content,
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Getters, MutGetters, Setters)]
pub struct CompilationResult {

    #[getset(get_mut = "pub", get = "pub", get_copy = "pub", set = "pub")]
    parts: Vec<CompilationResultPart>
}

impl CompilationResult {
    pub fn new(parts: Vec<CompilationResultPart>) -> Self {
        Self {
            parts
        }
    }

    pub fn new_empty() -> Self {
        Self {
            parts: Vec::new(),
        }
    }

    pub fn new_fixed(content: String) -> Self {
        Self::new(vec![CompilationResultPart::Fixed { content }])
    }

    pub fn new_compilable(content: String) -> Self {
        Self::new(vec![CompilationResultPart::Compilable { content }])
    }

    pub fn content(&self) -> String {
        let mut c = String::new();

        for part in &self.parts {
            match part {
                CompilationResultPart::Fixed { content } => c.push_str(content),
                CompilationResultPart::Compilable { content } => c.push_str(content),
            }
        }

        c
    }

    pub fn add_fixed_part(&mut self, content: String) {
        self.parts.push(CompilationResultPart::Fixed{ content });
    }

    pub fn add_compilable_part(&mut self, content: String) {
        self.parts.push(CompilationResultPart::Compilable{ content });
    }

    pub fn apply_compile_function<F, E>(&mut self, f: F) -> Result<(), E>
        where F: Fn(&CompilationResultPart) -> Result<CompilationResult, E> {

        let mut new_parts: Vec<CompilationResultPart> = Vec::new();
        for part in &self.parts {
            match part {
                CompilationResultPart::Fixed { content: _ } => new_parts.push(part.clone()),
                CompilationResultPart::Compilable { content: _ } => {
                    let outcome = f(part)?;

                    Into::<Vec<CompilationResultPart>>::into(outcome).into_iter().for_each(|p| new_parts.push(p))
                },
            }
        }

        self.parts = new_parts;

        Ok(())
    }

    pub fn append_compilation_result(&mut self, ext_res: &mut Self) {
        self.parts.append(ext_res.parts_mut());
    }
}

impl Into<String> for CompilationResult {
    fn into(self) -> String {
        self.content()
    }
}

impl Into<Vec<CompilationResultPart>> for CompilationResult {
    fn into(self) -> Vec<CompilationResultPart> {
        self.parts
    }
}