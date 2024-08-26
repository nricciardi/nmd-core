use crate::compilable_text::CompilableText;



pub trait CompiledTextAccessor {
    fn compiled_text(&self) -> Option<&CompilableText>;
}