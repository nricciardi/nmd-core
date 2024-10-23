//! `Loader` permits to create `Dossier` or `Document` reading filesystem


pub mod loader_configuration;
pub mod paragraph_loading_rule;
pub mod load_block;


use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use load_block::{LoadBlock, LoadBlockContent};
use getset::{Getters, Setters};
use loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay};
use once_cell::sync::Lazy;
use paragraph_loading_rule::ParagraphLoadingRule;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use thiserror::Error;
use crate::codex::modifier::base_modifier::BaseModifier;
use crate::codex::modifier::standard_heading_modifier::StandardHeading;
use crate::codex::modifier::Modifier;
use crate::content_bundle::ContentBundle;
use crate::resource::disk_resource::DiskResource;
use crate::resource::resource_reference::ResourceReferenceError;
use crate::resource::{Resource, ResourceError};
use super::codex::modifier::constants::CHAPTER_STYLE_PATTERN;
use super::codex::Codex;
use super::dossier::document::chapter::chapter_tag::ChapterTag;
use super::dossier::dossier_configuration::DossierConfiguration;
use super::dossier::Dossier;
use super::dossier::{document::chapter::heading::{Heading, HeadingLevel}, Document};


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

        configuration_overlay.set_document_name(Some(document_name.to_string()));
        
        let mut blocks: Vec<LoadBlock> = Self::load_from_str(content, codex, configuration, configuration_overlay.clone())?;

        blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));

        let document = Self::create_document_by_blocks(document_name, blocks)?;

        log::info!("document '{}' loaded (preamble: {}, chapters: {})", document_name, document.content().preamble().is_empty(), document.content().chapters().len());

        Ok(document)      
    }

    /// Load content from `&str` based on `Codex`
    /// 
    /// Blocks are not sorted, sort if you want:
    /// 
    /// ```rust
    /// blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));
    /// ```
    fn load_from_str(content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Vec<LoadBlock>, LoadError> {
        Self::inner_load_from_str(content, 0, codex, 0, configuration, configuration_overlay.clone())
    }

    /// Inner load method to load content from `&str` based on `Codex`
    /// 
    /// This method uses recursive algorithm, use `content_offset=0` and `paragraph_modifier_index=0` to start.
    fn inner_load_from_str(content: &str, content_offset: usize, codex: &Codex, paragraph_modifier_index: usize, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Vec<LoadBlock>, LoadError> {

        if let Some((modifier_identifier, paragraph_modifier)) = codex.paragraph_modifiers().get_index(paragraph_modifier_index) {

            let paragraph_loading_rule = codex.paragraph_loading_rules().get(modifier_identifier);

            if paragraph_loading_rule.is_none() {

                if configuration.strict_paragraphs_loading_rules_check() {
                    return Err(LoadError::ElaborationError(format!("paragraph loading rule not found for {}", modifier_identifier)));
                }

                log::warn!("{}", format!("paragraph loading rule not found for {}", modifier_identifier));
            }

            let paragraph_loading_rule = paragraph_loading_rule.unwrap();

            let mut current_paragraph_blocks: Vec<LoadBlock> = Vec::new();

            let mut unmatched_slices: Vec<(usize, &str)> = Vec::new();
            let mut last_position: usize = 0;

            // elaborate content based on current paragraph modifier
            for m in paragraph_modifier.modifier_pattern_regex().find_iter(content) {

                assert!(!m.is_empty());

                let m_start = content_offset + m.start();
                let m_end = content_offset + m.end();

                // save previous slice, it will be loaded after
                if m_start > last_position {
                    unmatched_slices.push((last_position, &content[last_position..m_start]));
                }

                last_position = m_end;

                let paragraph = paragraph_loading_rule.load(m.as_str(), codex, configuration, configuration_overlay.clone())?;

                if !paragraph.is_empty() {
                    let block = LoadBlock::new(m_start, m_end, load_block::LoadBlockContent::Paragraph(paragraph));

                    log::debug!("added block:\n{:#?}", block);

                    current_paragraph_blocks.push(block);
                }
            }

            // take last slice (if exists)
            if content.len() > last_position {
                unmatched_slices.push((last_position, &content[last_position..]));
            }

            let mut unmatched_slices_blocks: Vec<LoadBlock> = Vec::new();

            // load unmatched slices
            for (offset, unmatched_slice) in unmatched_slices {
                let mut blocks = Self::inner_load_from_str(unmatched_slice, offset, codex, paragraph_modifier_index + 1, configuration, configuration_overlay.clone())?;
            
                unmatched_slices_blocks.append(&mut blocks);
            }

            current_paragraph_blocks.append(&mut unmatched_slices_blocks);

            return Ok(current_paragraph_blocks)

        } else {    // => there are no other modifiers 

            // load headings
            let mut headings_blocks = Self::load_headings_and_chapter_tags_from_str(content, codex, configuration)?;

            let mut blocks: Vec<LoadBlock> = Vec::new();

            let mut last_position = 0;

            let fallback_loading_rule: Option<&Box<dyn ParagraphLoadingRule>>;

            if let Some(fb_id) = codex.fallback_paragraph_modifier() {
                fallback_loading_rule = codex.paragraph_loading_rules().get(fb_id);
            
            } else {
                fallback_loading_rule = None;

                log::warn!("there isn't fallback paragraph loading rule")
            }

            let mut add_fb_block = |s: &str, start: usize, end: usize| -> Result<(), LoadError> {
                if let Some(rule) = fallback_loading_rule {
                        
                    log::debug!("fallback rule {:?} will be used to load:\n{}", fallback_loading_rule, s);

                    let paragraph = rule.load(s, codex, configuration, configuration_overlay.clone())?;

                    blocks.push(LoadBlock::new(
                        start, 
                        end,
                        LoadBlockContent::Paragraph(paragraph)
                    ));
                }

                Ok(())
            };

            // assign fallback paragraph
            for heading_block in headings_blocks.iter_mut() {

                if heading_block.start() > last_position {

                    let s = &content[last_position..heading_block.start()];

                    add_fb_block(s, content_offset + last_position, content_offset + heading_block.start())?;
                }

                last_position = heading_block.end();

                heading_block.set_start(heading_block.start() + content_offset);
                heading_block.set_end(heading_block.end() + content_offset);
            }

            if content.len() > last_position {

                let s = &content[last_position..];

                add_fb_block(s, content_offset + last_position, content_offset + content.len())?;
            }

            blocks.append(&mut headings_blocks);

            return Ok(blocks);
        }
    }

    fn create_document_by_blocks(document_name: &str, blocks: Vec<LoadBlock>) -> Result<Document, LoadError> {

        log::debug!("create document '{}' using blocks: {:#?}", document_name, blocks);

        let content = ContentBundle::from(blocks);

        let document = Document::new(document_name.to_string(), content);

        log::debug!("document '{}' has {} chapters and preamble {}", document.name(), document.content().chapters().len(), !document.content().preamble().is_empty());

        Ok(document)
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

    /// Load chapter tags (e.g. `author`) from string. This method returns empty `Vec` if there are no tags.
    fn load_chapter_tags_from_str(content: &str, _codex: &Codex, _configuration: &LoaderConfiguration) -> Vec<LoadBlock> {
        
        let mut tags: Vec<LoadBlock> = Vec::new();
        
        let mut pos: usize = 0;
        for line in content.lines() {
            
            let tag = ChapterTag::from_str(line);

            if let Ok(t) = tag {

                tags.push(LoadBlock::new(
                    pos, 
                    pos + line.len(),
                    load_block::LoadBlockContent::ChapterTag(t)
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

    /// Load headings and chapter tags from `&str`
    fn load_headings_and_chapter_tags_from_str(content: &str, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Vec<LoadBlock>, LoadError> {
       
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
                            load_block::LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
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
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            load_block::LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
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
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            load_block::LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::HeadingGeneralExtendedVersion(_) => {
                        let level: u32 = content.chars().take_while(|&c| c == '#').count() as u32;
    
                        let title = capture.get(1).unwrap();
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            load_block::LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
                        );
    
    
                        let tags = Self::load_chapter_tags_from_str(content, codex, configuration);
                    
                        return Ok(Some((heading, tags)))
                    },
    
                    StandardHeading::HeadingGeneralCompactVersion(_) => {
                        let matched = heading_modifier.modifier_pattern_regex().captures(content).unwrap();
    
                        let level: HeadingLevel = matched.get(1).unwrap().as_str().parse().unwrap();
                        let title = capture.get(2).unwrap();
    
                        let heading = LoadBlock::new(
                            title.start(),
                            title.end(),
                            load_block::LoadBlockContent::Heading(Heading::new(level, title.as_str().to_string()))
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

        assert_eq!(document.content().preamble().len(), 1);

        assert_eq!(document.content().chapters().len(), 3);

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

        let paragraphs = Loader::load_from_str(content, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();

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