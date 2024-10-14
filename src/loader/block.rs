use getset::{CopyGetters, Getters, MutGetters, Setters};
use crate::dossier::document::chapter::{chapter_tag::ChapterTag, heading::Heading, paragraph::Paragraph};




#[derive(Debug, Getters, CopyGetters, MutGetters, Setters)]
pub struct Block {

    #[getset(get_copy = "pub", set = "pub")]
    start: usize,

    #[getset(get_copy = "pub", set = "pub")]
    end: usize,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
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

impl Into<BlockContent> for Block {
    fn into(self) -> BlockContent {
        self.content
    }
}

impl TryInto<Box<dyn Paragraph>> for Block {
    type Error = String;

    fn try_into(self) -> Result<Box<dyn Paragraph>, Self::Error> {
        if let BlockContent::Paragraph(p) = self.content {
            return Ok(p)
        }

        Err(String::from("this block doesn't contain a paragraph"))
    }
}

impl TryInto<Heading> for Block {
    type Error = String;

    fn try_into(self) -> Result<Heading, Self::Error> {
        if let BlockContent::Heading(h) = self.content {
            return Ok(h)
        }

        Err(String::from("this block doesn't contain an heading"))
    }
}

impl TryInto<ChapterTag> for Block {
    type Error = String;

    fn try_into(self) -> Result<ChapterTag, Self::Error> {
        if let BlockContent::ChapterTag(t) = self.content {
            return Ok(t)
        }

        Err(String::from("this block doesn't contain a chapter tag"))
    }
}


#[derive(Debug)]
pub enum BlockContent {
    Paragraph(Box<dyn Paragraph>),
    Heading(Heading),
    ChapterTag(ChapterTag)
}