use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use super::{base_modifier::BaseModifier, constants::{IDENTIFIER_PATTERN, NEW_LINE, STYLE_PATTERN}, ModifierIdentifier, ModifierPattern, ModifiersBucket};


pub const PARAGRAPH_SEPARATOR_START: &str = r"(?m:^[ \t]*\r?\n)+";
pub const PARAGRAPH_SEPARATOR_END: &str = r"(?m:[ \t]*\r?\n){2}";


static MODIFIER_PATTERNS_REGEX: Lazy<HashMap<ModifierIdentifier, Regex>> = Lazy::new(|| {
    let mut regex: HashMap<ModifierIdentifier, Regex> = HashMap::new();

    StandardParagraphModifier::ordered().into_iter().for_each(|m| {
        regex.insert(m.identifier(), Regex::new(&m.modifier_pattern()).unwrap());
    });

    regex
});

static MODIFIER_PATTERNS_WITH_PARAGRAPH_SEPARATOR_REGEX: Lazy<HashMap<ModifierIdentifier, Regex>> = Lazy::new(|| {
    let mut regex: HashMap<ModifierIdentifier, Regex> = HashMap::new();

    StandardParagraphModifier::ordered().into_iter().for_each(|m| {
        regex.insert(m.identifier(), Regex::new(&m.modifier_pattern_with_paragraph_separator()).unwrap());
    });

    regex
});


#[derive(Debug, PartialEq, Clone)]
pub enum StandardParagraphModifier {
    List,
    ListItem,
    Table,
    Image,
    AbridgedImage,
    MultiImage,
    CodeBlock,
    CommentBlock,
    ExtendedBlockQuote,
    ExtendedBlockQuoteLine,
    FocusBlock,
    MathBlock,
    LineBreakDash,
    LineBreakStar,
    LineBreakPlus,
    CommonParagraph,

    // TODO: incompatible with EmbeddedParagraphStyleWithId
    // AbridgedEmbeddedParagraphStyleWithId,
    // AbridgedEmbeddedParagraphStyle,
    EmbeddedParagraphStyleWithId,
    // EmbeddedParagraphStyle,
    ParagraphIdentifier,
    PageBreak,
    Todo,
    AbridgedTodo,
    MultilineTodo,
}

impl StandardParagraphModifier {
    pub fn ordered() -> Vec<Self> {

        //! they must have the compatibility order
        vec![
            Self::CodeBlock,
            Self::MathBlock,
            Self::EmbeddedParagraphStyleWithId,
            // Self::EmbeddedParagraphStyle,
            Self::ParagraphIdentifier,
            Self::Table,
            Self::ExtendedBlockQuote,
            Self::FocusBlock,
            Self::List,
            Self::AbridgedTodo,
            Self::MultilineTodo,
            Self::Todo,
            Self::PageBreak,
            // Self::AbridgedEmbeddedParagraphStyleWithId,
            // Self::AbridgedEmbeddedParagraphStyle,
            Self::LineBreakDash,
            Self::LineBreakStar,
            Self::LineBreakPlus,
            Self::MultiImage,
            Self::AbridgedImage,
            Self::Image,
            Self::CommentBlock,
            Self::CommonParagraph,
        ]
    }

    pub fn identifier(&self) -> ModifierIdentifier {
        match self {
            Self::Image => String::from("image"),
            Self::CommonParagraph => String::from("common-paragraph"),
            Self::CodeBlock => String::from("code-block"),
            Self::MathBlock => String::from("math-block"),
            Self::ListItem => String::from("list-item"),
            Self::List => String::from("list"),
            Self::ExtendedBlockQuoteLine => String::from("extended-block-quote-line"),
            Self::ExtendedBlockQuote => String::from("extended-block-quote"),
            Self::LineBreakDash => String::from("line-break-dash"),
            Self::LineBreakStar => String::from("line-break-star"),
            Self::LineBreakPlus => String::from("line-break-plus"),
            Self::FocusBlock => String::from("focus-block"),
            // Self::AbridgedEmbeddedParagraphStyle => String::from("abridged-embedded-paragraph-style"),
            // Self::AbridgedEmbeddedParagraphStyleWithId => String::from("abridged-embedded-paragraph-style-with-id"),
            Self::ParagraphIdentifier => String::from("paragraph-identifier"),
            Self::EmbeddedParagraphStyleWithId => String::from("embedded-paragraph-style-with-id"),
            // Self::EmbeddedParagraphStyle => String::from("embedded-paragraph-style"),
            Self::PageBreak => String::from("page-break"),
            Self::Todo => String::from("todo"),
            Self::AbridgedTodo => String::from("abridged-todo"),
            Self::MultilineTodo => String::from("multiline-todo"),
            Self::AbridgedImage => String::from(r"abridged-image"),
            Self::MultiImage => String::from("multi-image"),
            Self::Table => String::from("table"),
            Self::CommentBlock => String::from("comment-block"),

            // _ => {
            //     log::warn!("there is NOT a modifier pattern for {:#?}", self);
            //     String::from(r"RULE TODO")
            // }
        }
    }

