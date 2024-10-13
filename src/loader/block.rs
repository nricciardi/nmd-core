use getset::{CopyGetters, Getters, Setters};
use crate::dossier::document::chapter::{chapter_tag::ChapterTag, heading::Heading, paragraph::Paragraph};




#[derive(Debug, Getters, CopyGetters, Setters)]
pub struct Block {

    #[getset(get_copy = "pub", set = "pub")]
    start: usize,

    #[getset(get_copy = "pub", set = "pub")]
    end: usize,

    #[getset(get = "pub", set = "pub")]
    content: BlockContent
}

impl Block {
    pub fn new(start: usize, end: usize, content: BlockContent) -> Self {
        Self {
            start,
            end,
            content,
        }
    }
}


#[derive(Debug)]
pub enum BlockContent {
    Paragraph(Box<dyn Paragraph>),
    Heading(Heading),
    ChapterTag(ChapterTag)
}