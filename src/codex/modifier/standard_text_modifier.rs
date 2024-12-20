use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use super::{base_modifier::BaseModifier, constants::{ABRIDGED_STYLE_PATTERN, IDENTIFIER_PATTERN, NEW_LINE_PATTERN, STYLE_PATTERN}, ModifierIdentifier, ModifiersBucket};


static MODIFIER_PATTERNS_REGEX: Lazy<HashMap<ModifierIdentifier, Regex>> = Lazy::new(|| {
    let mut regex: HashMap<ModifierIdentifier, Regex> = HashMap::new();

    StandardTextModifier::ordered().into_iter().for_each(|m| {
        regex.insert(m.identifier(), Regex::new(&m.modifier_pattern()).unwrap());
    });

    regex
});


#[derive(Debug, PartialEq, Clone)]
pub enum StandardTextModifier {
    
    BoldStarVersion,
    BoldUnderscoreVersion,
    ItalicStarVersion,
    ItalicUnderscoreVersion,
    Strikethrough,
    Underlined,
    Link,
    AbridgedEmbeddedStyle,
    EmbeddedStyle,
    Identifier,
    Highlight,
    Emoji,
    Superscript,
    Subscript,
    InlineCode,
    InlineMath,
    Comment,
    AbridgedBookmark,
    Bookmark,
    Todo,
    Checkbox,
    CheckboxChecked,
    GreekLetter,
    Escape,
    Reference,
    Cite,
}

impl StandardTextModifier {

    pub fn ordered() -> Vec<Self> {

        //! they must have the compatibility order
        vec![
            Self::InlineCode,
            Self::InlineMath,
            Self::Comment,
            Self::GreekLetter,
            Self::Todo,
            Self::Bookmark,
            Self::AbridgedBookmark,
            Self::EmbeddedStyle,
            Self::AbridgedEmbeddedStyle,
            Self::Identifier,
            Self::Highlight,
            Self::BoldStarVersion,
            Self::BoldUnderscoreVersion,
            Self::ItalicStarVersion,
            Self::ItalicUnderscoreVersion,
            Self::Strikethrough,
            Self::Underlined,
            Self::Superscript,
            Self::Subscript,
            Self::Link,
            Self::Checkbox,
            Self::CheckboxChecked,
            Self::Emoji,
            Self::Escape,
            Self::Reference,
            Self::Cite,
        ]
    }

    pub fn identifier(&self) -> ModifierIdentifier {
        match self {
            Self::AbridgedBookmark => String::from("abridged-bookmark"),
            Self::Bookmark => String::from("bookmark"),
            Self::Todo => String::from("todo"),
            Self::AbridgedEmbeddedStyle => String::from("abridged-embedded-style"),
            Self::Identifier => String::from("identifier"),
            Self::EmbeddedStyle => String::from("embedded-style"),
            Self::Highlight => String::from("highlight"),
            Self::Comment => String::from("comment"),
            Self::Emoji => String::from("emoji"),
            Self::Checkbox => String::from("checkbox"),
            Self::CheckboxChecked => String::from("checkbox-checked"),
            Self::Superscript => String::from("superscript"),
            Self::Subscript => String::from("subscript"),
            Self::BoldStarVersion => String::from("bold-star-version"),
            Self::BoldUnderscoreVersion => String::from("bold-underscore-version"),
            Self::ItalicStarVersion => String::from("italic-star-version"),
            Self::ItalicUnderscoreVersion => String::from("italic-underscore-version"),
            Self::Strikethrough => String::from("strikethrough"),
            Self::Underlined => String::from("underlined"),
            Self::Link => String::from("link"),
            Self::InlineCode => String::from("inline-code"),
            Self::InlineMath => String::from("inline-math"),
            Self::GreekLetter => String::from("greek-letter"),
            Self::Escape => String::from("escape"),
            Self::Reference => String::from("reference"),
            Self::Cite => String::from("cite"),
        }
    }
    
    pub fn modifier_pattern(&self) -> String {
        match *self {
            Self::AbridgedBookmark => format!(r"@\[([^\]]*?)\](?:{})?", IDENTIFIER_PATTERN),
            Self::Bookmark => format!(r"@\[([^\]]*?)\](?:{})?\((?s:(.*?))\)", IDENTIFIER_PATTERN),
            Self::Todo => String::from(r"@\[(?i:TODO)\]\((?s:(.*?))\)"),
            Self::AbridgedEmbeddedStyle => format!(r"\[([^\]\n]*?)\]{}?(?:{})?{}?\{{{}\}}", NEW_LINE_PATTERN, IDENTIFIER_PATTERN, NEW_LINE_PATTERN, ABRIDGED_STYLE_PATTERN),
            Self::Identifier => format!(r"\[(.*?)\]{}?#([\w-]*)", NEW_LINE_PATTERN),
            Self::EmbeddedStyle => format!(r"\[([^\]\n]*?)\]{}?(?:{})?{}?\{{\{{{}\}}\}}", NEW_LINE_PATTERN, IDENTIFIER_PATTERN, NEW_LINE_PATTERN, STYLE_PATTERN),
            Self::Highlight => String::from(r"==(.*)=="),
            Self::Comment => String::from(r"^//(.*)"),
            Self::Emoji => String::from(r":(\w*):"),
            Self::Checkbox => String::from(r"(\[\]|\[ \])"),
            Self::CheckboxChecked => String::from(r"(\[x\]|\[X\])"),
            Self::Superscript => String::from(r"\^(.*)\^"),
            Self::Subscript => String::from(r"~(.*)~"),
            Self::BoldStarVersion => String::from(r"\*\*(.*?)\*\*"),
            Self::BoldUnderscoreVersion => String::from(r"__(.*?)__"),
            Self::ItalicStarVersion => String::from(r"\*(.*?)\*"),
            Self::ItalicUnderscoreVersion => String::from(r"_(.*?)_"),
            Self::Strikethrough => String::from(r"~~(.*?)~~"),
            Self::Underlined => String::from(r"\+\+(.*?)\+\+"),
            Self::Link => String::from(r"\[([^\]]+)\]\(([^)]+)\)"),
            Self::InlineCode => String::from(r"`(.*?)`"),
            Self::InlineMath => format!(r#"\$([^$]+)\$"#),
            Self::GreekLetter => String::from(r"%(\w*?)%"),        // if it changes, fix greek letters rules
            Self::Escape => String::from(r"\\([\*\+\\~%\^\$@=\[\]!<>\{\}\(\)#-_\|\?&]+)"),
            Self::Reference => String::from(r"&([\w-]+)&"),
            Self::Cite => String::from(r"\^\[([\w_]+)\]"),
        }
    }

    pub fn incompatible_modifiers(&self) -> ModifiersBucket {
        match self {

            Self::InlineCode => ModifiersBucket::All,
            Self::InlineMath => ModifiersBucket::All,
            Self::Emoji => ModifiersBucket::All,
            Self::GreekLetter => ModifiersBucket::All,
            Self::Escape => ModifiersBucket::All,
            Self::Reference => ModifiersBucket::All,
            Self::Cite => ModifiersBucket::All,
            _ => ModifiersBucket::None
        }
    }

    pub fn modifier_pattern_regex(&self) -> &Regex {
        MODIFIER_PATTERNS_REGEX.get(&self.identifier()).unwrap()
    }
}


impl Into<BaseModifier> for StandardTextModifier {
    fn into(self) -> BaseModifier {
        BaseModifier::new(self.modifier_pattern(), self.modifier_pattern_regex().clone(), self.incompatible_modifiers())
    }
}