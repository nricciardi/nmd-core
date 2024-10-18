use once_cell::sync::Lazy;
use regex::Regex;

use crate::codex::modifier::constants::NEW_LINE_PATTERN;


pub static DOUBLE_NEW_LINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("{}{{2}}", NEW_LINE_PATTERN)).unwrap());


pub const SPACE_TAB_EQUIVALENCE: &str = r"   ";

pub static ESCAPE_HTML: Lazy<Vec<(Regex, String)>> = Lazy::new(|| vec![
    (Regex::new(r"<").unwrap(), "&lt;".to_string()),
    (Regex::new(r">").unwrap(), "&gt;".to_string()),
]);