//! `Loader` permits to create `Dossier` or `Document` reading filesystem


pub mod loader_configuration;
pub mod paragraph_loading_rule;
pub mod block;


use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use block::{Block, BlockContent};
use getset::{Getters, Setters};
use loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay};
use once_cell::sync::Lazy;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use thiserror::Error;
use crate::codex::modifier::base_modifier::BaseModifier;
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


#[derive(Error, Debug)]
pub enum LoadError {
    #[error(transparent)]
    ResourceError(#[from] ResourceError),

    #[error(transparent)]
    ResourceReferenceError(#[from] ResourceReferenceError),

    #[error("elaboration error: {0}")]
    ElaborationError(String),
    
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("block error: {0}")]
    BlockError(String)
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


    pub fn load_document_from_str(document_name: &str, content: &str, codex: &Codex, configuration: &LoaderConfiguration, mut configuration_overlay: LoaderConfigurationOverLay) -> Result<Document, LoadError> {

        log::info!("loading document '{}' from its content...", document_name);

        // TODO: is needed?
        // let content = &text_utility::normalize_newlines(content);

        configuration_overlay.set_document_name(Some(document_name.to_string()));

        let mut paragraphs = Self::load_paragraphs_from_str(content, codex, configuration, configuration_overlay.clone())?;

        let mut incompatible_ranges: Vec<(usize, usize)> = paragraphs.par_iter().map(|p| (p.start(), p.end())).collect();

        incompatible_ranges.par_sort_by(|a, b| a.0.cmp(&b.0));

        let mut headings_and_chapter_tags = Self::load_headings_and_chapter_tags_from_str(content, codex, incompatible_ranges, configuration)?;

        let mut blocks: Vec<Block> = Vec::new();
    
        blocks.append(&mut paragraphs);
        blocks.append(&mut headings_and_chapter_tags);

        let document = Self::create_document_by_blocks(document_name, blocks)?;

        log::info!("document '{}' loaded (preamble: {}, chapters: {})", document_name, document.preamble().is_empty(), document.chapters().len());

        Ok(document)      
    }

    fn create_document_by_blocks(document_name: &str, mut blocks: Vec<Block>) -> Result<Document, LoadError> {

        if !blocks.windows(2).all(|w| w[0].start() <= w[1].start()) {
            
            blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));
        }

        log::debug!("create document '{}' using blocks: {:?}", document_name, blocks);

        let mut preamble: Vec<Box<dyn Paragraph>> = Vec::new();
        let mut current_chapter: Option<Chapter> = None;
        let mut chapters: Vec<Chapter> = Vec::new(); 

        for block in blocks {

            match Into::<BlockContent>::into(block) {
                BlockContent::Paragraph(paragraph) => {

                    if let Some(ref mut cc) = current_chapter {

                        cc.paragraphs_mut().push(paragraph);

                    } else {
                        preamble.push(paragraph);
                    }

                },
                BlockContent::Heading(heading) => {

                    if let Some(cc) = current_chapter.take() {
                        chapters.push(cc);
                    }

                    current_chapter = Some(Chapter::new(heading, Vec::new(), Vec::new()));
                },
                BlockContent::ChapterTag(chapter_tag) => {

                    assert!(current_chapter.is_some());

                    current_chapter.as_mut().unwrap().tags_mut().push(chapter_tag);

                },
            }
        }

        
        Ok(Document::new(document_name.to_string(), preamble, chapters))
    }


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

    /// Load paragraphs from `&str` using `Codex`.
    /// 
    /// Paragraphs are returned not in order, you should use `start` and `end` to sort if you want.
    /// 
    /// Assume `\n` instead of `\r\n` as new line, if you are on Windows please replace `\r\n` before use this method.
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
    /// 
    /// Assume `\n` instead of `\r\n` as new line, if you are on Windows please replace `\r\n` before use this method (you can use `normalize_newlines` of utilities).
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