use std::collections::HashSet;

use crate::{codex::Codex, load::{load_configuration::LoadConfiguration, load_error::LoadError, loading_rule::LoadingRule}, utility::datastruct::span::Span};

use super::ChapterHeader;

pub type ChapterHeaderLoadingRule = dyn LoadingRule<ChapterHeader>;


#[derive(Debug)]
pub struct StandardChapterHeaderLoadingRule {

}

impl ChapterHeader {
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
    
                        let level = HeadingLevel::Minor;

                        let title = capture.get(1).unwrap();    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::MajorHeading => {
                        
                        let level = HeadingLevel::Major;
    
                        let title = capture.get(1).unwrap();
       
    
                        let tags = ChapterTag::load_chapter_tags_from_str(&content[title.end()..])?;
                    
                        return Ok(Some((
                            Heading::new(level, title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::SameHeading => {
    
                        
                        let level = HeadingLevel::Same;
                        
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
                            Heading::new(HeadingLevel::Explicit(level), title.as_str().to_string()),
                            tags
                        )))
                    },
    
                    StandardHeading::HeadingGeneralCompactVersion(_) => {
                        let matched = heading_modifier.modifier_pattern_regex().captures(content).unwrap();
    
                        let level = HeadingLevel::Explicit(matched.get(1).unwrap().as_str().parse().unwrap());
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


impl LoadingRule<ChapterHeader> for StandardChapterHeaderLoadingRule {
    fn find(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<HashSet<Span<&str>>, LoadError> {
        todo!()     // TODO
    }

    fn load(&self, raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<ChapterHeader, LoadError> {
        todo!()     // TODO
    }
}