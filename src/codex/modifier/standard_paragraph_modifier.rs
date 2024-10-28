use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use super::{base_modifier::BaseModifier, constants::{build_strict_reserved_line_pattern, IDENTIFIER_PATTERN, MULTI_LINES_CONTENT_PATTERN, MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, NEW_LINE_PATTERN, STYLE_PATTERN}, ModifierIdentifier, ModifierPattern, ModifiersBucket};


pub const PARAGRAPH_SEPARATOR_START: &str = r"(?m:^[ \t]*\r?\n)+";
pub const PARAGRAPH_SEPARATOR_END: &str = r"(?m:[ \t]*\r?\n){2}";


static MODIFIER_PATTERNS_REGEX: Lazy<HashMap<ModifierIdentifier, Regex>> = Lazy::new(|| {
    let mut regex: HashMap<ModifierIdentifier, Regex> = HashMap::new();

    StandardParagraphModifier::ordered().into_iter().for_each(|m| {
        regex.insert(m.identifier(), Regex::new(&m.modifier_pattern()).unwrap());
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
    EmbeddedParagraphStyle,
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
            Self::EmbeddedParagraphStyle,
            Self::ParagraphIdentifier,
            Self::Table,
            Self::ExtendedBlockQuote,
            Self::FocusBlock,
            Self::List,
            Self::AbridgedTodo,
            Self::MultilineTodo,
            Self::Todo,
            Self::PageBreak,
            Self::LineBreakDash,
            Self::LineBreakStar,
            Self::LineBreakPlus,
            Self::MultiImage,
            Self::AbridgedImage,
            Self::Image,
            Self::CommentBlock,
            // Self::CommonParagraph, => fallback
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
            Self::ParagraphIdentifier => String::from("paragraph-identifier"),
            Self::EmbeddedParagraphStyle => String::from("embedded-paragraph-style"),
            Self::PageBreak => String::from("page-break"),
            Self::Todo => String::from("todo"),
            Self::AbridgedTodo => String::from("abridged-todo"),
            Self::MultilineTodo => String::from("multiline-todo"),
            Self::AbridgedImage => String::from(r"abridged-image"),
            Self::MultiImage => String::from("multi-image"),
            Self::Table => String::from("table"),
            Self::CommentBlock => String::from("comment-block"),
        }
    }

    // Return the modifier pattern
    pub fn modifier_pattern(&self) -> ModifierPattern {
        match *self {
            Self::Image => build_strict_reserved_line_pattern(&format!(r"!\[([^\]]*)\](?:{})?\(([^)]+)\)(?:\{{\{{{}\}}\}})?", IDENTIFIER_PATTERN, STYLE_PATTERN)),
            Self::AbridgedImage => build_strict_reserved_line_pattern(&format!(r"!\[\((.*)\)\](?:{})?(?:\{{\{{{}\}}\}})?", IDENTIFIER_PATTERN, STYLE_PATTERN)),
            Self::MultiImage => build_strict_reserved_line_pattern(r"!!(?::([\w-]+):)?\[\[(?s:(.*?))\]\]"),
            Self::CommonParagraph => format!("{}{}{}", MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, NEW_LINE_PATTERN, NEW_LINE_PATTERN),
            Self::CommentBlock => format!(r"<!--(?s:(.*?))-->"),
            Self::CodeBlock => format!(r"{}{}{}{}", build_strict_reserved_line_pattern(r"```[ \t]*(\w+)?"), NEW_LINE_PATTERN, MULTI_LINES_CONTENT_PATTERN, build_strict_reserved_line_pattern("```")),
            Self::MathBlock => format!(r"{}{}{}", build_strict_reserved_line_pattern(r"\$\$"), MULTI_LINES_CONTENT_PATTERN, build_strict_reserved_line_pattern(r"\$\$")),
            Self::FocusBlock => format!(r"{}{}{}{}", build_strict_reserved_line_pattern(r":::[ \t]*(\w+)?"), NEW_LINE_PATTERN, MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN, build_strict_reserved_line_pattern(":::")),
            Self::ListItem => format!(r#"(?m:^([\t ]*)(-\[\]|-\[ \]|-\[x\]|-\[X\]|-|->|\||\*|\+|--|\d[\.)]?|[a-zA-Z]{{1,8}}[\.)]|&[^;]+;) (.*){}?)"#, NEW_LINE_PATTERN),
            Self::List => format!(r#"((?:{}+)+)"#, Self::ListItem.modifier_pattern()),
            Self::ExtendedBlockQuoteLine => String::from(r"(?m:^> (.*))"),
            Self::ExtendedBlockQuote => format!(r"(?m)(^[ \t]*>.*(?:\r?\n>.*)*)"),
            Self::LineBreakDash => build_strict_reserved_line_pattern(r"-{3,}"),
            Self::LineBreakStar => build_strict_reserved_line_pattern(r"\*{3,}"),
            Self::LineBreakPlus => build_strict_reserved_line_pattern(r"\+{3,}"),
            Self::ParagraphIdentifier => format!(r"\[\[(?sx:(.*?))\]\]{}?{}", NEW_LINE_PATTERN, IDENTIFIER_PATTERN),
            Self::EmbeddedParagraphStyle => format!(r"\[\[(?sx:(.*?))\]\]{}?(?:{})?{}?\{{\{{{}\}}\}}", NEW_LINE_PATTERN, IDENTIFIER_PATTERN, NEW_LINE_PATTERN, STYLE_PATTERN),
            Self::PageBreak => build_strict_reserved_line_pattern(r"#{3,}"),
            Self::Todo => build_strict_reserved_line_pattern(r"(?i:TODO):?\s(?:(.*?))"),
            Self::AbridgedTodo => build_strict_reserved_line_pattern(r"(?i:TODO)"),
            Self::MultilineTodo => format!("{}{}", build_strict_reserved_line_pattern(r"(?i:TODO):"), r"(?s:(.*?)):(?i:TODO)"),
            Self::Table => format!(r"(\|(.*)\|{}?)+(?:\|(.*)\|)(?U:{}?(?:\[(.*)\])?(?:{})?(?:\{{\{{{}\}}\}})?)?", NEW_LINE_PATTERN, NEW_LINE_PATTERN, IDENTIFIER_PATTERN, STYLE_PATTERN),
        }
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
}

impl Into<BaseModifier> for StandardParagraphModifier {
    fn into(self) -> BaseModifier {
        BaseModifier::new(self.modifier_pattern(), self.modifier_pattern_regex().clone(), self.incompatible_modifiers())
    }
}


#[cfg(test)]
mod test {
    use regex::Regex;

    use super::StandardParagraphModifier;

    #[test]
    #[cfg(not(windows))]
    fn match_list() {
        let regex = Regex::new(StandardParagraphModifier::List.modifier_pattern().as_str()).unwrap();

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