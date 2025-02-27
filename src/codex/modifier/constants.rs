use once_cell::sync::Lazy;
use regex::Regex;

pub(super) fn build_strict_reserved_line_pattern(delimiter: &str) -> String {
    format!(r"(?m)^[ \t]*{}[ \t]*$", delimiter)
}

pub(super) const CHAPTER_TAGS_PATTERN: &str = r"(?:\r?\n@(.*))*";
pub(crate) const CHAPTER_STYLE_PATTERN: &str = r"(\r?\n\{(?s:(.*))\})?";
pub(crate) const IDENTIFIER_PATTERN: &str = r"#([\w-]+)";
pub(crate) const NEW_LINE_PATTERN: &str = r"(?:\n|\r\n)";
pub(super) const MULTI_LINES_CONTENT_PATTERN: &str = r"([\s\S]*?)";
pub(super) const MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN: &str = r"(?m)^([^#\n][\s\S]*?)";

pub(crate) const MAX_HEADING_LEVEL: u32 = 6;


pub(crate) const STYLE_PATTERN: &str = r"([^{}]*(?:\.(?:\w+|\d+)\s*|(?:\w+\s*:\s*[^;{}]+\s*;?))*)";
pub(super) const ABRIDGED_STYLE_PATTERN: &str = r"((#?[\w\d\-]+)?;(#?[\w\d\-]+)?;?([\w\d\-]+)?)";

pub(super) static STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(STYLE_PATTERN).unwrap()});
pub(super) static ABRIDGED_STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(ABRIDGED_STYLE_PATTERN).unwrap()});