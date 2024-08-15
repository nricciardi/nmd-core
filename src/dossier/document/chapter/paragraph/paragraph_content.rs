

use std::fmt::Debug;
use crate::compiler::{compilation_result_accessor::CompilationResultAccessor, self_compile::SelfCompile};

pub trait ParagraphContent: Debug + SelfCompile + CompilationResultAccessor + Send + Sync {
    
}