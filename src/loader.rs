//! `Loader` permits to create `Dossier` or `Document` reading filesystem


pub mod loader_configuration;
pub mod paragraph_loading_rule;
mod block;


use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use block::Block;
use getset::{Getters, Setters};
use loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay};
use once_cell::sync::Lazy;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use thiserror::Error;
use crate::codex::modifier::base_modifier::BaseModifier;
use crate::codex::modifier::constants::{INCOMPATIBLE_CHAPTER_HEADING_REGEX, NEW_LINE};
use crate::codex::modifier::standard_heading_modifier::StandardHeading;
use crate::codex::modifier::Modifier;
use crate::dossier::document::chapter::paragraph::Paragraph;
use crate::resource::disk_resource::DiskResource;
use crate::resource::resource_reference::ResourceReferenceError;
use crate::resource::{Resource, ResourceError};
use super::codex::modifier::constants::CHAPTER_STYLE_PATTERN;
use super::codex::Codex;
use super::dossier::document::chapter::chapter_tag::ChapterTag;
use super::dossier::dossier_configuration::DossierConfiguration;
use super::dossier::Dossier;
use super::dossier::{document::{chapter::heading::{Heading, HeadingLevel}, Chapter}, Document};


static CHAPTER_STYLE_PATTERN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(CHAPTER_STYLE_PATTERN).unwrap());
static DOUBLE_NEW_LINES: Lazy<String> = Lazy::new(|| format!("{}{}", NEW_LINE, NEW_LINE));
static TRIPLE_NEW_LINES: Lazy<String> = Lazy::new(|| format!("{}{}{}", NEW_LINE, NEW_LINE, NEW_LINE));


