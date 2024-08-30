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