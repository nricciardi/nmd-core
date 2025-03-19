use crate::text_compiler::text::{text_part::TextPart, Text};

use super::{transformation_configuration::TextTransformationConfiguration, transformation_error::TransformationError, TextTransformationRule};
use regex::{Captures, Match, Regex};

pub trait TextRegexTransformationRule: TextTransformationRule {

    fn search_regex(&self) -> &Regex;

    // TODO: iterable instead vec
    fn captures_transform(&self, captures: Captures, configuration: &dyn TextTransformationConfiguration) -> Result<Vec<TextPart>, TransformationError>;

    fn apply(&self, text: &mut Text, configuration: &dyn TextTransformationConfiguration) -> Result<(), TransformationError> {

        let compilable = text.parts_compatible_with_rule(self.identifier());

        let compilable_string = compilable.to_string();

        log::debug!("apply {} to {}", self.identifier(), compilable_string);

        let captures = self.search_regex().captures_iter(&compilable_string);


        // TODO: parallelize
        // for captures in self.search_regex().captures_iter(&text.compilable_content()) {

        //     let parts = self.captures_transform(captures, configuration)?;

        //     // TODO: replace `parts` in `text`
        // }

        Ok(())
    }
} 