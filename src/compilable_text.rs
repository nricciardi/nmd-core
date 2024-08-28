pub mod compilable_text_part;


use compilable_text_part::{CompilableTextPart, CompilableTextPartType};
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;
use thiserror::Error;

use crate::utility::nmd_unique_identifier::NmdUniqueIdentifier;


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

        let mut compilable_content = String::new();

        self.parts.iter().for_each(|part| {
            match part.part_type() {
                CompilableTextPartType::Fixed => (),
                CompilableTextPartType::Compilable { incompatible_modifiers: _ } => compilable_content.push_str(&part.content()),
            }
        });

        compilable_content
    }

    /// string generated using all parts contents 
    pub fn content(&self) -> String {

        let mut content = String::new();

        self.parts.iter().for_each(|part| content.push_str(part.content()));

        content
    }

    /// parts between two positions in `compilable_content`.
    /// If start or end are in the middle of a compilable part, it will be split.
    /// 
    /// **`start` is included, but `end` is excluded** as typically behavior of `end()` methods.
    pub fn parts_slice(&self, start: usize, end: usize) -> Result<Vec<CompilableTextPart>, CompilableError> {

        let compilable_content = self.compilable_content();

        if end > compilable_content.len() {
            return Err(CompilableError::ContentOverflow(compilable_content, start, end))
        }

        let mut parts_slice = Vec::new();

        let mut start_part_position_in_compilable_content: usize = 0; 
        let mut end_part_position_in_compilable_content: usize;

        let mut slice_found = false;

        for part in &self.parts {

            let mut slice_found_in_current_iteration = false;

            match part.part_type() {
                CompilableTextPartType::Fixed => {
                    if slice_found {
                        parts_slice.push(part.clone());

                        continue;
                    }
                },
                CompilableTextPartType::Compilable { incompatible_modifiers: _ } => {

                    end_part_position_in_compilable_content = start_part_position_in_compilable_content + part.content().len();

                    if start_part_position_in_compilable_content == end {
                        break;
                    } 

                    if start_part_position_in_compilable_content <= start
                        && start < end_part_position_in_compilable_content {     // start matching
                        
                        let part = CompilableTextPart::new(
                            compilable_content[start..end_part_position_in_compilable_content.min(end)].to_string(),
                            part.part_type().clone()
                        );
                        
                        parts_slice.push(part);

                        if end < end_part_position_in_compilable_content {         // start and end are in the same part
                            break;              
                        }

                        slice_found = true;
                        slice_found_in_current_iteration = true;
                    }

                    if slice_found {

                        if end < end_part_position_in_compilable_content {

                            let part = CompilableTextPart::new(
                                compilable_content[start_part_position_in_compilable_content..end].to_string(),
                                part.part_type().clone()
                            );
                            
                            parts_slice.push(part);
    
                            break;

                        } else {

                            if !slice_found_in_current_iteration {

                                parts_slice.push(part.clone());
                            }
                        }
                    }

                    
                    
                    start_part_position_in_compilable_content = end_part_position_in_compilable_content;
                },
            }

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

// impl FromStr for GenericCompilable {
//     type Err = Infallible;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Ok(Self {
//             content: s.to_string(),
//             nuid: None
//         })
//     }
// }

// impl From<String> for GenericCompilable {
//     fn from(value: String) -> Self {
//         Self {
//             content: value,
//             nuid: None
//         }
//     }
// }


#[cfg(test)]
mod test {
    use crate::{codex::modifier::ModifiersBucket, compilable_text::compilable_text_part::{CompilableTextPart, CompilableTextPartType}};

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

}







