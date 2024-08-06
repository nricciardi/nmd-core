pub mod content_tree;

use getset::{CopyGetters, Getters, Setters};
use serde::{Deserialize, Serialize};
use crate::compiler::compilation_result::CompilationResult;
use super::dossier::document::chapter::heading::Heading;


pub const TOC_INDENTATION: &str = r#"<span class="toc-item-indentation"></span>"#;



#[derive(Debug, Clone, Getters, CopyGetters, Setters, Serialize, Deserialize)]
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

    #[getset(get = "pub", set = "pub")]
    compilation_result: Option<CompilationResult>,
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