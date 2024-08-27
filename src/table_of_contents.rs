pub mod content_tree;

use getset::{CopyGetters, Getters, Setters};
use serde::Serialize;
use crate::{compilable_text::CompilableText, compiler::compiled_text_accessor::CompiledTextAccessor};
use super::dossier::document::chapter::heading::Heading;


pub const TOC_INDENTATION: &str = r#"<span class="toc-item-indentation"></span>"#;



#[derive(Debug, Clone, Getters, CopyGetters, Setters, Serialize)]
pub struct TableOfContents {

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get_copy = "pub", set = "pub")]
    page_numbers: bool,

    #[getset(get_copy = "pub", set = "pub")]
    plain: bool,

    #[getset(get_copy = "pub", set = "pub")]
    maximum_heading_level: usize,

    #[getset(get = "pub", set = "pub")]
    headings: Vec<Heading>,

    #[getset(set = "pub")]
    compilation_result: Option<CompilableText>,
}

impl TableOfContents {
    pub fn new(title: String, page_numbers: bool, plain: bool, maximum_heading_level: usize, headings: Vec<Heading>) -> Self {
        Self {
            title,
            page_numbers,
            plain,
            maximum_heading_level,
            headings,
            compilation_result: None
        }
    }
}

impl CompiledTextAccessor for TableOfContents {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compilation_result.as_ref()
    }
}