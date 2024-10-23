pub mod compilable_text_part;


use compilable_text_part::{CompilableTextPart, CompilableTextPartType};
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use thiserror::Error;
use crate::{codex::{modifier::ModifiersBucket, Codex, CodexIdentifier}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_rule::CompilationRule, self_compile::SelfCompile}, output_format::OutputFormat, resource::bucket::Bucket, utility::nmd_unique_identifier::NmdUniqueIdentifier};


#[derive(Debug, Clone)]
pub enum PartsSliceElaborationPolicy {
    DontTakeBorderFixedParts,
    TakeLeftFixedParts,
    TakeRightFixedParts,
    TakeLeftAndRightFixedParts,
}

#[derive(Debug, Clone)]
enum ElaborationPosition {
    BeforeRange,
    InRange,
    AfterRange,
}


#[derive(Error, Debug)]
pub enum CompilableError {
    #[error("compilable content {0} has an overflow using {1} -> {2}")]
    ContentOverflow(String, usize, usize),
}


#[derive(Debug, Clone, Getters, MutGetters, Setters, Serialize)]
pub struct CompilableText {

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    parts: Vec<CompilableTextPart>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,
}



impl From<CompilableTextPart> for CompilableText {
    fn from(value: CompilableTextPart) -> Self {
        Self::new(vec![value])
    }
}

impl From<Vec<CompilableTextPart>> for CompilableText {
    fn from(value: Vec<CompilableTextPart>) -> Self {
        Self::new(value)
    }
}

impl Into<Vec<CompilableTextPart>> for CompilableText {
    fn into(self) -> Vec<CompilableTextPart> {
        self.parts  
    }
}

impl Into<String> for CompilableText {
    fn into(self) -> String {
        self.content()
    }
}

impl From<String> for CompilableText {
    fn from(value: String) -> Self {
        Self::from(CompilableTextPart::new_compilable(
            value,
            ModifiersBucket::None
        ))
    }
}

impl From<&str> for CompilableText {
    fn from(value: &str) -> Self {
        Self::from(CompilableTextPart::new_compilable(
            value.to_string(),
            ModifiersBucket::None
        ))
    }
}

impl CompilableText {

    pub fn new_empty() -> Self {
        Self {
            parts: Vec::new(),
            nuid: None,
        }
    }

    pub fn new(parts: Vec<CompilableTextPart>) -> Self {

        Self {
            parts,
            nuid: None,
        }
    }

    pub fn new_with_nuid(parts: Vec<CompilableTextPart>, nuid: Option<NmdUniqueIdentifier>) -> Self {
        Self {
            parts,
            nuid
        }
    }

    /// content usable in regex. It's the string obtained concatenating compilable parts
    pub fn compilable_content(&self) -> String {

        self.compilable_content_with_ends_positions().0
    }

    pub fn compilable_content_with_ends_positions(&self) -> (String, Vec<usize>) {
        let mut compilable_content = String::new();
        let mut ends: Vec<usize> = Vec::new();
        let mut last_end: usize = 0;

        self.parts.iter().for_each(|part| {
            match part.part_type() {
                CompilableTextPartType::Fixed => (),
                CompilableTextPartType::Compilable { incompatible_modifiers: _ } => {

                    let content = part.content();

                    ends.push(last_end + content.len());
                    last_end = *ends.last().unwrap();

                    compilable_content.push_str(&content);
                },
            }
        });

        (compilable_content, ends)
    }

    /// string generated using all parts contents 
    pub fn content(&self) -> String {

        let mut content = String::new();

        self.parts.iter().for_each(|part| content.push_str(part.content()));

        content
    }

    /// this method calls `parts_slice_with_explicit_policy` using policy `TakeLeftAndRightFixedParts`
    pub fn parts_slice(&self, start: usize, end: usize) -> Result<Vec<CompilableTextPart>, CompilableError> {
        self.parts_slice_with_explicit_policy(start, end, PartsSliceElaborationPolicy::TakeLeftAndRightFixedParts)
    }