    // Return the modifier pattern
    pub fn modifier_pattern(&self) -> ModifierPattern {
        match *self {
            Self::Image => format!(r"!\[([^\]]*)\](?:{})?\(([^)]+)\)(?:\{{\{{{}\}}\}})?", IDENTIFIER_PATTERN, STYLE_PATTERN),
            Self::CommonParagraph => String::from(r#"(?s:(.*?))"#),
            Self::CommentBlock => String::from(r"<!--(?s:(.*?))-->"),
            Self::CodeBlock => format!(r"```\s?(\w+){}+(?s:(.*?)){}+```", NEW_LINE, NEW_LINE),
            Self::MathBlock => String::from(r#"\$\$((?s:.+?))\$\$"#),
            Self::ListItem => format!(r#"(?m:^([\t ]*)(-\[\]|-\[ \]|-\[x\]|-\[X\]|-|->|\||\*|\+|--|\d[\.)]?|[a-zA-Z]{{1,8}}[\.)]|&[^;]+;) (.*){}?)"#, NEW_LINE),
            Self::List => format!(r#"((?:{}+)+)"#, Self::ListItem.modifier_pattern()),
            Self::ExtendedBlockQuoteLine => String::from(r"(?m:^> (.*))"),
            Self::ExtendedBlockQuote => format!(r"(?ms:^> .*?)"),
            Self::LineBreakDash => String::from(r"(?m:^-{3,})"),
            Self::LineBreakStar => String::from(r"(?m:^\*{3,})"),
            Self::LineBreakPlus => String::from(r"(?m:^\+{3,})"),
            Self::FocusBlock => format!(r":::\s?(\w+){}(?s:(.*?)){}:::", NEW_LINE, NEW_LINE),
            Self::ParagraphIdentifier => format!(r"\[\[(?sx:(.*?))\]\]{}?{}", NEW_LINE, IDENTIFIER_PATTERN),
            Self::EmbeddedParagraphStyleWithId => format!(r"\[\[(?sx:(.*?))\]\]{}?(?:{})?{}?\{{\{{{}\}}\}}", NEW_LINE, IDENTIFIER_PATTERN, NEW_LINE, STYLE_PATTERN),
            // Self::EmbeddedParagraphStyle => format!(r"\[\[(?sx:(.*?))\]\]{}?\{{\{{{}\}}\}}", NEW_LINE, STYLE_PATTERN),
            // Self::AbridgedEmbeddedParagraphStyle => format!(r"\[\[(?sx:(.*?))\]\]{}?\{{{}\}}", NEW_LINE, ABRIDGED_STYLE),
            // Self::AbridgedEmbeddedParagraphStyleWithId => format!(r"\[\[(?sx:(.*?))\]\]{}?{}{}*\{{{}\}}", NEW_LINE, IDENTIFIER_PATTERN, NEW_LINE, ABRIDGED_STYLE),
            Self::PageBreak => String::from(r"(?m:^#{3,}$)"),
            Self::Todo => String::from(r"(?m:^(?i:TODO):?\s(?:(.*?))$)"),
            Self::AbridgedTodo => String::from(r"(?m:^(?i:TODO)$)"),
            Self::MultilineTodo => String::from(r"(?i:TODO):(?s:(.*?)):(?i:TODO)"),
            Self::AbridgedImage => format!(r"!\[\((.*)\)\](?:{})?(?:\{{\{{{}\}}\}})?", IDENTIFIER_PATTERN, STYLE_PATTERN),
            Self::MultiImage => String::from(r"!!(?::([\w-]+):)?\[\[(?s:(.*?))\]\]"),
            Self::Table => format!(r"(\|(.*)\|{}?)+(?:\|(.*)\|)(?U:{}?(?:\[(.*)\])?(?:{})?(?:\{{\{{{}\}}\}})?)?", NEW_LINE, NEW_LINE, IDENTIFIER_PATTERN, STYLE_PATTERN),
            
            // _ => {
            //     log::warn!("there is NOT a modifier pattern for {:#?}", self);
            //     String::from(r"RULE TODO")
            // }
        }
    }

    pub fn modifier_pattern_with_paragraph_separator(&self) -> String {
        let mp = self.modifier_pattern();

        format!(r"{}{}{}", PARAGRAPH_SEPARATOR_START, mp, PARAGRAPH_SEPARATOR_END)
    }

    pub fn incompatible_modifiers(&self) -> ModifiersBucket {
        match self {

            Self::Image => ModifiersBucket::All,
            Self::AbridgedImage => ModifiersBucket::All,
            Self::MultiImage => ModifiersBucket::All,
            Self::CodeBlock => ModifiersBucket::All,
            Self::MathBlock => ModifiersBucket::All,
            Self::CommentBlock => ModifiersBucket::All,

            _ => ModifiersBucket::None
        }
    }

    pub fn modifier_pattern_regex(&self) -> &Regex {
        MODIFIER_PATTERNS_REGEX.get(&self.identifier()).unwrap()
    }

    pub fn modifier_pattern_regex_with_paragraph_separator(&self) -> &Regex {
        MODIFIER_PATTERNS_WITH_PARAGRAPH_SEPARATOR_REGEX.get(&self.identifier()).unwrap()
    }
}

impl Into<BaseModifier> for StandardParagraphModifier {
    fn into(self) -> BaseModifier {
        BaseModifier::new(self.modifier_pattern_with_paragraph_separator(), self.modifier_pattern_regex_with_paragraph_separator().clone(), self.incompatible_modifiers())
    }
}


#[cfg(test)]
mod test {
    use regex::Regex;

    use super::StandardParagraphModifier;

    #[test]
    #[cfg(not(windows))]
    fn match_list() {
        let regex = Regex::new(StandardParagraphModifier::List.modifier_pattern_with_paragraph_separator().as_str()).unwrap();

        let list = concat!(
            "\n",
            "\n",
            "- [Element 1](#element-1)",
            "- [Element 2](#element-2)",
            "\n",
            "\n",
        );

        assert!(regex.is_match(list));

    }

}