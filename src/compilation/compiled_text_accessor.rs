use crate::compilable_text::CompilableText;


// TODO: deprecated
pub trait CompiledTextAccessor {
    fn compiled_text(&self) -> Option<&CompilableText>;
}