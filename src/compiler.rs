pub mod compilable;
pub mod compilation_rule;
pub mod compilation_error;
pub mod compilation_result;
pub mod compilation_configuration;
pub mod compilation_metadata;

use std::sync::{Arc, RwLock};


use compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration};
use compilation_error::CompilationError;
use compilation_result::{CompilationResult, CompilationResultPart};
use getset::{Getters, Setters};

use super::{codex::{modifier::modifiers_bucket::ModifiersBucket, Codex}, dossier::document::Paragraph};



#[derive(Debug)]
enum Segment {
    Match(String),
    NonMatch(String),
}


#[derive(Debug, Getters, Setters)]
pub struct Compiler {
}

impl Compiler {

    /// Compile a string
    pub fn compile_str(codex: &Codex, content: &str, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        let excluded_modifiers = compilation_configuration.read().unwrap().excluded_modifiers().clone();

        Self::compile_str_excluding_modifiers(codex, content, excluded_modifiers, Arc::clone(&compilation_configuration), compilation_configuration_overlay)
    }

    /// Compile a string excluding a set of modifiers
    pub fn compile_str_excluding_modifiers(codex: &Codex, content: &str, excluded_modifiers: ModifiersBucket, 
        compilation_configuration: Arc<RwLock<CompilationConfiguration>>, _compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        log::debug!("start to compile content:\n{}\nexcluding: {:?}", content, excluded_modifiers);

        if excluded_modifiers == ModifiersBucket::All {
            log::debug!("compilation of content:\n{} is skipped because are excluded all modifiers", content);
            
            return Ok(CompilationResult::new_fixed(content.to_string()))
        }

        let mut content_parts: Vec<CompilationResultPart> = vec![CompilationResultPart::Mutable{ content: String::from(content) }];
        let mut content_parts_additional_excluded_modifiers: Vec<ModifiersBucket> = vec![ModifiersBucket::None];

        for text_modifier in codex.configuration().ordered_text_modifiers() {

            assert_eq!(content_parts.len(), content_parts_additional_excluded_modifiers.len());

            if excluded_modifiers.contains(text_modifier) {

                log::debug!("{:?} is skipped", text_modifier);
                continue;
            }

            let text_rule = codex.text_rules().get(text_modifier.identifier());

            if text_rule.is_none() {
                log::warn!("text rule for {:#?} not found", text_modifier);
                continue;
            }

            let text_rule = text_rule.unwrap();

            let mut new_content_parts: Vec<CompilationResultPart> = Vec::new();
            let mut new_content_parts_additional_excluded_modifiers: Vec<ModifiersBucket> = Vec::new();
            
            for (content_part_index, content_part) in content_parts.iter().enumerate() {

                let current_excluded_modifiers = &content_parts_additional_excluded_modifiers[content_part_index];

                let mut no_match_fn = || {
                        
                    new_content_parts.push(content_part.clone());
                    new_content_parts_additional_excluded_modifiers.push(current_excluded_modifiers.clone());
                };

                if let CompilationResultPart::Fixed { content } = content_part {

                    log::debug!("{:?} is skipped for because '{}' fixed", text_modifier, content);

                    no_match_fn();
                    continue;
                }

                if current_excluded_modifiers.contains(text_modifier) {
                    log::debug!("{:?} is skipped for '{}'", text_modifier, content_part.content());

                    no_match_fn();
                    continue;
                }

                let matches = text_rule.find_iter(&content_part.content());

                if matches.len() == 0 {
                    log::debug!("'{}' => no matches with {:?}", content_part.content(), text_rule);
                    
                    no_match_fn();
                    continue;
                }

                log::debug!("'{}' => there is a match with {:#?}", content_part.content(), text_rule);

                let mut last_end = 0;

                let mut elaborate_segment_fn = |segment: Segment| -> Result<(), CompilationError> {
                    match segment {
                        Segment::Match(m) => {
                            
                            let outcome = text_rule.compile(&m, codex, Arc::clone(&compilation_configuration))?;

                            for part in Into::<Vec<CompilationResultPart>>::into(outcome) {

                                new_content_parts.push(part);
            
                                let new_current_excluded_modifiers = current_excluded_modifiers.clone() + text_modifier.incompatible_modifiers().clone();
            
                                new_content_parts_additional_excluded_modifiers.push(new_current_excluded_modifiers);
                            
                            }
                        },
                        Segment::NonMatch(s) => {
                            new_content_parts.push(CompilationResultPart::Mutable { content: s.clone() });
                            new_content_parts_additional_excluded_modifiers.push(current_excluded_modifiers.clone());

                        },
                    }

                    Ok(())
                };

                for mat in matches {

                    if mat.start() > last_end {
                        let segment = Segment::NonMatch(content_part.content()[last_end..mat.start()].to_string());

                        elaborate_segment_fn(segment)?;
                    }

                    let segment = Segment::Match(mat.as_str().to_string());

                    elaborate_segment_fn(segment)?;

                    last_end = mat.end();
                }

                if last_end < content_part.content().len() {
                    let segment = Segment::NonMatch(content_part.content()[last_end..].to_string());

                    elaborate_segment_fn(segment)?;
                }
            }

            content_parts = new_content_parts;
            content_parts_additional_excluded_modifiers = new_content_parts_additional_excluded_modifiers;
            
        }
        
        Ok(CompilationResult::new(content_parts))
    }

