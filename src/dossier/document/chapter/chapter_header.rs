use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_heading_modifier::StandardHeading, Modifier}, Codex}, load::{LoadConfiguration, LoadError}, load_block::{LoadBlock, LoadBlockContent}};
use super::{chapter_tag::ChapterTag, heading::{Heading, HeadingLevel}};



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

    /// Load headings and chapter tags from `&str`
    pub fn load(content: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Vec<LoadBlock>, LoadError> {

        let mut headers: Vec<LoadBlock> = Vec::new();

        for heading in StandardHeading::ordered() {     // TODO: include `StandardHeading::ordered()` in `Codex`

            let heading_modifier = Into::<BaseModifier>::into(heading);

            for m in heading_modifier.modifier_pattern_regex().find_iter(content) {

                let matched_str = m.as_str().to_string();

                let m_start = m.start();
                let m_end = m.end();

                log::debug!("header found (between {} and {}): {:?}", m_start, m_end, &matched_str);

                if let Some((heading, tags)) = Self::parse_chapter_heading_and_tags_from_str(&matched_str, codex, configuration)? {

                    headers.push(LoadBlock::new(
                        m_start,
                        m_end,
                        LoadBlockContent::ChapterHeader(ChapterHeader::new(heading, tags))
                    ));
                }

            };
        }

        log::debug!("found headers:\n{:#?}", headers);

        Ok(headers)
    }

    /// Load the chapter heading and metadata from `&str`. This method returns a tuple with optional heading and a chapter tags vector.
    fn parse_chapter_heading_and_tags_from_str(content: &str, _codex: &Codex, _configuration: &LoadConfiguration) -> Result<Option<(Heading, Vec<ChapterTag>)>, LoadError> {

        log::debug!("parse headings and chapter tags from:\n{}", content);

        for heading in StandardHeading::ordered() {         // TODO: insert in codex

            let heading_modifier = Into::<BaseModifier>::into(heading.clone());

            if !heading_modifier.modifier_pattern_regex().is_match(content) {
                continue
            }

            if let Some(capture) = heading_modifier.modifier_pattern_regex().captures(content) {

                match heading {

                    StandardHeading::MinorHeading => {
    
                        let level: HeadingLevel;
    
                        if *last_heading_level < 1 {
                            log::warn!("{} found, but last heading has level {}, so it is set as 1", StandardHeading::MinorHeading.identifier(), last_heading_level);
                            level = 1;
    
                        } else {
    
                            level = *last_heading_level - 1;
                        }
    
                        let title = capture.get(1).unwrap();    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::MajorHeading => {
    
                        let mut level: HeadingLevel = *last_heading_level + 1;
    
                        if level < 1 {
                            log::warn!("level {} < 0, so it is set as 1", level);
                            level = 1;
                        }
    
                        let title = capture.get(1).unwrap();
       
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::SameHeading => {
    
                        let level: HeadingLevel;
                        if *last_heading_level < 1 {
                            log::warn!("{} found, but last heading has level {}, so it is set as 1", StandardHeading::MinorHeading.identifier(), last_heading_level);
                            level = 1;
    
                        } else {
    
                            level = *last_heading_level;
                        }
                        
                        let title = capture.get(1).unwrap();
    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::HeadingGeneralExtendedVersion(_) => {
                        let level: u32 = content.chars().take_while(|&c| c == '#').count() as u32;
    
                        let title = capture.get(1).unwrap();    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::HeadingGeneralCompactVersion(_) => {
                        let matched = heading_modifier.modifier_pattern_regex().captures(content).unwrap();
    
                        let level: HeadingLevel = matched.get(1).unwrap().as_str().parse().unwrap();
                        let title = capture.get(2).unwrap();    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
                }   
            }

        }
        
        Ok(None)
    }

}