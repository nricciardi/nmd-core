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

pub type Styles = String;
pub type Classes = String;

/// return styles and classes
pub fn split_styles_and_classes_with_default(content: &str, default: (Option<Styles>, Option<Classes>)) -> (Option<Styles>, Option<Classes>) {
    let mut styles = default.0.unwrap_or(Styles::new());
    let mut classes = default.1.unwrap_or(Styles::new());
    
    for item in content.split(";") {
        let item = item.trim();

        if !item.is_empty() {
            if item.starts_with(".") {
        
                classes.push_str(" ");
                classes.push_str(&item[1..item.len()]);
            
            } else {
    
                classes.push_str(" ");
                styles.push_str(item);
                styles.push_str(";");
            }
        }

    }

    styles = styles.trim().to_string();
    classes = classes.trim().to_string();

    let mut result: (Option<Styles>, Option<Classes>) = (None, None);

    if !styles.is_empty() {
        result.0 = Some(styles);
    }

    if !classes.is_empty() {
        result.1 = Some(classes);
    }

    result
}

pub fn split_styles_and_classes(content: &str) -> (Option<Styles>, Option<Classes>) {
    self::split_styles_and_classes_with_default(content, (None, None))
}


#[cfg(test)]
mod test {

    #[test]
    fn test_split_styles_and_classes() {
        let content = "\nstyle1:value1;.class1;style2:value2;.class2\n";

        let (styles, classes) = super::split_styles_and_classes(content);

        assert_eq!(styles.unwrap(), "style1:value1; style2:value2;");
        assert_eq!(classes.unwrap(), "class1 class2");

        let content = "\nstyle1:value1;style2:value2;\n";

        let (styles, classes) = super::split_styles_and_classes(content);

        assert_eq!(styles.unwrap(), "style1:value1; style2:value2;");
        assert_eq!(classes, None);
    }
}