    /// parts between two positions in `compilable_content`.
    /// If start or end are in the middle of a compilable part, it will be split.
    /// 
    /// **`start` is included, but `end` is excluded** as typically behavior of `end()` methods.
    pub fn parts_slice_with_explicit_policy(&self, start: usize, end: usize, elaboration_policy: PartsSliceElaborationPolicy) -> Result<Vec<CompilableTextPart>, CompilableError> {

        let (compilable_content, ends) = self.compilable_content_with_ends_positions();

        if end > compilable_content.len() {
            return Err(CompilableError::ContentOverflow(compilable_content, start, end))
        }

        let mut parts_slice: Vec<CompilableTextPart> = Vec::new();

        let mut start_part_position_in_compilable_content: usize = 0; 
        let mut end_part_position_in_compilable_content: usize;

        let mut elaboration_position = ElaborationPosition::BeforeRange;

        let mut left_fixed_parts: Vec<&CompilableTextPart> = Vec::new();
        let mut right_fixed_parts: Vec<&CompilableTextPart> = Vec::new();

        let mut index: usize = 0;
        let mut compilable_parts_index: usize = 0;

        while index < self.parts.len() {

            let part = &self.parts[index];

            index += 1;

            match part.part_type() {
                CompilableTextPartType::Fixed => {

                    match elaboration_position {
                        ElaborationPosition::BeforeRange => left_fixed_parts.push(part),
                        ElaborationPosition::InRange => parts_slice.push(part.clone()),
                        ElaborationPosition::AfterRange => right_fixed_parts.push(part),
                    }
                },
                CompilableTextPartType::Compilable { incompatible_modifiers: _ } => {

                    end_part_position_in_compilable_content = ends[compilable_parts_index];

                    compilable_parts_index += 1;

                    if start_part_position_in_compilable_content == end {

                        elaboration_position = ElaborationPosition::AfterRange;
                    }

                    match elaboration_position {

                        ElaborationPosition::BeforeRange => {
                            if start_part_position_in_compilable_content <= start
                                && start < end_part_position_in_compilable_content {     // start matching
        
                                if start_part_position_in_compilable_content < start {      // there is a pre-match compilable part segment
                                    left_fixed_parts.clear();
                                } 
                                
                                let part = CompilableTextPart::new(
                                    compilable_content[start..end_part_position_in_compilable_content.min(end)].to_string(),
                                    part.part_type().clone()
                                );
                                
                                parts_slice.push(part);
        
                                if end < end_part_position_in_compilable_content {         // start and end are in the same part
                                    break;              
                                }
        
                                elaboration_position = ElaborationPosition::InRange;

                                start_part_position_in_compilable_content = end_part_position_in_compilable_content.min(end);

                                if start_part_position_in_compilable_content < end_part_position_in_compilable_content {

                                    index -= 1;
                                    compilable_parts_index -= 1;
                                    
                                    continue;
                                }
                            
                            } else {        // no matching in this part
        
                                left_fixed_parts.clear();
                            }
                        },

                        ElaborationPosition::InRange => {
                            if end <= end_part_position_in_compilable_content {      // the end is in this part

                                let content = compilable_content[start_part_position_in_compilable_content..end].to_string();

                                if !content.is_empty() {
                                    // take last part segment
                                    let part = CompilableTextPart::new(
                                        content,
                                        part.part_type().clone()
                                    );
                                    
                                    parts_slice.push(part);
                                }

                                if end < end_part_position_in_compilable_content {
                                    break;
                                }

                                elaboration_position = ElaborationPosition::AfterRange;

                            } else {
                                let part = CompilableTextPart::new(
                                    compilable_content[start_part_position_in_compilable_content..end_part_position_in_compilable_content].to_string(),
                                    part.part_type().clone()
                                );
                                
                                parts_slice.push(part);
                            }
                        },
                        
                        ElaborationPosition::AfterRange => break,
                    }

                    if start_part_position_in_compilable_content == end {

                        elaboration_position = ElaborationPosition::AfterRange;
                    }
                    
                    start_part_position_in_compilable_content = end_part_position_in_compilable_content;
                },
            }

        }

        match elaboration_policy {
            PartsSliceElaborationPolicy::DontTakeBorderFixedParts => (),
            PartsSliceElaborationPolicy::TakeLeftFixedParts => left_fixed_parts.into_iter().for_each(|p| parts_slice.insert(0, p.clone())),
            PartsSliceElaborationPolicy::TakeRightFixedParts => right_fixed_parts.into_iter().for_each(|p| parts_slice.push(p.clone())),
            PartsSliceElaborationPolicy::TakeLeftAndRightFixedParts => {

                left_fixed_parts.into_iter().for_each(|p| parts_slice.insert(0, p.clone()));

                right_fixed_parts.into_iter().for_each(|p| parts_slice.push(p.clone()));
            },
        }

        Ok(parts_slice)
    }
}

