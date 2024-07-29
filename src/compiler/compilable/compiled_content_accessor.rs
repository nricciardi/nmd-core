use crate::compiler::compilation_result::CompilationResult;



pub trait CompiledContentAccessor {
    fn parsed_content(&self) -> &Option<CompilationResult>;
}