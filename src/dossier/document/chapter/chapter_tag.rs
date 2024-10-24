use std::str::FromStr;
use getset::{Getters, Setters};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::{codex::{modifier::constants::CHAPTER_STYLE_PATTERN, Codex}, load::{load_block::{LoadBlock, LoadBlockContent}, loader_configuration::LoaderConfiguration}};


static FROM_STR_PATTERN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"@(\w+) (.*)").unwrap());
static CHAPTER_STYLE_PATTERN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(CHAPTER_STYLE_PATTERN).unwrap());


#[derive(Debug, Clone, Getters, Setters, Serialize, Deserialize)]
pub struct ChapterTag {

    #[getset(get = "pub", set = "pub")]
    key: ChapterTagKey,

    #[getset(get = "pub", set = "pub")]
    value: Option<String>
}

impl ChapterTag {

    /// Load chapter tags (e.g. `author`) from string. This method returns empty `Vec` if there are no tags.
    pub fn load_chapter_tags_from_str(content: &str, _codex: &Codex, _configuration: &LoaderConfiguration) -> Vec<LoadBlock> {
        
        let mut tags: Vec<LoadBlock> = Vec::new();
        
        let mut pos: usize = 0;
        for line in content.lines() {
            
            let tag = ChapterTag::from_str(line);

            if let Ok(t) = tag {

                tags.push(LoadBlock::new(
                    pos, 
                    pos + line.len(),
                    LoadBlockContent::ChapterTag(t)
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
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChapterTagKey {
    Id,
    Author,
    Date,
    Intent,
    Style,
    StyleClass,
    None
}

impl FromStr for ChapterTagKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "id" => Ok(Self::Id),
            "author" => Ok(Self::Author),
            "date" => Ok(Self::Date),
            "intent" => Ok(Self::Intent),
            "style" => Ok(Self::Style),
            "styleclass" => Ok(Self::StyleClass),

            _ => Err(format!("chapter key '{}' not found", s))
        }
    }
}

impl Default for ChapterTag {
    fn default() -> Self {
        Self { key: ChapterTagKey::None, value: None }
    }
}


impl FromStr for ChapterTag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let captures = FROM_STR_PATTERN_REGEX.captures(s);

        if let Some(captures) = captures {

            if let Some(key) = captures.get(1) {

                let mut chapter_tag = ChapterTag::default();
                chapter_tag.set_key(ChapterTagKey::from_str(key.as_str())?);

                if let Some(value) = captures.get(2) {
                    chapter_tag.set_value(Some(value.as_str().to_string()));
                }

                return Ok(chapter_tag)
            }
        }
        
        Err(format!("{} is not a valid tag", s))
    }
}