
#[derive(Debug, Clone)]
pub enum CompilationResultPart {
    Fixed{ content: String },
    Mutable{ content: String },
}

impl CompilationResultPart {
    pub fn content(&self) -> &String {
        match self {
            CompilationResultPart::Fixed { content } => content,
            CompilationResultPart::Mutable { content } => content,
        }
    }
}



#[derive(Debug, Clone)]
pub struct CompilationResult {
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

    pub fn new_mutable(content: String) -> Self {
        Self::new(vec![CompilationResultPart::Mutable { content }])
    }

    pub fn parsed_content(&self) -> String {
        let mut parsed_content = String::new();

        for part in &self.parts {
            match part {
                CompilationResultPart::Fixed { content } => parsed_content.push_str(content),
                CompilationResultPart::Mutable { content } => parsed_content.push_str(content),
            }
        }

        parsed_content
    }

    pub fn add_fixed_part(&mut self, content: String) {
        self.parts.push(CompilationResultPart::Fixed{ content });
    }

    pub fn add_mutable_part(&mut self, content: String) {
        self.parts.push(CompilationResultPart::Mutable{ content });
    }

    pub fn parts(&self) -> &Vec<CompilationResultPart> {
        &self.parts
    }

    pub fn parts_mut(&mut self) -> &mut Vec<CompilationResultPart> {
        &mut self.parts
    }

    pub fn apply_parsing_function_to_mutable_parts<F, E>(&mut self, f: F) -> Result<(), E>
        where F: Fn(&CompilationResultPart) -> Result<CompilationResult, E> {

        let mut new_parts: Vec<CompilationResultPart> = Vec::new();
        for part in &self.parts {
            match part {
                CompilationResultPart::Fixed { content: _ } => new_parts.push(part.clone()),
                CompilationResultPart::Mutable { content: _ } => {
                    let outcome = f(part)?;

                    Into::<Vec<CompilationResultPart>>::into(outcome).into_iter().for_each(|p| new_parts.push(p))
                },
            }
        }

        self.parts = new_parts;

        Ok(())
    }

    pub fn append_parsing_outcome(&mut self, ext_outcome: &mut Self) {
        self.parts.append(ext_outcome.parts_mut());
    }
}

impl Into<String> for CompilationResult {
    fn into(self) -> String {
        self.parsed_content()
    }
}

impl Into<Vec<CompilationResultPart>> for CompilationResult {
    fn into(self) -> Vec<CompilationResultPart> {
        self.parts
    }
}