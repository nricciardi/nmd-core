use once_cell::sync::Lazy;
use regex::Regex;

use super::constants::NEW_LINE_PATTERN;

pub static DOUBLE_NEW_LINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("{}{{2}}", NEW_LINE_PATTERN)).unwrap());










