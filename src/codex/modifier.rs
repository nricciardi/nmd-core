pub mod standard_paragraph_modifier;
pub mod standard_text_modifier;
pub mod standard_heading_modifier;
pub mod base_modifier;
pub mod constants;


use regex::Regex;
use crate::resource::bucket::Bucket;
use self::base_modifier::BaseModifier;
use super::CodexIdentifier;


pub type ModifiersBucket = Bucket<CodexIdentifier>;
pub type ModifierIdentifier = String;
pub type ModifierPattern = String;


/// `Modifier` is the component to identify a NMD modifier, which will be replaced using particular rule indicated by `Codex` 
pub trait Modifier: std::fmt::Debug + Sync + Send {

    fn modifier_pattern(&self) -> &ModifierPattern;
    
    fn modifier_pattern_regex(&self) -> &Regex; 

    fn incompatible_modifiers(&self) -> &ModifiersBucket {
        &ModifiersBucket::None
    }
}

impl PartialEq for dyn Modifier {
    fn eq(&self, other: &Self) -> bool {
        self.modifier_pattern().eq(other.modifier_pattern())
    }
}

impl Clone for Box<dyn Modifier> {
    fn clone(&self) -> Self {
        Box::new(BaseModifier::new(self.modifier_pattern().clone(), self.incompatible_modifiers().clone()))
    }
}
