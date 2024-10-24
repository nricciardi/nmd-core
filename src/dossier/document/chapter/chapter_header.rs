use getset::{Getters, MutGetters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Serialize;
use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_heading_modifier::StandardHeading, Modifier}, Codex}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, self_compile::SelfCompile}, load::{load_block::{LoadBlock, LoadBlockContent}, loader_configuration::LoaderConfiguration, LoadError}, output_format::OutputFormat};
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
    pub fn load_headings_and_chapter_tags_from_str(content: &str, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Vec<LoadBlock>, LoadError> {
       
        let mut last_heading_level = 0;
        let mut headings_and_chapter_tags: Vec<LoadBlock> = Vec::new();

        for heading in StandardHeading::ordered() {     // TODO: include `StandardHeading::ordered()` in `Codex`

            let heading_modifier = Into::<BaseModifier>::into(heading);

            for m in heading_modifier.modifier_pattern_regex().find_iter(content) {

                let matched_str = m.as_str().to_string();

                let m_start = m.start();
                let m_end = m.end();

                log::debug!("chapter found between {} and {}: {:?}", m_start, m_end, &matched_str);

                if let Some((mut heading, mut tags)) = Self::parse_chapter_heading_and_tags_from_str(content, &mut last_heading_level, codex, configuration)? {

                    heading.set_start(heading.start() + m_start);
                    heading.set_end(heading.end() + m_start);

                    tags.par_iter_mut().for_each(|tag| {
                        tag.set_start(tag.start() + m_start);
                        tag.set_end(tag.end() + m_start);
                    });

                    headings_and_chapter_tags.push(heading);
                    headings_and_chapter_tags.append(&mut tags);
                }

            };
        }

        // headings_and_chapter_tags.par_sort_by(|a, b| a.start().cmp(&b.start()));

        Ok(headings_and_chapter_tags)
    }

    /// Load the chapter heading and metadata from `&str`. This method returns a tuple with optional heading and a chapter tags vector.
    fn parse_chapter_heading_and_tags_from_str(content: &str, last_heading_level: &mut HeadingLevel, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Option<(LoadBlock, Vec<LoadBlock>)>, LoadError> {

        log::debug!("parse headings and chapter tags from (last heading level: {}):\n{}", last_heading_level, content);

        for heading in StandardHeading::ordered() {

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
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::MajorHeading => {
    
                        let mut level: HeadingLevel = *last_heading_level + 1;
    
                        if level < 1 {
                            log::warn!("level {} < 0, so it is set as 1", level);
                            level = 1;
                        }
    
                        let title = capture.get(1).unwrap();
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
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
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::HeadingGeneralExtendedVersion(_) => {
                        let level: u32 = content.chars().take_while(|&c| c == '#').count() as u32;
    
                        let title = capture.get(1).unwrap();
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::HeadingGeneralCompactVersion(_) => {
                        let matched = heading_modifier.modifier_pattern_regex().captures(content).unwrap();
    
                        let level: HeadingLevel = matched.get(1).unwrap().as_str().parse().unwrap();
                        let title = capture.get(2).unwrap();
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = ChapterTag::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
                }   
            }

        }
        
        Ok(None)
    }

}

impl SelfCompile for ChapterHeader {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
    
        self.heading.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
    }
}