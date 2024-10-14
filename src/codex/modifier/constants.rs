use once_cell::sync::Lazy;
use regex::Regex;


pub const CHAPTER_TAGS_PATTERN: &str = r"(?:\r?\n@(.*))*";
pub const CHAPTER_STYLE_PATTERN: &str = r"(\r?\n\{(?s:(.*))\})?";
pub const IDENTIFIER_PATTERN: &str = r"#([\w-]+)";

pub const MAX_HEADING_LEVEL: u32 = 6;

pub const NEW_LINE: &str = "\n";


pub const STYLE_PATTERN: &str = r"([^{}]*(?:\.(?:\w+|\d+)\s*|(?:\w+\s*:\s*[^;{}]+\s*;?))*)";
pub const ABRIDGED_STYLE_PATTERN: &str = r"((#?[\w\d\-]+)?;(#?[\w\d\-]+)?;?([\w\d\-]+)?)";

pub static STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(STYLE_PATTERN).unwrap()});
pub static ABRIDGED_STYLE_REGEX: Lazy<Regex> = Lazy::new(|| {Regex::new(ABRIDGED_STYLE_PATTERN).unwrap()});