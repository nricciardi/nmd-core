use getset::{CopyGetters, Getters, MutGetters, Setters};
use crate::dossier::document::chapter::{chapter_tag::ChapterTag, heading::Heading, paragraph::Paragraph};




#[derive(Debug, Getters, CopyGetters, MutGetters, Setters)]
pub struct LoadBlock {

    #[getset(get_copy = "pub", set = "pub")]
    start: usize,

    #[getset(get_copy = "pub", set = "pub")]
    end: usize,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    content: LoadBlockContent
}

impl LoadBlock {
    pub fn new(start: usize, end: usize, content: LoadBlockContent) -> Self {
        Self {
            start,
            end,
            content,
        }
    }
}

impl Into<LoadBlockContent> for LoadBlock {
    fn into(self) -> LoadBlockContent {
        self.content
    }
}

impl TryInto<Box<dyn Paragraph>> for LoadBlock {
    type Error = String;

    fn try_into(self) -> Result<Box<dyn Paragraph>, Self::Error> {
        if let LoadBlockContent::Paragraph(p) = self.content {
            return Ok(p)
        }

        Err(String::from("this block doesn't contain a paragraph"))
    }
}

impl TryInto<Heading> for LoadBlock {
    type Error = String;

    fn try_into(self) -> Result<Heading, Self::Error> {
        if let LoadBlockContent::Heading(h) = self.content {
            return Ok(h)
        }

        Err(String::from("this block doesn't contain an heading"))
    }
}

impl TryInto<ChapterTag> for LoadBlock {
    type Error = String;

    fn try_into(self) -> Result<ChapterTag, Self::Error> {
        if let LoadBlockContent::ChapterTag(t) = self.content {
            return Ok(t)
        }

        Err(String::from("this block doesn't contain a chapter tag"))
    }
}


#[derive(Debug)]
pub enum LoadBlockContent {
    Paragraph(Box<dyn Paragraph>),
    Heading(Heading),
    ChapterTag(ChapterTag)
}