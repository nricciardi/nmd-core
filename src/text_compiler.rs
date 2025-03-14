pub mod transformation_rule;
pub mod text;
pub mod text_compilation_error;
pub mod text_compilation_configuration;

use text::{text_part::TextPart, Text};
use text_compilation_configuration::TextCompilationConfiguration;
use text_compilation_error::TextCompilationError;

use crate::{compilation::{compilation_configuration::CompilationConfiguration, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, utility::datastruct::bucket::Bucket};

pub struct TextCompiler {
}


impl TextCompiler {

    /// Compile parts and return the new compiled parts or `None` if there are not matches using
    /// provided rule
    /*fn compile_with_compilation_rule() -> Result<(), CompilationError> {
    
        let parts = self.parts();

        let mut compilable_content = String::new();
        let mut compilable_content_end_parts_positions: Vec<usize> = Vec::new();

        parts.iter()
                .filter(|part| {
                    match &part.part_type() {
                        TextPart::Fixed(_) => false,
                        TextPart::Compilable(content, incompatible_modifiers) => {
                            if incompatible_modifiers.contains(&rule_identifier) {
                                return false
                            } else {
                                return true
                            }
                        },
                    }
                })
                .for_each(|part| {

                    compilable_content.push_str(part.content());

                    let last_pos = *compilable_content_end_parts_positions.last().unwrap_or(&0);

                    compilable_content_end_parts_positions.push(last_pos + part.content().len());
                });

        let matches = rule.find_iter(&compilable_content);

        if matches.len() == 0 {
            log::debug!("'{}' => no matches with {:?} -> {:?}", compilable_content, rule_identifier, rule.search_pattern());
            
            return Ok(());
        }

        log::debug!("'{}' => there is a match with {:?} -> {:?}", compilable_content, rule_identifier, rule.search_pattern());

        let mut compiled_parts: Vec<TextPart> = Vec::new();     // final output

        let mut parts_index: usize = 0;
        let mut compilable_parts_index: usize = 0;

        // only for compilable parts
        let mut part_start_position_in_compilable_content: usize = 0;
        let mut part_end_position_in_compilable_content: usize;

        let mut match_index: usize = 0;

        while parts_index < parts.len() {      // there are other parts

            let match_start_end: Option<(usize, usize)>;        // start and end

            if match_index < matches.len() {

                let current_evaluated_match = matches[match_index];

                match_index += 1;    
            
                match_start_end = Some((
                    current_evaluated_match.start(),
                    current_evaluated_match.end()
                ));

            } else {

                match_start_end = None;
            }

            let mut match_found = false;

            let mut matched_parts: Vec<TextPart> = Vec::new();
            
            'parts_loop: while parts_index < parts.len() {

                let part = &parts[parts_index];

                parts_index += 1;   // for next iteration

                match part.part_type() {
                    TextPart::Fixed(_) => {

                        if let Some((_start, _end)) = match_start_end {

                            if match_found {        // matching end cannot be in a fixed part

                                matched_parts.push(part.clone());
        
                                continue 'parts_loop;
                            
                            } else {
                                
                                compiled_parts.push(part.clone());      // direct in compiled_parts
    
                                continue 'parts_loop;
                            }
                        
                        } else {
                            compiled_parts.push(part.clone());      // direct in compiled_parts

                            continue 'parts_loop;
                        }
                    },
                    TextPart::Compilable(_, incompatible_modifiers) => {

                        if incompatible_modifiers.contains(rule_identifier) {
                            compiled_parts.push(part.clone());      // direct in compiled_parts

                            continue 'parts_loop;
                        }

                        part_end_position_in_compilable_content = compilable_content_end_parts_positions[compilable_parts_index];
                        
                        compilable_parts_index += 1;

                        if let Some((match_start, match_end)) = match_start_end {

                            if !match_found && part_end_position_in_compilable_content <= match_start {      // there is no match in this part
                            
                                let sub_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];

                                compiled_parts.push(TextPart::new(
                                    TextPart::Compilable {
                                        content: sub_part.to_string(),
                                        incompatible_modifiers: incompatible_modifiers.clone()
                                    }
                                ));
    
                            } else {
                                // ...part has a match
    
                                if !match_found     // first part in which current match is found
                                    && part_start_position_in_compilable_content <= match_start
                                    && match_start < part_end_position_in_compilable_content {

                                    // === pre-matched part ==
                                    let pre_matched_part = &compilable_content[part_start_position_in_compilable_content..match_start];
                                                                            
                                    if !pre_matched_part.is_empty() {
                                        compiled_parts.push(TextPart::new(
                                            TextPart::Compilable {
                                                content: pre_matched_part.to_string(),
                                                incompatible_modifiers: incompatible_modifiers.clone()
                                            }
                                        ));
                                    }

                                    part_start_position_in_compilable_content = match_start;

                                    // === matched part ===
                                    let matched_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content.min(match_end)];

                                    matched_parts.push(TextPart::new(
                                        TextPart::Compilable {
                                            content: matched_part.to_string(),
                                            incompatible_modifiers: incompatible_modifiers.clone()
                                        }
                                    ));
                                }
                                
                                if match_end <= part_end_position_in_compilable_content {       // matching end is in this part

                                    if match_found {   // the matching end is in another part respect of matching start

                                        let matched_part = &compilable_content[part_start_position_in_compilable_content..match_end];

                                        matched_parts.push(TextPart::new(
                                            TextPart::Compilable {
                                                content: matched_part.to_string(),
                                                incompatible_modifiers: incompatible_modifiers.clone()
                                            }
                                        ));
                                    }

                                    // compile and append found matched parts
                                    compiled_parts.append(
                                        &mut rule.compile(
                                            &Text::from(matched_parts),
                                            format,
                                            compilation_configuration,
                                            compilation_configuration_overlay.clone()
                                        )?.parts_mut() 
                                    );

                                    // re-start next parts loop from this part
                                    parts_index -= 1;       
                                    compilable_parts_index -= 1;

                                    part_start_position_in_compilable_content = match_end;

                                    break 'parts_loop;

                                } else {

                                    if match_found {        // this part is a compilable part in the middle of matched parts

                                        let matched_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];

                                        matched_parts.push(TextPart::new(
                                            TextPart::Compilable {
                                                content: matched_part.to_string(),
                                                incompatible_modifiers: incompatible_modifiers.clone()
                                            }
                                        ));
                                    }
                                }

                                match_found = true;     // update to check if match is found in next iterations
                            }

                        } else {
                            
                            let part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];
                                                                            
                            if !part.is_empty() {
                                compiled_parts.push(TextPart::new(
                                    TextPart::Compilable {
                                        content: part.to_string(),
                                        incompatible_modifiers: incompatible_modifiers.clone()
                                    }
                                ));
                            }
                        }
        
                        // update start position
                        part_start_position_in_compilable_content = part_end_position_in_compilable_content;
                    }

                }
            }
        }

        self.set_parts(compiled_parts);
        
        Ok(())
    }*/

    pub fn compile(text: Text, configuration: &dyn TextCompilationConfiguration) -> Result<CompilationOutcome, TextCompilationError> {

        let excluded_rules = configuration.excluded_rules().clone();        // TODO: remove .clone()? 

        log::debug!("start to compile content:\n{:?}\nexcluding: {:?}", text, excluded_rules);

        if excluded_rules == Bucket::All {
            log::debug!("compilation of content:\n{:?} is skipped because are excluded all transformation rules", text);
            
            return Ok(CompilationOutcome::from(text))
        }

        for rule in configuration.transformation_rules() {

            if excluded_rules.contains(rule.identifier()) {

                log::debug!("{:?} is skipped", rule.identifier());
                continue;
            }

            rule.apply(&mut text, configuration)?;
        }

        Ok(CompilationOutcome::from(text.into()))
    }

    pub fn compile_str(str: &str, configuration: &dyn TextCompilationConfiguration) -> Result<CompilationOutcome, TextCompilationError> {

        Self::compile(Text::from(str), configuration)      
    }
}
