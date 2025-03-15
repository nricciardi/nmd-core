use std::{collections::HashMap, fmt::Debug};
use regex::Regex;

use crate::{mmo::bucket::Bucket, text_compiler::text::{text_part::TextPart, Text}};

use super::{transformation_configuration::TextTransformationConfiguration, transformation_error::TransformationError, TextRegexTransformationRule, TextTransformationRule, TextTransformationRuleIdentifier};


pub struct GreekLettersRule {
    identifier: TextTransformationRuleIdentifier,
    search_regex: Regex,
    greek_letters_map: HashMap<&'static str, &'static str>,
}

impl GreekLettersRule {
    pub fn new() -> Self {
        Self {
            identifier: String::from("greek-letter"),
            search_regex: Regex::new(r"%(\w*?)%").unwrap(),
            greek_letters_map: HashMap::from([
                ("a", r"alpha"),
                ("b", r"beta"),
                ("g", r"gamma"),
                ("d", r"delta"),
                ("e", r"epsilon"),
                ("z", r"zeta"),
                ("n", r"eta"),
                ("th", r"theta"),
                ("i", r"iota"),
                ("k", r"kappa"),
                ("l", r"lambda"),
                ("m", r"mu"),
                ("nu", r"nu"),
                ("x", r"xi"),
                ("o", r"omicron"),
                ("p", r"pi"),
                ("r", r"rho"),
                ("s", r"sigma"),
                ("t", r"tau"),
                ("u", r"upsilon"),
                ("phi", r"phi"),
                ("chi", r"chi"),
                ("psi", r"psi"),
                ("w", r"omega"),

                ("A", r"Alpha"),
                ("B", r"Beta"),
                ("G", r"Gamma"),
                ("D", r"Delta"),
                ("E", r"Epsilon"),
                ("Z", r"Zeta"),
                ("N", r"Eta"),
                ("Th", r"Theta"),
                ("I", r"Iota"),
                ("K", r"Kappa"),
                ("L", r"Lambda"),
                ("M", r"Mu"),
                ("Nu", r"Nu"),
                ("X", r"Xi"),
                ("O", r"Omicron"),
                ("P", r"Pi"),
                ("R", r"Rho"),
                ("S", r"Sigma"),
                ("T", r"Tau"),
                ("U", r"Upsilon"),
                ("Phi", r"Phi"),
                ("Chi", r"Chi"),
                ("Psi", r"Psi"),
                ("W", r"Omega"),
            ])
        }
    }

    fn replace_with_greek_letters(&self, input: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
    
        while i < input.len() {
            let mut matched = false;
            
            let mut keys: Vec<&str> = self.greek_letters_map.keys().cloned().collect();

            keys.sort_by(|a, b| b.len().cmp(&a.len()));

            for key in keys {
                if input[i..].starts_with(key) {
                    result.push_str(r"\");
                    result.push_str(self.greek_letters_map.get(key).unwrap());
                    i += key.len();
                    matched = true;
                    break;
                }
            }
    
            if !matched {
                result.push(input.chars().nth(i).unwrap());
                i += 1;
            }
        }
    
        result
    }
}

impl Debug for GreekLettersRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GreekLettersRule").field("searching_pattern", &self.search_regex).finish()
    }
}

impl TextTransformationRule for GreekLettersRule {

    fn identifier(&self) -> &TextTransformationRuleIdentifier {
        &self.identifier
    }

    fn incompatible_rules(&self) -> &Bucket<TextTransformationRuleIdentifier> {
        &Bucket::All
    }
    
    fn apply(&self, text: &mut Text, configuration: &dyn TextTransformationConfiguration) -> Result<(), TransformationError> {
        TextRegexTransformationRule::apply(self, text, configuration)
    }
}

impl TextRegexTransformationRule for GreekLettersRule {
    fn search_regex(&self) -> &Regex {
        &self.search_regex
    }

    fn captures_transform(&self, captures: regex::Captures, configuration: &dyn TextTransformationConfiguration) -> Result<Vec<TextPart>, TransformationError> {
        
        if let Some(greek_ref) = captures.get(1) {
                
            return Ok(vec![
                TextPart::Fixed {
                    content: format!(r#"<span class="greek">${}$</span>"#, self.replace_with_greek_letters(greek_ref.as_str()))
                }
            ]);
        
        } else {

            log::error!("no greek letters found in '{}'", captures.get(0).unwrap().as_str());
            
            // TODO
            // if configuration.others().get("strict_greek_letters_check") {
            //     return Err(CompilationError::ElaborationErrorVerbose(format!("no greek letters found in '{}' ({})", text.compilable_content(), matc.get(0).unwrap().as_str())))
            // }

            todo!()
        }
    }
}

#[cfg(test)]
mod test {

    use super::GreekLettersRule;

    // TODO

    // #[test]
    // fn standard_compile() {
    //     let rule = HtmlGreekLettersRule::new();

    //     let compilable = CompilableString::from(vec![
    //         CompilableStringPart::new_fixed(String::from("fixed1")),
    //         CompilableStringPart::new_compilable(String::from("%aphib%"), ModifiersBucket::None),
    //         CompilableStringPart::new_fixed(String::from("fixed2")),
    //     ]);

    //     let output = rule.compile(&compilable, &OutputFormat::Html, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap();
    
    //     assert_eq!(output.parts().len(), 1);
    // }

}
