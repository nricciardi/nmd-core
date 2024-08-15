use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, compiler::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, list_bullet_configuration_record::{self, ListBulletConfigurationRecord}, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult}, output_format::OutputFormat, utility::text_utility};
use super::{constants::{ESCAPE_HTML, SPACE_TAB_EQUIVALENCE}, CompilationRule};
use std::fmt::Debug;


static SEARCH_LIST_ITEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&StandardParagraphModifier::ListItem.modifier_pattern()).unwrap());

pub const LIST_ITEM_INDENTATION: &str = r#"<span class="list-item-indentation"></span>"#;



#[derive(Debug)]
pub struct HtmlListRule {
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl HtmlListRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardParagraphModifier::List.modifier_pattern_with_paragraph_separator(),
            search_pattern_regex: Regex::new(&StandardParagraphModifier::List.modifier_pattern_with_paragraph_separator()).unwrap(),
        }
    }

    fn transform_to_field(to: String) -> String {

        if to.eq(list_bullet_configuration_record::CHECKBOX) {
            return String::from(r#"<div class="checkbox checkbox-unchecked"></div>"#)
        }

        if to.eq(list_bullet_configuration_record::CHECKBOX_CHECKED) {
            return String::from(r#"<div class="checkbox checkbox-checked"></div>"#)
        }

        to
    }

    fn bullet_transform(bullet: &str, indentation_level: usize, list_bullets_configurations: &Vec<ListBulletConfigurationRecord>) -> String {

        for bullet_configuration in list_bullets_configurations {

            if bullet_configuration.from.eq(bullet) {
                if bullet_configuration.strict_indentation && indentation_level == bullet_configuration.indentation_level {

                    return Self::transform_to_field(bullet_configuration.to.clone())

                } else if !bullet_configuration.strict_indentation && indentation_level >= bullet_configuration.indentation_level {
                    return Self::transform_to_field(bullet_configuration.to.clone())
                }
            }
        }

        String::from(bullet)
    }
}

impl CompilationRule for HtmlListRule {
    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }

    fn standard_compile(&self, compilable: &Box<dyn Compilable>, _format: &OutputFormat, _codex: &Codex, compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {
        
        let mut compilation_result = CompilationResult::new_empty();

        let content = compilable.compilable_content();

        let nuid_attr: String;

        if let Some(nuid) = compilable.nuid() {
            nuid_attr = format!(r#"data-nuid="{}""#, nuid);
        } else {
            nuid_attr = String::new();
        }

        compilation_result.add_fixed_part(format!(r#"<ul class="list" {}>"#, nuid_attr));

        let mut items_found = 0;

        let mut parsed_lines: Vec<(&str, &str)> = Vec::new();

        for captures in SEARCH_LIST_ITEM_REGEX.captures_iter(content) {
            if let Some(indentation) = captures.get(1) {
                if let Some(bullet) = captures.get(2) {
                    if let Some(content) = captures.get(3) {

                        items_found += 1;

                        let mut indentation = String::from(indentation.as_str());
                        let bullet = bullet.as_str();
                        let content = content.as_str();

                        parsed_lines.push((bullet, content));
                        
                        indentation = indentation.replace("\t", SPACE_TAB_EQUIVALENCE);

                        let mut indentation_level: usize = 0;
                        while indentation.starts_with(SPACE_TAB_EQUIVALENCE) {
                            indentation = indentation.split_off(SPACE_TAB_EQUIVALENCE.len());
                            indentation_level += 1;
                        }

                        let bullet = Self::bullet_transform(bullet, indentation_level, compilation_configuration.list_bullets_configuration());

                        let content = text_utility::replace(&content, &ESCAPE_HTML);

                        compilation_result.add_fixed_part(r#"<li class="list-item">"#.to_string());
                        compilation_result.add_fixed_part(LIST_ITEM_INDENTATION.repeat(indentation_level));
                        compilation_result.add_fixed_part(r#"<span class="list-item-bullet">"#.to_string());
                        compilation_result.add_fixed_part(bullet);
                        compilation_result.add_fixed_part(r#"</span><span class="list-item-content">"#.to_string());
                        compilation_result.add_mutable_part(content);
                        compilation_result.add_fixed_part(r#"</span></li>"#.to_string());

                    }
                }
            }
        }

        let total_valid_lines = content.lines().into_iter().filter(|l| !l.is_empty() && !l.to_string().eq("\n")).count();

        if items_found != total_valid_lines {

            if compilation_configuration.strict_list_check() {
                log::error!("the following list has incorrect items (parsed {} on {}):\n{}\n-----\nparsed:\n{:#?}", items_found, total_valid_lines, content, parsed_lines);
                panic!("incorrect list item(s)")
            } else {
                log::warn!("the following list has incorrect items (parsed {} on {}):\n{}\n-----\nparsed:\n{:#?}", items_found, total_valid_lines, content, parsed_lines);
            }
        }

        compilation_result.add_fixed_part("</ul>".to_string());
        
        Ok(compilation_result)
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::compilable::GenericCompilable;

    use super::*;

    #[test]
    fn parsing() {

        todo!()

//         let nmd_text = r#"
// - element 1
// - element 2
//     - element 2.1
//         - element 2.1.1a
//         | element 2.1.1b
//         - element 2.1.2
//         - element 2.1.3
//     - element 2.2
// - element 3
// "#.trim();
       
//        let rule = HtmlListRule::new();

//        let _ = Regex::new(rule.search_pattern()).unwrap();

//        let codex = Codex::of_html(CodexConfiguration::default());

//        let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(nmd_text.to_string()));

//        let _ = rule.compile(&compilable, &OutputFormat::Html, &codex, &CompilationConfiguration::default(), Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

    }
}