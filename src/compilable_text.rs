pub mod compilable_text_part;


use compilable_text_part::{CompilableTextPart, CompilableTextPartType};
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use thiserror::Error;
use crate::utility::nmd_unique_identifier::NmdUniqueIdentifier;


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


#[cfg(test)]
mod test {
    use crate::{codex::modifier::ModifiersBucket, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, PartsSliceElaborationPolicy}};

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
}







