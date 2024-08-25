use getset::{Getters, Setters};
use thiserror::Error;
use crate::utility::nmd_unique_identifier::NmdUniqueIdentifier;
use super::compilation_result::{CompilationResultPart, CompilationResultPartType, CompilationResultParts};


#[derive(Error, Debug)]
pub enum CompilableError {
    #[error("compilable content {0} has an overflow using {1} -> {2}")]
    ContentOverflow(String, usize, usize),
}


#[derive(Debug, Getters, Setters)]
pub struct Compilable {

    #[getset(get = "pub")]
    parts: CompilationResultParts,

    /// content usable in regex. It's the string obtained concatenating compilable parts 
    #[getset(get = "pub")]
    compilable_content: String,

    #[getset(get = "pub")]
    nuid: Option<NmdUniqueIdentifier>
}

impl Compilable {
    pub fn new(parts: CompilationResultParts, nuid: Option<NmdUniqueIdentifier>) -> Self {

        let mut compilable_content = String::new();

        parts.compilable_content().iter().for_each(|part| {
            match part.part_type() {
                CompilationResultPartType::Fixed => (),
                CompilationResultPartType::Compilable { incompatible_modifiers: _ } => compilable_content.push_str(&part.content()),
            }
        });

        Self {
            parts,
            compilable_content,
            nuid
        }
    }

    pub fn parts_slice(&self, start: usize, end: usize) -> Result<CompilationResultParts, CompilableError> {

        if end > self.compilable_content.len() {
            return Err(CompilableError::ContentOverflow(self.compilable_content.clone(), start, end))
        }

        let mut parts_slice = CompilationResultParts::new();

        let mut start_part_position_in_compilable_content: usize = 0; 
        let mut end_part_position_in_compilable_content: usize;

        let mut slice_found = false;

        for part in self.parts {

            match part.part_type() {
                CompilationResultPartType::Fixed => {
                    if slice_found {
                        parts_slice.push(part.clone());

                        continue;
                    }
                },
                CompilationResultPartType::Compilable { incompatible_modifiers } => {

                    end_part_position_in_compilable_content = start_part_position_in_compilable_content + part.content().len();


                    if start_part_position_in_compilable_content <= start {
                        
                        let part = part.clone();
                        
                        part.set_content(part.content()[(start - start_part_position_in_compilable_content)..end_part_position_in_compilable_content.min(end)]);

                        parts_slice.push(part);

                        if end < end_part_position_in_compilable_content {         // start and end are in the same part
                            break;              
                        }

                        slice_found = true;
                    }

                    if end < end_part_position_in_compilable_content {
                        
                        let part = part.clone();
                        
                        part.set_content(part.content()[start_part_position_in_compilable_content..end]);

                        parts_slice.push(part);
                        
                        break;              
                    }
                    
                    start_part_position_in_compilable_content = end_part_position_in_compilable_content;
                },
            }

        }

        Ok(parts_slice)
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
    use crate::{codex::modifier::ModifiersBucket, compiler::compilation_result::{CompilationResultPart, CompilationResultPartType}};

    use super::Compilable;


    #[test]
    fn parts_between_positions() {
        let compilable = Compilable::new(vec![
            CompilationResultPart::new(
                String::from("this is a string with 35 characters"),
                CompilationResultPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
            ),
            CompilationResultPart::new(
                String::from("this is the fixed part"),
                CompilationResultPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
            ),
            CompilationResultPart::new(
                String::from("end of the content"),
                CompilationResultPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
            ),
        ], None);

        let start1: usize = 5;
        let start2: usize = 25;

        let end1: usize = 16;
        let end2: usize = 37;

        let parts_slice = compilable.parts_slice(start1, end1).unwrap();

        assert_eq!(parts_slice.len(), 1);
        assert_eq!(parts_slice[0].content(), String::from("is a string"));

        let parts_slice = compilable.parts_slice(start2, end2).unwrap();

        assert_eq!(parts_slice.len(), 3);
        assert_eq!(parts_slice[0].content(), String::from("characters"));
        assert_eq!(parts_slice[1].content(), String::from("this is the fixed part"));
        assert_eq!(parts_slice[2].content(), String::from("end"));
    }

}







