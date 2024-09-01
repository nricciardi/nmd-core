use once_cell::sync::Lazy;
use regex::Regex;
use super::standard_paragraph_modifier::StandardParagraphModifier;


pub const CHAPTER_TAGS_PATTERN: &str = r"(?:\r?\n@(.*))*";
pub const CHAPTER_STYLE_PATTERN: &str = r"(\r?\n\{(?s:(.*))\})?";
pub const IDENTIFIER_PATTERN: &str = r"#([\w-]+)";

pub const MAX_HEADING_LEVEL: u32 = 6;

#[cfg(windows)]
pub const NEW_LINE: &str = "\r\n";

#[cfg(not(windows))]
pub const NEW_LINE: &str = "\n";


pub static INCOMPATIBLE_CHAPTER_HEADING_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(&StandardParagraphModifier::CodeBlock.modifier_pattern()).unwrap(),
        Regex::new(&StandardParagraphModifier::MathBlock.modifier_pattern()).unwrap(),
        Regex::new(&StandardParagraphModifier::FocusBlock.modifier_pattern()).unwrap(),
        Regex::new(&StandardParagraphModifier::ExtendedBlockQuote.modifier_pattern()).unwrap(),
    ]
});

pub const STYLE_PATTERN: &str = r"([^{}]*(?:\.(?:\w+|\d+)\s*|(?:\w+\s*:\s*[^;{}]+\s*;?))*)";
pub const ABRIDGED_STYLE_PATTERN: &str = r"((#?[\w\d\-]+)?;(#?[\w\d\-]+)?;?([\w\d\-]+)?)";

pub static STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(STYLE_PATTERN).unwrap()});
pub static ABRIDGED_STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(ABRIDGED_STYLE_PATTERN).unwrap()});