#[derive(Error, Debug)]
pub enum LoadError {
    #[error(transparent)]
    ResourceError(#[from] ResourceError),

    #[error(transparent)]
    ResourceReferenceError(#[from] ResourceReferenceError),

    #[error("elaboration error: {0}")]
    ElaborationError(String),
    
    #[error(transparent)]
    IoError(#[from] io::Error)
}

impl Clone for LoadError {
    fn clone(&self) -> Self {
        match self {
            Self::IoError(e) => Self::ElaborationError(e.to_string()),
            other => other.clone()
        }
    }
}


#[derive(Debug, Getters, Setters)]
pub struct Loader {
}

impl Loader {

    /// Count number of newlines at string start.
    /// It counts number of '\n' characters
    fn count_newlines_at_start(s: &str) -> usize {
        s.bytes().take_while(|&b| b == b'\n').count()
    }

    /// Count number of newlines at string end.
    /// It counts number of '\n' characters
    fn count_newlines_at_end(s: &str) -> usize {
        s.bytes().rev().take_while(|&b| b == b'\n').count()
    }

    pub fn load_document_from_str(document_name: &str, content: &str, codex: &Codex, configuration: &LoaderConfiguration, mut configuration_overlay: LoaderConfigurationOverLay) -> Result<Document, LoadError> {

        log::info!("loading document '{}' from its content...", document_name);

        configuration_overlay.set_document_name(Some(document_name.to_string()));

        let paragraphs = Self::load_paragraphs_from_str(content, codex, configuration, configuration_overlay.clone())?;

        let mut incompatible_ranges: Vec<(usize, usize)> = paragraphs.par_iter().map(|p| (p.start(), p.end())).collect();

        incompatible_ranges.par_sort_by(|a, b| a.0.cmp(&b.0));

        let headings_and_chapter_tags = Self::load_headings_and_chapter_tags_from_str(content, codex, incompatible_ranges, configuration)?;

        let mut blocks: Vec<Block> = Vec::new();
    
        blocks.append(&mut paragraphs);
        blocks.append(&mut headings_and_chapter_tags);

        blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));

        
    }

    /// Load a document from a raw string, so `document_name` must be provided
    /*pub fn load_document_from_str(document_name: &str, content: &str, codex: &Codex, configuration: &LoaderConfiguration, mut configuration_overlay: LoaderConfigurationOverLay) -> Result<Document, LoadError> {

        log::info!("loading document '{}' from its content...", document_name);

        configuration_overlay.set_document_name(Some(document_name.to_string()));

        let mut content = String::from(content);

        // work-around to fix paragraph matching end line
        while !content.starts_with(&(*DOUBLE_NEW_LINES)) {
            content.insert_str(0, NEW_LINE);
        }

        while !content.ends_with(&(*DOUBLE_NEW_LINES)) {
            content.push_str(NEW_LINE);
        }

        let mut document_chapters: Vec<Chapter> = Vec::new();

        log::debug!("start to find chapter borders in document '{}'", document_name);

        // usize: chapter start/end position
        let mut incompatible_chapter_heading_borders: Vec<(usize, usize)> = Vec::new();

        INCOMPATIBLE_CHAPTER_HEADING_REGEX.iter().for_each(|regex| {            // TODO: par_iter
            regex.find_iter(&content).for_each(|m| {
                incompatible_chapter_heading_borders.push((m.start(), m.end()));
            });
        });

        // usize: chapter start/end position
        // String: chapter heading + options found
        let mut chapter_borders: Vec<(usize, usize, String)> = Vec::new();

        for heading in StandardHeading::ordered() {

            let heading_modifier = Into::<BaseModifier>::into(heading);

            heading_modifier.modifier_pattern_regex().find_iter(content.as_str()).for_each(|m| {

                let matched_str = m.as_str().to_string();

                let start = m.start();
                let end = m.end();

                let overlap_chapter = chapter_borders.par_iter().find_any(|c| {
                    (c.0 >= start && c.1 <= end) ||     // current paragraph contains p
                    (c.0 <= start && c.1 >= end) ||     // p contains current paragraph
                    (c.0 <= start && c.1 >= start && c.1 <= end) ||     // left overlap
                    (c.0 >= start && c.0 <= end && c.1 >= end)          // right overlap
                });

                if let Some(p) = overlap_chapter {     // => overlap
                    log::debug!("discarded chapter because there is an overlap between {} and {} using pattern {:?}:\n{:#?}\n", start, end, heading_modifier, p);
                    return
                }

                let not_heading = incompatible_chapter_heading_borders.par_iter().find_any(|border| {
                    (border.0 <= start && border.1 >= start) ||
                    (border.0 <= end && border.1 >= end)
                });

                if let Some(p) = not_heading {     // => overlap
                    log::debug!("discarded chapter:\n{}\nbecause there is in an incompatible slice between {} and {} ({:#?})", m.as_str(), start, end, p);
                    return
                }

                log::debug!("find chapter between {} and {}: {:?}", start, end, &matched_str);

                let cb = (
                    start,
                    end,
                    matched_str
                );

                log::debug!("push in chapter_borders: {:?}", cb);

                chapter_borders.push(cb);

            });
        }

        chapter_borders.par_sort_by(|a, b| a.0.cmp(&b.0));

        log::debug!("loading {} chapters of document '{}'...", chapter_borders.len(), document_name);

        let mut last_heading_level: HeadingLevel = 0;

        // build chapters
        for index in 0..chapter_borders.len() {

            log::debug!("load chapter {:?}", chapter_borders[index]);

            let _start = chapter_borders[index].0;
            let end = chapter_borders[index].1;
            let raw_content = &chapter_borders[index].2;

            let (heading, tags) = Self::load_chapter_heading_and_tags_from_str(raw_content, last_heading_level, codex, configuration);

            if heading.is_none() {
                return Err(LoadError::ResourceError(ResourceError::ResourceNotFound("heading".to_string())))
            }

            let heading = heading.unwrap();

            last_heading_level = heading.level();

            let mut next_start: usize = content.len();

            if index < chapter_borders.len() - 1 {
                next_start = chapter_borders[index + 1].0;
            }

            let sub_content = content.get(end..next_start).unwrap();     // exclude heading

            let paragraphs = Self::load_paragraphs_from_str(sub_content, codex, configuration, configuration_overlay.clone())?;

            document_chapters.push(Chapter::new(heading, tags, paragraphs));
        }

        let mut preamble_end = content.len();

        if chapter_borders.len() > 0 {
            preamble_end = chapter_borders[0].0;
        }

        let preamble: Vec<Box<dyn Paragraph>>;
        
        if preamble_end > 0 {      // => there is a preamble
            
            log::debug!("preamble found in document '{}'", document_name);

            let s = String::from(content.get(0..preamble_end).unwrap());

            preamble = Self::load_paragraphs_from_str(&s, codex, configuration, configuration_overlay.clone())?;
        
        } else {

            log::debug!("preamble not found in document '{}'", document_name);

            preamble = Vec::new();
        }

        log::info!("document '{}' loaded", document_name);

        Ok(Document::new(document_name.to_string(), preamble, document_chapters))

    }*/

    /// Load a document from its path (`PathBuf`). The document have to exist.
    pub fn load_document_from_path(path_buf: &PathBuf, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Document, LoadError> {

        if !path_buf.exists() {
            return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(format!("{} not exists", path_buf.to_string_lossy())))) 
        }

        let resource = DiskResource::try_from(path_buf.clone())?;

        let content = resource.content()?;

        let document_name = resource.name();

        match Self::load_document_from_str(document_name, &content, codex, configuration, configuration_overlay.clone()) {
            Ok(document) => {
                return Ok(document)
            },
            Err(err) => return Err(LoadError::ElaborationError(err.to_string()))
        }
    }

