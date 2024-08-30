use regex::Regex;

use crate::codex::modifier::constants::{CHAPTER_STYLE_PATTERN, CHAPTER_TAGS_PATTERN};

use super::base_modifier::BaseModifier;
use super::constants::MAX_HEADING_LEVEL;
use super::ModifiersBucket;
use super::ModifierIdentifier;

#[derive(Debug, PartialEq, Clone)]
pub enum StandardHeading {

    HeadingGeneralCompactVersion(u32),
    HeadingGeneralExtendedVersion(u32),
    MinorHeading,
    MajorHeading,
    SameHeading,

}

impl StandardHeading {
    pub fn ordered() -> Vec<Self> {
        let mut heading_modifiers: Vec<Self> = vec![Self::MinorHeading, Self::MajorHeading, Self::SameHeading];

        for i in (1..=MAX_HEADING_LEVEL).rev() {
            heading_modifiers.push(Self::HeadingGeneralExtendedVersion(i));
            heading_modifiers.push(Self::HeadingGeneralCompactVersion(i));
        }

        heading_modifiers
    }

    pub fn heading_level(content: &str) -> Option<u32> {
        let heading_modifiers = Self::ordered();

        for heading_modifier in heading_modifiers {
            let regex = Regex::new(&heading_modifier.modifier_pattern()).unwrap();

            if regex.is_match(content) {
                match heading_modifier {
                    Self::HeadingGeneralExtendedVersion(level) => return Option::Some(level),
                    Self::HeadingGeneralCompactVersion(level) => return Option::Some(level),
                    _ => panic!("unexpected modifier: {:?}", heading_modifier)
                }
            }
        }

        Option::None
    }

    pub fn str_is_heading(content: &str) -> bool {
        Self::heading_level(content).is_some()
    }

    pub fn identifier(&self) -> ModifierIdentifier {
        match *self {
            Self::HeadingGeneralExtendedVersion(level) => {

                if level == 0 || level > MAX_HEADING_LEVEL {
                    panic!("{level} is an invalid heading level.")
                }

                format!(r"heading-{}-extended-version", level)
            },
            Self::HeadingGeneralCompactVersion(level) => {

                if level == 0 || level > MAX_HEADING_LEVEL {
                    panic!("{level} is an invalid heading level.")
                }

                format!(r"heading-{}-compact-version", level)
            },
            StandardHeading::MinorHeading => String::from("minor-heading"),
            StandardHeading::MajorHeading => String::from("major-heading"),
            StandardHeading::SameHeading => String::from("same-heading"),
        }
    }
    
    pub fn modifier_pattern(&self) -> String {
        let specific_pattern = match *self {
            Self::HeadingGeneralExtendedVersion(level) => {

                if level == 0 || level > MAX_HEADING_LEVEL {
                    panic!("{level} is an invalid heading level.")
                }

                format!(r"(?m:^#{{{}}}\s+(.*))", level)
            },
            Self::HeadingGeneralCompactVersion(level) => {

                if level == 0 || level > MAX_HEADING_LEVEL {
                    panic!("{level} is an invalid heading level.")
                }

                format!(r"(?m:^#({})\s+(.*))", level)
            },
            StandardHeading::MinorHeading => String::from(r"(?m:^#-\s+(.*))"),
            StandardHeading::MajorHeading => String::from(r"(?m:^#\+\s+(.*))"),
            StandardHeading::SameHeading => String::from(r"(?m:^#=\s+(.*))"),
        };

        format!("{}{}{}", specific_pattern, CHAPTER_TAGS_PATTERN, CHAPTER_STYLE_PATTERN)
    }

    pub fn incompatible_modifiers(&self) -> ModifiersBucket {
        ModifiersBucket::None
    }
}

impl Into<BaseModifier> for StandardHeading {
    fn into(self) -> BaseModifier {
        BaseModifier::new(self.modifier_pattern(), Regex::new(&self.modifier_pattern()).unwrap(), self.incompatible_modifiers())
    }
}