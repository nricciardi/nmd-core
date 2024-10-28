use getset::{Getters, Setters};
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{codex::{modifier::{standard_paragraph_modifier::StandardParagraphModifier, ModifiersBucket}, Codex}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, list_bullet_configuration_record::{self, ListBulletConfigurationRecord}, CompilationConfiguration}, compilation_error::CompilationError, compilation_rule::constants::{ESCAPE_HTML, SPACE_TAB_EQUIVALENCE}, compilable::Compilable}, dossier::document::chapter::paragraph::Paragraph, output_format::OutputFormat, utility::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};


static SEARCH_LIST_ITEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&StandardParagraphModifier::ListItem.modifier_pattern()).unwrap());

pub const LIST_ITEM_INDENTATION: &str = r#"<span class="list-item-indentation"></span>"#;


#[derive(Debug, Getters, Setters)]
pub struct ListParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

}

impl ListParagraph {

    pub fn new(raw_content: String) -> Self {
        Self {
            raw_content,
            nuid: None,
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

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        let mut compilation_result = CompilableText::new_empty();

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<ul class="list"{}>"#, text_utility::html_nuid_tag_or_nothing(self.nuid.as_ref()))));

        let mut items_found = 0;

        let mut parsed_lines: Vec<(&str, &str)> = Vec::new();

        for captures in SEARCH_LIST_ITEM_REGEX.captures_iter(&self.raw_content) {
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

                        compilation_result.parts_mut().push(
                            CompilableTextPart::new_fixed(
                                format!(
                                    r#"<li class="list-item">{}<span class="list-item-bullet">{}</span><span class="list-item-content">"#,
                                    LIST_ITEM_INDENTATION.repeat(indentation_level),
                                    bullet
                                )
                            )
                        );
                        
                        compilation_result.parts_mut().push(CompilableTextPart::new_compilable(content, ModifiersBucket::None));
                        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(r#"</span></li>"#.to_string()));

                    }
                }
            }
        }

        let total_valid_lines = self.raw_content.lines().into_iter().filter(|l| !l.is_empty() && !l.to_string().eq("\n")).count();

        if items_found != total_valid_lines {

            if compilation_configuration.strict_list_check() {
                log::error!("the following list has incorrect items (parsed {} on {}):\n{}\n-----\nparsed:\n{:#?}", items_found, total_valid_lines, self.raw_content, parsed_lines);
                panic!("incorrect list item(s)")
            } else {
                log::warn!("the following list has incorrect items (parsed {} on {}):\n{}\n-----\nparsed:\n{:#?}", items_found, total_valid_lines, self.raw_content, parsed_lines);
            }
        }

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(r#"</ul>"#.to_string()));

        Compiler::compile_compilable_text(&mut compilation_result, &OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        self.compiled_content = Some(compilation_result);
        
        Ok(())
    }
}

impl Compilable for ListParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }
    }
}
impl Paragraph for ListParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}


#[cfg(test)]
mod test {

    use crate::load::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, paragraph_loading_rule::{list_paragraph_loading_rule::ListParagraphLoadingRule, ParagraphLoadingRule}};

    use super::*;

    #[test]
    fn compile() {

        let nmd_text = r#"
- element 1
- element 2
    - element 2.1
        - element 2.1.1a
        | element 2.1.1b
        - element 2.1.2
        - element 2.1.3
    - element 2.2
- element 3
"#.trim();
       
        let codex = Codex::of_html();
        
        let rule = ListParagraphLoadingRule::new();

        let mut paragraph = rule.load(nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();
        
        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap();

        let compiled_content = paragraph.compiled_text().as_ref().unwrap().content();
        let li_n = Regex::new("<li").unwrap().find_iter(&compiled_content).count();

        assert_eq!(li_n, 9);

        // TODO
        // assert_eq!(
        //     list_paragraph.compilation_result().unwrap().content(),
        //     concat!(
        //         r#"<ul class="list">"#,
        //         r#"<li class="list-item">"#,
        //         r#"<span class="list-item-bullet">"#
        //         "</ul>"
        //     )
        // )

    }
}