    /// Load paragraphs from `&str` using `Codex`. Paragraphs are returned not in order, you should use `start` and `end` to sort if you want.
    pub fn load_paragraphs_from_str(content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Vec<Block>, LoadError> {

        if content.trim().is_empty() {
            log::debug!("skip paragraphs loading: empty content");
            return Ok(Vec::new());
        }

        log::debug!("loading paragraph:\n{}", content);

        let mut paragraphs: Vec<Block> = Vec::new();

        for (codex_identifier, paragraph_modifier) in codex.paragraph_modifiers() {

            let search_pattern = paragraph_modifier.modifier_pattern();

            log::debug!("searching paragraph '{}': {:?}", codex_identifier, search_pattern);

            for m in paragraph_modifier.modifier_pattern_regex().find_iter(content) {

                assert!(!m.is_empty());

                let m_start = m.start();
                let m_end = m.end();
                
                log::debug!("found paragraph using '{}': {:?} between {} and {}:\n{}", codex_identifier, search_pattern, m_start, m_end, m.as_str());

                let overlap_paragraph = paragraphs.par_iter().find_any(|p| {

                    let p_start = p.start();
                    let p_end = p.end();

                    (p_start >= m_start && p_end <= m_end) ||     // current paragraph contains p
                    (p_start <= m_start && p_end >= m_end) ||     // p contains current paragraph
                    (p_start <= m_start && p_end >= m_start && p_end <= m_end) ||     // left overlap
                    (p_start >= m_start && p_start <= m_end && p_end >= m_end)          // right overlap
                });

                if let Some(p) = overlap_paragraph {     // => overlap
                    log::debug!("paragraph discarded because there is an overlap between {} and {} using pattern {:?}:\n{:#?}\n", m_start, m_end, search_pattern, p);
                    continue;
                }

                if let Some(loading_rule) = codex.paragraph_loading_rules().get(codex_identifier) {

                    let paragraph = loading_rule.load(m.as_str(), codex, configuration, configuration_overlay.clone())?;

                    if !paragraph.is_empty() {
                        log::debug!("added paragraph to paragraphs list:\n{:#?}", paragraph);
                        
                        paragraphs.push(Block::new(m_start, m_end, block::BlockContent::Paragraph(paragraph)));
                    }

                } else {

                    return Err(LoadError::ElaborationError(format!("paragraph content loading rule not found for {}", codex_identifier)))
                }

            };
        }

        // paragraphs.par_sort_by(|a, b| a.start().cmp(&b.start()));

        Ok(paragraphs)
    }


    /// Load chapter tags (e.g. `author`) from string. This method returns empty `Vec` if there are no tags.
    fn load_chapter_tags_from_str(content: &str, _codex: &Codex, _configuration: &LoaderConfiguration) -> Vec<Block> {
        
        let mut tags: Vec<Block> = Vec::new();
        
        let mut pos: usize = 0;
        for line in content.lines() {
            
            let tag = ChapterTag::from_str(line);

            if let Ok(t) = tag {

                tags.push(Block::new(
                    pos, 
                    pos + line.len(),
                    block::BlockContent::ChapterTag(t)
                ));

            }

            pos += line.len();
        }

        tags
    }

    #[allow(dead_code)]
    /// Load the chapter style from string
    fn load_chapter_style_from_str(content: &str, _codex: &Codex, _configuration: &LoaderConfiguration) -> Option<String> {
        
        let mut style: Option<String> = None;

        if let Some(captures) = CHAPTER_STYLE_PATTERN_REGEX.captures(content) {
            if let Some(s) = captures.get(1) {
                style = Some(s.as_str().to_string())
            }
        }

        style
    }

    /// Load headings and chapter tags from `&str` not in incompatible ranges.
    /// 
    /// Assuming that ranges of `incompatible_ranges` are correct.
    fn load_headings_and_chapter_tags_from_str(content: &str, codex: &Codex, incompatible_ranges: Vec<(usize, usize)>, configuration: &LoaderConfiguration) -> Result<Vec<Block>, LoadError> {
       
        incompatible_ranges.par_iter().for_each(|range| assert!(range.0 <= range.1));

        let mut last_heading_level = 0;
        let mut headings_and_chapter_tags: Vec<Block> = Vec::new();

        for heading in StandardHeading::ordered() {     // TODO: include `StandardHeading::ordered()` in `Codex`

            let heading_modifier = Into::<BaseModifier>::into(heading);

            for m in heading_modifier.modifier_pattern_regex().find_iter(content) {

                let matched_str = m.as_str().to_string();

                let m_start = m.start();
                let m_end = m.end();

                let incompatible = incompatible_ranges.par_iter().find_any(|range| {

                    let r_start = range.0;
                    let r_end = range.1;

                    (r_start >= m_start && r_end <= m_end) ||     // current paragraph contains p
                    (r_start <= m_start && r_end >= m_end) ||     // p contains current paragraph
                    (r_start <= m_start && r_end >= m_start && r_end <= m_end) ||     // left overlap
                    (r_start >= m_start && r_start <= m_end && r_end >= m_end)          // right overlap
                });

                if let Some(i) = incompatible {     // => incompatible
                    log::debug!("discarded heading or chapter tag block [{}, {}] because it is contained in an incompatible range [{}, {}]", m_start, m_end, i.0, i.1);
                    continue
                }

                let overlap = headings_and_chapter_tags.par_iter().find_any(|b| {

                    let b_start = b.start();
                    let b_end = b.end();

                    (b_start >= m_start && b_end <= m_end) ||     // current paragraph contains p
                    (b_start <= m_start && b_end >= m_end) ||     // p contains current paragraph
                    (b_start <= m_start && b_end >= m_start && b_end <= m_end) ||     // left overlap
                    (b_start >= m_start && b_start <= m_end && b_end >= m_end)          // right overlap
                });

                if let Some(o) = overlap {     // => overlap
                    log::debug!("discarded heading or chapter tag block because there is an overlap with another heading/tag block between {} and {} using pattern {:?}:\n{:#?}\n", m_start, m_end, heading_modifier, o);
                    continue
                }

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

    // TODO: return Option of (heading, tags)
    // TODO: doesn't use section of content in which there is also a paragraph
    /// Load the chapter heading and metadata from `&str`. This method returns a tuple with optional heading and a chapter tags vector.
    fn parse_chapter_heading_and_tags_from_str(content: &str, last_heading_level: &mut HeadingLevel, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Option<(Block, Vec<Block>)>, LoadError> {

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
    
                        let heading = Block::new(
                            title.start(),
                            title.end(),
                            block::BlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::MajorHeading => {
    
                        let mut level: HeadingLevel = *last_heading_level + 1;
    
                        if level < 1 {
                            log::warn!("level {} < 0, so it is set as 1", level);
                            level = 1;
                        }
    
                        let title = capture.get(1).unwrap();
    
                        let heading = Block::new(
                            title.start(),
                            title.end(),
                            block::BlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
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
    
                        let heading = Block::new(
                            title.start(),
                            title.end(),
                            block::BlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::HeadingGeneralExtendedVersion(_) => {
                        let level: u32 = content.chars().take_while(|&c| c == '#').count() as u32;
    
                        let title = capture.get(1).unwrap();
    
                        let heading = Block::new(
                            title.start(),
                            title.end(),
                            block::BlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::HeadingGeneralCompactVersion(_) => {
                        let matched = heading_modifier.modifier_pattern_regex().captures(content).unwrap();
    
                        let level: HeadingLevel = matched.get(1).unwrap().as_str().parse().unwrap();
                        let title = capture.get(2).unwrap();
    
                        let heading = Block::new(
                            title.start(),
                            title.end(),
                            block::BlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
                }   
            }

        }
        
        Ok(None)
    }

    /// Load dossier from its filesystem path
    pub fn load_dossier_from_path_buf(path_buf: &PathBuf, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Dossier, LoadError> {
        let dossier_configuration = DossierConfiguration::try_from(path_buf)?;

        Self::load_dossier_from_dossier_configuration(&dossier_configuration, codex, configuration, configuration_overlay.clone())
    }

    /// Load dossier from its filesystem path considering only a subset of documents
    pub fn load_dossier_from_path_buf_only_documents(path_buf: &PathBuf, only_documents: &HashSet<String>, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Dossier, LoadError> {
        let mut dossier_configuration = DossierConfiguration::try_from(path_buf)?;

        let d: Vec<String> = dossier_configuration.raw_documents_paths().iter()
                                                    .filter(|item| {

                                                        let file_name = PathBuf::from(*item).file_name().unwrap().to_string_lossy().to_string();

                                                        only_documents.contains(file_name.as_str())
                                                    })
                                                    .map(|item| item.clone())
                                                    .collect();

        dossier_configuration.set_raw_documents_paths(d);

        let mut configuration_overlay = configuration_overlay.clone();

        configuration_overlay.set_dossier_name(Some(dossier_configuration.name().clone()));

        Self::load_dossier_from_dossier_configuration(&dossier_configuration, codex, configuration, configuration_overlay)
    }

    /// Load dossier from its dossier configuration
    pub fn load_dossier_from_dossier_configuration(dossier_configuration: &DossierConfiguration, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Dossier, LoadError> {

        // TODO: are really mandatory?
        if dossier_configuration.documents_paths().is_empty() {
            return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose("there are no documents".to_string())))
        }

        // TODO: is really mandatory?
        if dossier_configuration.name().is_empty() {
            return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose("there is no name".to_string())))
        }

        if dossier_configuration.compilation().parallelization() {

            let mut documents_res: Vec<Result<Document, LoadError>> = Vec::new();

            dossier_configuration.documents_paths().par_iter()
            .map(|document_path| {
                Self::load_document_from_path(&PathBuf::from(document_path), codex, configuration, configuration_overlay.clone())
            }).collect_into_vec(&mut documents_res);
            
            let error = documents_res.par_iter().find_any(|result| result.is_err());

            // handle errors
            if let Some(Err(err)) = error.as_ref() {
                return Err(err.clone())
            }

            let documents = documents_res.into_iter().map(|d| d.unwrap()).collect();

            return Ok(Dossier::new(dossier_configuration.clone(), documents))


        } else {

            let mut documents: Vec<Document> = Vec::new();

            for document_path in dossier_configuration.documents_paths() {
    
                let document = Self::load_document_from_path(&PathBuf::from(document_path), codex, configuration, configuration_overlay.clone())?;
    
                documents.push(document)
            }

            return Ok(Dossier::new(dossier_configuration.clone(), documents))
        }
    }
}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn chapters_from_str() {

        let codex = Codex::of_html();

        let content: String = 
r#"
preamble

# title 1a

paragraph 1a

## title 2a

paragraph 2a

# title 1b

paragraph 1b
"#.trim().to_string();

        let document = Loader::load_document_from_str("test", &content, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();

        assert_eq!(document.preamble().len(), 1);

        assert_eq!(document.chapters().len(), 3);

    }

    #[test]
    fn paragraphs_from_str() {
        let content = concat!(
            "paragraph1",
            "\n\n",
            "paragraph2a\nparagraph2b",
            "\n\n",
            "paragraph3",
        );

        let codex = Codex::of_html();

        let paragraphs = Loader::load_paragraphs_from_str(content, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();

        assert_eq!(paragraphs.len(), 3)
    }

    #[test]
    fn load_dossier() {

        let dossier_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("nmd-test-dossier-1");

        let codex = Codex::of_html();

        let loader_configuration = LoaderConfiguration::default();

        let _dossier = Loader::load_dossier_from_path_buf(&dossier_path, &codex, &loader_configuration, LoaderConfigurationOverLay::default()).unwrap();
    }
}