    /// Compile a `Paragraph`.
    /// 
    /// Only one paragraph rule can be applied on Paragraph.
    pub fn compile_paragraph(codex: &Codex, paragraph: &Paragraph, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {
        Self::compile_paragraph_excluding_modifiers(codex, paragraph, ModifiersBucket::None, compilation_configuration, compilation_configuration_overlay)
    }

    /// Compile a `Paragraph` excluding a set of modifiers.
    /// 
    /// Only one paragraph rule can be applied on Paragraph.
    pub fn compile_paragraph_excluding_modifiers(codex: &Codex, paragraph: &Paragraph, mut excluded_modifiers: ModifiersBucket, compilation_configuration: Arc<RwLock<CompilationConfiguration>>,
        compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        log::debug!("start to compile paragraph ({:?}):\n{}\nexcluding: {:?}", paragraph.paragraph_type(), paragraph, excluded_modifiers);

        let mut outcome: CompilationResult = CompilationResult::new_fixed(paragraph.content().to_string());

        if excluded_modifiers == ModifiersBucket::All {
            log::debug!("compilation of paragraph:\n{:#?} is skipped are excluded all modifiers", paragraph);
            
            return Ok(outcome)
        }

        let paragraph_modifier = codex.configuration().paragraph_modifier(paragraph.paragraph_type()).unwrap();

        let paragraph_rule = codex.paragraph_rules().get(paragraph_modifier.identifier());

        if let Some(paragraph_rule) = paragraph_rule {

            log::debug!("paragraph rule {:?} is found, it is about to be applied to compile paragraph", paragraph_rule);

            outcome = paragraph_rule.compile(&paragraph.content(), codex, Arc::clone(&compilation_configuration))?;

            excluded_modifiers = excluded_modifiers + paragraph_modifier.incompatible_modifiers().clone();

        } else {

            log::warn!("there is NOT a paragraph rule for '{}' in codex", paragraph.paragraph_type());
        }

        outcome.apply_compile_function_to_mutable_parts(|mutable_part| Self::compile_str_excluding_modifiers(codex, &mutable_part.content(), excluded_modifiers.clone(), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay)))?;

        Ok(outcome)
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use crate::{codex::{codex_configuration::CodexConfiguration, modifier::modifiers_bucket::ModifiersBucket, Codex}, compiler::compilation_configuration::CompilationConfiguration};

    use super::Compiler;


    #[test]
    fn parse_text() {

        let codex = Codex::of_html(CodexConfiguration::default());

        let content = "Text **bold text** `a **bold text** which must be not parsed`";
        let compilation_configuration = CompilationConfiguration::default();
        let excluded_modifiers = ModifiersBucket::None;

        let outcome = Compiler::compile_str_excluding_modifiers(&codex, content, excluded_modifiers, Arc::new(RwLock::new(compilation_configuration)), Arc::new(None)).unwrap();

        assert_eq!(outcome.content(), r#"Text <strong class="bold">bold text</strong> <code class="language-markup inline-code">a **bold text** which must be not parsed</code>"#)
    }
}