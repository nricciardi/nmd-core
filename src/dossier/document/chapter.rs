pub mod paragraph;
pub mod heading;
pub mod chapter_tag;


use chapter_tag::ChapterTag;
use getset::{Getters, MutGetters, Setters};
use paragraph::Paragraph;
use serde::Serialize;
use self::heading::Heading;


#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct Chapter {

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    heading: Heading,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    tags: Vec<ChapterTag>,
    
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    #[serde(skip)]      // TODO
    paragraphs: Vec<Box<dyn Paragraph>>,
}


impl Chapter {

    pub fn new(heading: Heading, tags: Vec<ChapterTag>, paragraphs: Vec<Box<dyn Paragraph>>) -> Self {
        Self {
            heading,
            tags,
            paragraphs
        }
    }
}