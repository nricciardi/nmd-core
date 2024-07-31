use crate::compiler::compilation_result::CompilationResult;



pub trait CompilationResultAccessor {
    fn compilation_result(&self) -> &Option<CompilationResult>;
}