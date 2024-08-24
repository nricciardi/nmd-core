use getset::{Getters, MutGetters, Setters};
use serde::Serialize;

use crate::codex::modifier::ModifiersBucket;


#[derive(Debug, Clone, Serialize)]
pub enum CompilationResultPartType {
    Fixed,
    Compilable{ incompatible_modifiers: ModifiersBucket },
}

#[derive(Debug, Getters, Setters, Clone, Serialize)]
pub struct CompilationResultPart {

    #[getset(get_mut = "pub", get = "pub", set = "pub")]
    content: String,

    #[getset(get = "pub", set = "pub")]
    part_type: CompilationResultPartType,
}

impl CompilationResultPart {
    
    pub fn new(content: String, part_type: CompilationResultPartType) -> Self {
        Self {
            content,
            part_type,
        }
    }
}


pub type CompilationResultParts = Vec<CompilationResultPart>;


#[derive(Debug, Clone, Getters, MutGetters, Setters, Serialize)]
pub struct CompilationResult {

    #[getset(get_mut = "pub", get = "pub", set = "pub")]
    parts: CompilationResultParts
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
        Self::new(vec![
            CompilationResultPart::new(
                content,
                CompilationResultPartType::Fixed
            )
        ])
    }

    pub fn new_compilable(content: String, incompatible_modifiers: ModifiersBucket) -> Self {
        Self::new(vec![
            CompilationResultPart::new(
                content,
                CompilationResultPartType::Compilable { incompatible_modifiers }
            )
        ])
    }

    pub fn content(&self) -> String {
        let mut c = String::new();

        &self.parts.iter().for_each(|part| c.push_str(part.content()));

        c
    }

    pub fn add_fixed_part(&mut self, content: String) {
        self.append_compilation_result(&mut Self::new_fixed(content))
    }

    pub fn add_compilable_part(&mut self, content: String, incompatible_modifiers: ModifiersBucket) {
        self.append_compilation_result(&mut Self::new_compilable(content, incompatible_modifiers))
    }

    pub fn apply_compile_function<F, E>(&mut self, f: F) -> Result<(), E>
        where F: Fn(&CompilationResultPart) -> Result<CompilationResult, E> {

        todo!();        // replaced from Compiler::compile_parts

        // let mut new_parts: CompilationResultParts = Vec::new();
        // for part in &self.parts {
        //     match part {
        //         CompilationResultPart::Fixed { content: _ } => new_parts.push(part.clone()),
        //         CompilationResultPart::Compilable { content: _ } => {
        //             let outcome = f(part)?;

        //             Into::<CompilationResultParts>::into(outcome).into_iter().for_each(|p| new_parts.push(p))
        //         },
        //     }
        // }

        // self.parts = new_parts;

        // Ok(())
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

impl Into<CompilationResultParts> for CompilationResult {
    fn into(self) -> CompilationResultParts {
        self.parts
    }
}