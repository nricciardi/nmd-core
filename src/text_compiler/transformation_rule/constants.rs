use once_cell::sync::Lazy;
use regex::Regex;


pub static ESCAPE_HTML: Lazy<Vec<(Regex, String)>> = Lazy::new(|| vec![
    (Regex::new(r"<").unwrap(), "&lt;".to_string()),
    (Regex::new(r">").unwrap(), "&gt;".to_string()),
]);