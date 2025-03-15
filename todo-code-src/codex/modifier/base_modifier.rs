use regex::Regex;

use super::{ModifiersBucket, Modifier, ModifierPattern};


#[derive(Debug)]
pub struct BaseModifier {
    modifier_pattern: ModifierPattern,
    incompatible_modifiers: ModifiersBucket,
    modifier_pattern_regex: Regex,
}

impl BaseModifier {
    pub fn new(modifier_pattern: ModifierPattern, regex: Regex, incompatible_modifiers: ModifiersBucket) -> Self {
        Self {
            modifier_pattern_regex: regex,
            modifier_pattern,
            incompatible_modifiers
        }
    }
}

impl Modifier for BaseModifier {

    fn modifier_pattern(&self) -> &ModifierPattern {
        &self.modifier_pattern
    }

    fn incompatible_modifiers(&self) -> &ModifiersBucket {
        &self.incompatible_modifiers
    }
    
    fn modifier_pattern_regex(&self) -> &Regex {
        &self.modifier_pattern_regex
    }
}