impl CompilableText {

    /// Compile parts and return the new compiled parts or `None` if there are not matches using
    /// provided rule
    pub fn compile_with_compilation_rule(&mut self, (rule_identifier, rule): (&CodexIdentifier, &Box<dyn CompilationRule>), format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
    
        let parts = self.parts();

        let mut compilable_content = String::new();
        let mut compilable_content_end_parts_positions: Vec<usize> = Vec::new();

        parts.iter()
                .filter(|part| {
                    match &part.part_type() {
                        CompilableTextPartType::Fixed => false,
                        CompilableTextPartType::Compilable{ incompatible_modifiers } => {
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

        let mut compiled_parts: Vec<CompilableTextPart> = Vec::new();     // final output

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

            let mut matched_parts: Vec<CompilableTextPart> = Vec::new();
            
            'parts_loop: while parts_index < parts.len() {

                let part = &parts[parts_index];

                parts_index += 1;   // for next iteration

                match part.part_type() {
                    CompilableTextPartType::Fixed => {

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
                    CompilableTextPartType::Compilable{ incompatible_modifiers } => {

                        if incompatible_modifiers.contains(rule_identifier) {
                            compiled_parts.push(part.clone());      // direct in compiled_parts

                            continue 'parts_loop;
                        }

                        part_end_position_in_compilable_content = compilable_content_end_parts_positions[compilable_parts_index];
                        
                        compilable_parts_index += 1;

                        if let Some((match_start, match_end)) = match_start_end {

                            if !match_found && part_end_position_in_compilable_content <= match_start {      // there is no match in this part
                            
                                let sub_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];

                                compiled_parts.push(CompilableTextPart::new(
                                    sub_part.to_string(),
                                    CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                ));
    
                            } else {
                                // ...part has a match
    
                                if !match_found     // first part in which current match is found
                                    && part_start_position_in_compilable_content <= match_start
                                    && match_start < part_end_position_in_compilable_content {

                                    // === pre-matched part ==
                                    let pre_matched_part = &compilable_content[part_start_position_in_compilable_content..match_start];
                                                                            
                                    if !pre_matched_part.is_empty() {
                                        compiled_parts.push(CompilableTextPart::new(
                                            pre_matched_part.to_string(),
                                            CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                        ));
                                    }

                                    part_start_position_in_compilable_content = match_start;

                                    // === matched part ===
                                    let matched_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content.min(match_end)];

                                    matched_parts.push(CompilableTextPart::new(
                                        matched_part.to_string(),
                                        CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                    ));
                                }
                                
                                if match_end <= part_end_position_in_compilable_content {       // matching end is in this part

                                    if match_found {   // the matching end is in another part respect of matching start

                                        let matched_part = &compilable_content[part_start_position_in_compilable_content..match_end];

                                        matched_parts.push(CompilableTextPart::new(
                                            matched_part.to_string(),
                                            CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                        ));
                                    }

                                    // compile and append found matched parts
                                    compiled_parts.append(
                                        &mut rule.compile(
                                            &CompilableText::from(matched_parts),
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

                                        matched_parts.push(CompilableTextPart::new(
                                            matched_part.to_string(),
                                            CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                        ));
                                    }
                                }

                                match_found = true;     // update to check if match is found in next iterations
                            }

                        } else {
                            
                            let part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];
                                                                            
                            if !part.is_empty() {
                                compiled_parts.push(CompilableTextPart::new(
                                    part.to_string(),
                                    CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
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
    }
}

impl SelfCompile for CompilableText {

    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        let excluded_modifiers = compilation_configuration_overlay.excluded_modifiers().clone();

        log::debug!("start to compile content:\n{:?}\nexcluding: {:?}", self, excluded_modifiers);

        if excluded_modifiers == Bucket::All {
            log::debug!("compilation of content:\n{:?} is skipped because are excluded all modifiers", self);
            
            return Ok(())
        }

        for (codex_identifier, text_modifier) in codex.text_modifiers() {

            if excluded_modifiers.contains(codex_identifier) {

                log::debug!("{:?} is skipped", text_modifier);
                continue;
            }

            if let Some(text_rule) = codex.text_compilation_rules().get(codex_identifier) {

                self.compile_with_compilation_rule((codex_identifier, text_rule), format, compilation_configuration, compilation_configuration_overlay.clone())?;

            } else {

                log::warn!("text rule for {:#?} not found", text_modifier);
                continue;
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{codex::{modifier::{standard_text_modifier::StandardTextModifier, ModifiersBucket}, Codex}, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, PartsSliceElaborationPolicy}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, self_compile::SelfCompile}, output_format::OutputFormat};

    use super::CompilableText;


    #[test]
    fn parts_between_positions_in_cfc() {
        let compilable = CompilableText::new(vec![
            CompilableTextPart::new(
                String::from("this is a string with 35 characters"),
                CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
            ),
            CompilableTextPart::new(
                String::from("this is the fixed part"),
                CompilableTextPartType::Fixed
            ),
            CompilableTextPart::new(
                String::from("end of the content"),
                CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
            ),
        ]);

        let start1: usize = 5;
        let start2: usize = 25;

        let end1: usize = 16;
        let end2: usize = 38;

        let parts_slice = compilable.parts_slice(start1, end1).unwrap();

        assert_eq!(parts_slice.len(), 1);
        assert_eq!(parts_slice[0].content(), &String::from("is a string"));

        let parts_slice = compilable.parts_slice(start2, end2).unwrap();

        assert_eq!(parts_slice.len(), 3);
        assert_eq!(parts_slice[0].content(), &String::from("characters"));
        assert_eq!(parts_slice[1].content(), &String::from("this is the fixed part"));
        assert_eq!(parts_slice[2].content(), &String::from("end"));
    }

    #[test]
    fn parts_between_positions_in_cfcfc() {
        let compilable = CompilableText::new(vec![
            CompilableTextPart::new_compilable(String::from("c1"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f1")),
            CompilableTextPart::new_compilable(String::from("c2"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f2")),
            CompilableTextPart::new_compilable(String::from("c3"), ModifiersBucket::None),
        ]);

        let start: usize = 1;
        let end: usize = 5;

        let parts_slice = compilable.parts_slice(start, end).unwrap();

        assert_eq!(parts_slice.len(), 5);
        assert_eq!(parts_slice[0].content(), &String::from("1"));
        assert_eq!(parts_slice[1].content(), &String::from("f1"));
        assert_eq!(parts_slice[2].content(), &String::from("c2"));
        assert_eq!(parts_slice[3].content(), &String::from("f2"));
        assert_eq!(parts_slice[4].content(), &String::from("c"));

        let compilable = CompilableText::new(vec![
            CompilableTextPart::new_compilable(String::from("c1"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f1")),
            CompilableTextPart::new_compilable(String::from("c2"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f2")),
            CompilableTextPart::new_compilable(String::from("c3"), ModifiersBucket::None),
        ]);

        let start: usize = 1;
        let end: usize = 4;

        let parts_slice = compilable.parts_slice(start, end).unwrap();

        assert_eq!(parts_slice.len(), 4);
        assert_eq!(parts_slice[0].content(), &String::from("1"));
        assert_eq!(parts_slice[1].content(), &String::from("f1"));
        assert_eq!(parts_slice[2].content(), &String::from("c2"));
        assert_eq!(parts_slice[3].content(), &String::from("f2"));
    }

    #[test]
    fn parts_between_positions_in_cfcfc_with_explicit_policy() {
        let compilable = CompilableText::new(vec![
            CompilableTextPart::new_fixed(String::from("f-1")),
            CompilableTextPart::new_compilable(String::from("c0"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f0")),
            CompilableTextPart::new_compilable(String::from("*"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f1")),
            CompilableTextPart::new_compilable(String::from("c2"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f2")),
            CompilableTextPart::new_compilable(String::from("*"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f3")),
            CompilableTextPart::new_compilable(String::from("c3"), ModifiersBucket::None),
            CompilableTextPart::new_fixed(String::from("f4")),
        ]);

        let start: usize = 3;
        let end: usize = 5;

        // ==== take left and right ====
        let parts_slice = compilable.parts_slice_with_explicit_policy(start, end, PartsSliceElaborationPolicy::TakeLeftAndRightFixedParts).unwrap();

        assert_eq!(parts_slice.len(), 3);
        assert_eq!(parts_slice[0].content(), &String::from("f1"));
        assert_eq!(parts_slice[1].content(), &String::from("c2"));
        assert_eq!(parts_slice[2].content(), &String::from("f2"));

        // ==== take left ====
        let parts_slice = compilable.parts_slice_with_explicit_policy(start, end, PartsSliceElaborationPolicy::TakeLeftFixedParts).unwrap();

        assert_eq!(parts_slice.len(), 2);
        assert_eq!(parts_slice[0].content(), &String::from("f1"));
        assert_eq!(parts_slice[1].content(), &String::from("c2"));

        // ==== take right ====
        let parts_slice = compilable.parts_slice_with_explicit_policy(start, end, PartsSliceElaborationPolicy::TakeRightFixedParts).unwrap();

        assert_eq!(parts_slice.len(), 2);
        assert_eq!(parts_slice[0].content(), &String::from("c2"));
        assert_eq!(parts_slice[1].content(), &String::from("f2"));

        // ==== no take ====
        let parts_slice = compilable.parts_slice_with_explicit_policy(start, end, PartsSliceElaborationPolicy::DontTakeBorderFixedParts).unwrap();

        assert_eq!(parts_slice.len(), 1);
        assert_eq!(parts_slice[0].content(), &String::from("c2"));
    }

    #[test]
    fn compile_nested_modifiers() {

        let mut codex = Codex::of_html();

        codex.retain(HashSet::from([
            StandardTextModifier::BoldStarVersion.identifier(),
            StandardTextModifier::BoldUnderscoreVersion.identifier(),
            StandardTextModifier::ItalicStarVersion.identifier(),
            StandardTextModifier::ItalicUnderscoreVersion.identifier(),
            StandardTextModifier::InlineCode.identifier(),
        ]));

        let compilation_configuration = CompilationConfiguration::default();

        let content = "A piece of **bold text**, *italic text*, `a **(fake) bold text** which must be not parsed` and *nested **bold text***";

        let mut outcome = CompilableText::from(content);
        
        outcome.compile(&OutputFormat::Html, &codex, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();       

        assert_eq!(outcome.content(), concat!(
            "A piece of ",
            r#"<strong class="bold">bold text</strong>, "#,
            r#"<em class="italic">italic text</em>, "#,
            r#"<code class="language-markup inline-code">a **(fake) bold text** which must be not parsed</code>"#,
            r#" and "#,
            r#"<em class="italic">nested <strong class="bold">bold text</strong></em>"#,
        ));
    }

}







