use std::{collections::HashMap, fmt::Debug};
use regex::Regex;
use crate::{codex::modifier::standard_text_modifier::StandardTextModifier, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, compiler::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, output_format::OutputFormat};
use super::CompilationRule;
use crate::compiler::compilation_error::CompilationError;


pub struct HtmlGreekLettersRule {
    search_pattern: String,
    search_pattern_regex: Regex,
    greek_letters_map: HashMap<&'static str, &'static str>,
}

impl HtmlGreekLettersRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardTextModifier::GreekLetter.modifier_pattern(),
            search_pattern_regex: Regex::new(&StandardTextModifier::GreekLetter.modifier_pattern()).unwrap(),
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

impl Debug for HtmlGreekLettersRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlGreekLettersRule").field("searching_pattern", &self.search_pattern).finish()
    }
}

impl CompilationRule for HtmlGreekLettersRule {
    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }

    fn standard_compile(&self, compilable: &CompilableText, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {

        let mut compiled_parts = Vec::new();

        for matc in self.search_pattern_regex.captures_iter(&compilable.compilable_content()) {

            if let Some(greek_ref) = matc.get(1) {
                
                let reference_part = CompilableTextPart::new(
                    format!(r#"<span class="greek">${}$</span>"#, self.replace_with_greek_letters(greek_ref.as_str())),
                    CompilableTextPartType::Fixed
                );

                compiled_parts.push(reference_part);
            }

            log::error!("no greek letters found in '{}' ({})", compilable.compilable_content(), matc.get(0).unwrap().as_str());
            
            // TODO: strict option with panic
        }

        Ok(CompilableText::new(compiled_parts))
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}
