pub mod transformation_configuration;
pub mod transformation_error;
// pub mod replacement_rule;
pub mod greek_letter_rule;
// pub mod reference_rule;
// pub mod html_cite_rule;
pub mod constants;


use std::fmt::Debug;
use regex::{Captures, Match, Regex};
use transformation_configuration::TextTransformationConfiguration;
use transformation_error::TransformationError;
use crate::mmo::bucket::Bucket;

use super::{text::text_part::TextPart, Text};


pub type TextTransformationRuleIdentifier = String;


pub trait TextTransformationRule: Send + Sync + Debug {

    fn identifier(&self) -> &TextTransformationRuleIdentifier;

    fn incompatible_rules(&self) -> &Bucket<TextTransformationRuleIdentifier>;

    fn apply(&self, text: &mut Text, configuration: &dyn TextTransformationConfiguration) -> Result<(), TransformationError>;
}

pub trait TextRegexTransformationRule: TextTransformationRule {

    fn search_regex(&self) -> &Regex;

    // TODO: iterable instead vec
    fn captures_transform(&self, captures: Captures, configuration: &dyn TextTransformationConfiguration) -> Result<Vec<TextPart>, TransformationError>;

    fn apply(&self, text: &mut Text, configuration: &dyn TextTransformationConfiguration) -> Result<(), TransformationError> {

        // TODO: parallelize
        // for captures in self.search_regex().captures_iter(&text.compilable_content()) {

        //     let parts = self.captures_transform(captures, configuration)?;

        //     // TODO: replace `parts` in `text`
        // }

        Ok(())
    }
} 