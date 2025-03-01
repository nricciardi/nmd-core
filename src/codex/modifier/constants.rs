use once_cell::sync::Lazy;
use regex::Regex;

pub fn build_strict_reserved_line_pattern(delimiter: &str) -> String {
    format!(r"(?m)^[ \t]*{}[ \t]*$", delimiter)
}

pub const CHAPTER_TAGS_PATTERN: &str = r"(?:\r?\n@(.*))*";
pub const CHAPTER_STYLE_PATTERN: &str = r"(\r?\n\{(?s:(.*))\})?";
pub const IDENTIFIER_PATTERN: &str = r"#([\w-]+)";
pub const NEW_LINE_PATTERN: &str = r"(?:\n|\r\n)";
pub const MULTI_LINES_CONTENT_PATTERN: &str = r"([\s\S]*?)";
pub const MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN: &str = r"(?m)^([^#\n][\s\S]*?)";

pub const MAX_HEADING_LEVEL: u32 = 6;


pub const STYLE_PATTERN: &str = r"([^{}]*(?:\.(?:\w+|\d+)\s*|(?:\w+\s*:\s*[^;{}]+\s*;?))*)";
pub const ABRIDGED_STYLE_PATTERN: &str = r"((#?[\w\d\-]+)?;(#?[\w\d\-]+)?;?([\w\d\-]+)?)";

pub static STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(STYLE_PATTERN).unwrap()});
pub static ABRIDGED_STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(ABRIDGED_STYLE_PATTERN).unwrap()});