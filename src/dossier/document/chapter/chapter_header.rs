pub mod chapter_header_loading_rule;

use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use super::{chapter_tag::ChapterTag, heading::Heading};


// TODO: rename in Header

/// `ChapterHeading` represents heading of `Chapter`
/// 
/// It contains both `Heading` (title) and `Vec<ChapterTag>` (metadata)
#[derive(Debug, Clone, Getters, MutGetters, Setters, Serialize)]
pub struct ChapterHeader {

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    heading: Heading,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    tags: Vec<ChapterTag>,
}

impl ChapterHeader {

    pub fn new(heading: Heading, tags: Vec<ChapterTag>,) -> Self {
        Self {
            heading,
            tags
        }
    }

}