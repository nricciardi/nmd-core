// use getset::{Getters, Setters};
// use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
// use regex::{Captures, Regex, Replacer};

// use crate::{codex::CodexIdentifier, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::{CompilationResult, CompilationResultPart, CompilationResultPartType, CompilationResultParts}}, output_format::OutputFormat};

// use super::CompilationRule;




// #[derive(Debug, Getters, Setters, Clone)]
// pub struct NewReplacementRule/* <R: Replacer> */ {

//     #[getset(set)]
//     search_pattern: String,

//     #[getset(set)]
//     search_pattern_regex: Regex,

//     #[getset(get = "pub", set = "pub")]
//     replacer_parts: Vec</* ReplacementRuleReplacerPart<R> */>,

//     #[getset(get = "pub", set = "pub")]
//     newline_fix_pattern: Option<String>,

//     #[getset(get = "pub", set = "pub")]
//     nuid_placeholder: String,
// }


// impl NewReplacementRule {
    
//     fn search_pattern(&self) -> &String {
//         &self.search_pattern
//     }
    
//     fn search_pattern_regex(&self) -> &Regex {
//         &self.search_pattern_regex
//     }

//     fn standard_compile(&self, parts: &CompilationResultParts, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<Option<CompilationResultParts>, CompilationError> {
        
//         let rule_identifier: CodexIdentifier;       // TODO

//         let mut compilable_content = String::new();

//         parts.iter()
//                 .filter(|part| {
//                     match &part.part_type() {
//                         CompilationResultPartType::Fixed => false,
//                         CompilationResultPartType::Compilable => {
//                             if part.incompatible_modifiers().contains(&rule_identifier) {
//                                 return false
//                             } else {
//                                 return true
//                             }
//                         },
//                     }
//                 })
//                 .for_each(|part| compilable_content.push_str(part.content()));

//         let matches: Vec<Captures> = self.search_pattern_regex.captures_iter(&compilable_content).collect();

//         if matches.len() == 0 {
//             log::debug!("'{}' => no matches with {:?}", compilable_content, self);
            
//             return Ok(None);
//         }

//         log::debug!("'{}' => there is a match with {:#?}", compilable_content, self);

//         let mut compiled_parts: CompilationResultParts = CompilationResultParts::new();     // final output

//         let mut parts_index: usize = 0;

//         // only for compilable parts
//         let mut part_start_position_in_compilable_content: usize = 0;
//         let mut part_end_position_in_compilable_content: usize = 0;
//         let mut part_start_offset: usize = 0;

//         'match_loop: for matc in matches {

//             if matc.get(0).is_none() {
//                 panic!("full match not found using {:?} on {:?}", self, parts);
//             }

//             let full_match = matc.get(0).unwrap();

//             let match_start = full_match.start();
//             let match_end = full_match.end();

//             let mut match_found = false;

//             let mut matched_parts: CompilationResultParts = CompilationResultParts::new();

//             'parts_loop: loop {

//                 let part = &parts[parts_index];

//                 parts_index += 1;   // for next iteration  

//                 match part.part_type() {
//                     CompilationResultPartType::Fixed => {

//                         if match_found {        // matching end cannot be in a fixed part

//                             matched_parts.push(part.clone());
    
                            
//                             continue 'parts_loop;
                        
//                         } else {
                            
//                             compiled_parts.push(part.clone());

//                             continue 'parts_loop;
//                         }

//                     },
//                     CompilationResultPartType::Compilable => {

//                         part_end_position_in_compilable_content = part_start_position_in_compilable_content + part.content().len();

//                         if !match_found && part_end_position_in_compilable_content < match_start {      // there is no match in this part
                            
//                             compiled_parts.push(part.clone());

//                         } else {
//                             // ...part has a match

//                             if !match_found     // first part in which current match is found
//                                 && part_start_position_in_compilable_content <= match_start
//                                 && match_start < part_end_position_in_compilable_content {

//                                 // === pre-matched part ==
//                                 let pre_matched_part = &compilable_content[part_start_position_in_compilable_content..match_start];
                                                                    
//                                 if !pre_matched_part.is_empty() {
//                                     compiled_parts.push(CompilationResultPart::new(
//                                         pre_matched_part.to_string(),
//                                         CompilationResultPartType::Compilable,
//                                         part.incompatible_modifiers().clone()
//                                     ));
//                                 }

//                                 // === matched part ===
//                                 let matched_part = &compilable_content[match_start..part_end_position_in_compilable_content.min(match_end)];

//                                 matched_parts.push(CompilationResultPart::new(
//                                     matched_part.to_string(),
//                                     CompilationResultPartType::Compilable,
//                                     part.incompatible_modifiers().clone()
//                                 ));
//                             }

//                             // TODO: check if matching start -> add to match_parts, else add to compiled_parts

//                             // TODO: check if matching end -> split part (first in match_parts, second in next match iterations -> continue match loop), else whole part in match_parts

//                             if match_end <= part_end_position_in_compilable_content {       // matching end is in this part

//                                 if match_found {   // the matching end is in another part respect of matching start

//                                     let matched_part = &compilable_content[part_start_position_in_compilable_content..match_end];

//                                     matched_parts.push(CompilationResultPart::new(
//                                         matched_part.to_string(),
//                                         CompilationResultPartType::Compilable,
//                                         part.incompatible_modifiers().clone()
//                                     ));
//                                 }


//                                 // TODO: compile matched parts
//                                 println!("matched_parts: {:?}", matched_parts);


//                                 part_end_position_in_compilable_content = part_start_offset;
//                                 part_start_offset = match_end;

//                             } else {

//                                 if match_found {        // simple matched part in matched parts 

//                                     matched_parts.push(part.clone());
//                                 }
//                             }

//                             // update start position
//                             part_start_position_in_compilable_content = part_end_position_in_compilable_content + part_start_offset;
//                             part_start_offset = 0;

//                             match_found = true;     // update to check if match is found in next iterations
//                         }

//                     },
//                 }

//             }
//         }
        
//         Ok(Some(compiled_parts))
//     }
// }