use regex::Regex;

use super::nmd_unique_identifier::NmdUniqueIdentifier;



pub fn replace(content: &str, replacements: &Vec<(Regex, String)>) -> String {
    let mut result = String::from(content);

    for (regex, rep) in replacements {
        result = regex.replace_all(&result, rep).to_string();
    }

    result
}

pub fn html_nuid_tag_or_nothing(nuid: Option<&NmdUniqueIdentifier>) -> String {
    if let Some(nuid) = nuid {
        return format!(r#" data-nuid="{}""#, nuid);
      }

      String::new()
}

/// return styles and classes
pub fn split_styles_and_classes(content: &str) -> (String, String) {
    let mut styles = String::new();
    let mut classes = String::new();
    
    for item in content.split(";") {
        let item = item.trim();

        if item.starts_with(".") {
        
            classes.push_str(&item[1..item.len()]);
            classes.push_str(" ");
        
        } else {

            styles.push_str(item);
            styles.push_str("; ");
        }

    }

    (styles, classes)
}