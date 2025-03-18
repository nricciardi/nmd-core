pub mod transformation_configuration;
pub mod transformation_error;
pub mod text_regex_transformation_rule;
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