use std::str::FromStr;
use getset::{Getters, Setters};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::{codex::modifier::constants::CHAPTER_STYLE_PATTERN, load::LoadError};


static FROM_STR_PATTERN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"@(\w+)(?: (.*))?").unwrap());
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
    pub fn load_chapter_tags_from_str(content: &str) -> Result<Vec<ChapterTag>, LoadError> {
        
        let mut tags: Vec<ChapterTag> = Vec::new();
        
        for line in content.lines() {

            if line.trim().is_empty() {
                continue;
            }

            let tag = ChapterTag::from_str(&line);

            if let Ok(t) = tag {

                tags.push(t);

            } else {

                return Err(LoadError::InvalidTag(tag.err().unwrap()))
            }
        }

        Ok(tags)
    }

    #[allow(dead_code)]
    /// Load the chapter style from string
    fn load_chapter_style_from_str(content: &str) -> Option<String> {
        
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
            "class" => Ok(Self::StyleClass),

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

                if let Ok(key) = ChapterTagKey::from_str(key.as_str()) {

                    chapter_tag.set_key(key);
                
                } else {
                    return Err(s.to_string())
                }

                if let Some(value) = captures.get(2) {
                    chapter_tag.set_value(Some(value.as_str().to_string()));
                }

                return Ok(chapter_tag)
            }
        }
        
        Err(s.to_string())
    }
}


#[cfg(test)]
mod test {
    use super::ChapterTag;


    #[test]
    fn load_tags() {
        let s = concat!(
            "\r\n",
            "@style color:red\n",
            "@class class\r\n",
            "\t\r\n"
        );

        let tags = ChapterTag::load_chapter_tags_from_str(s).unwrap();

        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn load_empty() {
        let s = concat!(
            ""
        );

        let tags = ChapterTag::load_chapter_tags_from_str(s).unwrap();

        assert_eq!(tags.len(), 0);

        let s = concat!(
            "\r\n"
        );

        let tags = ChapterTag::load_chapter_tags_from_str(s).unwrap();

        assert_eq!(tags.len(), 0);
    }

}