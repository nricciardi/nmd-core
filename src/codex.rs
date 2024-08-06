//! `Codex` is a set of associations used to transform the text using a `Compiler`


pub mod codex_configuration;
pub mod modifier;


use std::collections::HashMap;
use getset::{Getters, Setters};
use self::modifier::standard_paragraph_modifier::StandardParagraphModifier;
use self::modifier::standard_text_modifier::StandardTextModifier;
use self::modifier::ModifierIdentifier;
use crate::output_format::OutputFormat;
use self::codex_configuration::CodexConfiguration;
use super::compiler::compilation_rule::constants::ESCAPE_HTML;
use super::compiler::compilation_rule::html_cite_rule::HtmlCiteRule;
use super::compiler::compilation_rule::html_extended_block_quote_rule::HtmlExtendedBlockQuoteRule;
use super::compiler::compilation_rule::html_greek_letter_rule::HtmlGreekLettersRule;
use super::compiler::compilation_rule::html_image_rule::HtmlImageRule;
use super::compiler::compilation_rule::html_list_rule::HtmlListRule;
use super::compiler::compilation_rule::html_table_rule::HtmlTableRule;
use super::compiler::compilation_rule::reference_rule::ReferenceRule;
use super::compiler::compilation_rule::replacement_rule::{ReplacementRule, ReplacementRuleReplacerPart};
use super::compiler::compilation_rule::CompilationRule;


/// Ordered collection of rules
/// A **rule** is defined as the actual text transformation
#[derive(Debug, Getters, Setters)]
pub struct Codex {

    #[getset(get = "pub", set = "pub")]
    configuration: CodexConfiguration,

    #[getset(get = "pub", set = "pub")]
    text_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>,

    #[getset(get = "pub", set = "pub")]
    paragraph_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>,

    #[getset(get = "pub", set = "pub")]
    chapter_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>,

    #[getset(get = "pub", set = "pub")]
    document_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>,
    
}

impl Codex {

    pub fn from(format: &OutputFormat, configuration: CodexConfiguration) -> Self {
        match format {
            OutputFormat::Html => Self::of_html(configuration)
        }
    }

    fn new(configuration: CodexConfiguration, text_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>, paragraph_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>, chapter_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>, document_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>) -> Codex {

        // TODO: check if there are all necessary rules based on theirs type

        Codex {
            configuration,
            text_rules,
            paragraph_rules,
            chapter_rules,
            document_rules
        }
    }

    /// Standard HTML `Codex`
    pub fn of_html(configuration: CodexConfiguration) -> Self {

        let text_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>> = HashMap::from([
            (
                StandardTextModifier::Todo.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Todo.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="todo"><div class="todo-title"></div><div class="todo-description">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ])) as Box<dyn CompilationRule>,
            ),
            (
                StandardTextModifier::BookmarkWithId.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::BookmarkWithId.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="bookmark" id="$2"><div class="bookmark-title">"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div><div class="bookmark-description">"#)),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"$3"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]))
            ),
            (
                StandardTextModifier::Bookmark.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Bookmark.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="bookmark"><div class="bookmark-title">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div><div class="bookmark-description">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$2"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]))
            ),
            (
                StandardTextModifier::GreekLetter.identifier().clone(),
                Box::new(HtmlGreekLettersRule::new()),
            ),
            (
                StandardTextModifier::AbridgedBookmarkWithId.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::AbridgedBookmarkWithId.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="abridged-bookmark" id="$2""#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"><div class="abridged-bookmark-title">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]))
            ),
            (
                StandardTextModifier::AbridgedBookmark.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::AbridgedBookmark.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="abridged-bookmark"><div class="abridged-bookmark-title">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]))
            ),
            (
                StandardTextModifier::EmbeddedStyleWithId.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::EmbeddedStyleWithId.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="identifier embedded-style" id="$2" style="$3">"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</span>"#)),
                ]))
            ),
            (
                StandardTextModifier::EmbeddedStyle.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::EmbeddedStyle.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="embedded-style" style="$2">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</span>"#)),
                ]))
            ),
            (
                StandardTextModifier::AbridgedEmbeddedStyleWithId.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::AbridgedEmbeddedStyleWithId.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="identifier abridged-embedded-style" id="$2" style="color: $3; background-color: $4; font-family: $5;">"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</span>"#)),
                ]))
            ),
            (
                StandardTextModifier::AbridgedEmbeddedStyle.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::AbridgedEmbeddedStyle.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="abridged-embedded-style" style="color: $2; background-color: $3; font-family: $4;">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</span>"#)),
                ]))
            ),
            (
                StandardTextModifier::Identifier.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Identifier.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="identifier" id="$2">"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</span>"#)),
                ]))
            ),
            (
                StandardTextModifier::Highlight.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Highlight.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<mark class="highlight">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</mark>"#)),
                ]))
            ),
            (
                StandardTextModifier::InlineMath.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::InlineMath.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="inline-math">$$"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"$$</span>"#)),
                ]))
            ),
            (
                StandardTextModifier::InlineCode.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::InlineCode.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<code class="language-markup inline-code">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</code>"#)),
                ]))
            ),
            (
                StandardTextModifier::BoldStarVersion.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::BoldStarVersion.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<strong class="bold">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</strong>"#)),
                ]))
            ),
            (
                StandardTextModifier::BoldUnderscoreVersion.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::BoldUnderscoreVersion.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<strong class="bold">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</strong>"#)),
                ]))
            ),
            (
                StandardTextModifier::ItalicStarVersion.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::ItalicStarVersion.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<em class="italic">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</em>"#)),
                ]))
            ),
            (
                StandardTextModifier::ItalicUnderscoreVersion.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::ItalicUnderscoreVersion.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<em class="italic">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</em>"#)),
                ]))
            ),
            (
                StandardTextModifier::Strikethrough.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Strikethrough.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<del class="strikethrough">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</del>"#)),
                ]))
            ),
            (
                StandardTextModifier::Underlined.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Underlined.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<u class="underlined">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</u>"#)),
                ]))
            ),
            (
                StandardTextModifier::Superscript.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Superscript.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<sup class="superscript">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</sup>"#)),
                ]))
            ),
            (
                StandardTextModifier::Subscript.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Subscript.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<sub class="subscript">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</sub>"#)),
                ]))
            ),
            (
                StandardTextModifier::Link.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Link.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<a href="$2" class="link">"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</a>"#)),
                ]))
            ),
            (
                StandardTextModifier::Comment.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Comment.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<!-- "#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#" -->"#)),
                ]))
            ),
            (
                StandardTextModifier::Checkbox.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Checkbox.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="checkbox checkbox-unchecked"></div>"#)),
                ])) as Box<dyn CompilationRule>,
            ),
            (
                StandardTextModifier::CheckboxChecked.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::CheckboxChecked.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="checkbox checkbox-checked"></div>"#)),
                ]))
            ),
            (
                StandardTextModifier::Emoji.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Emoji.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<i class="em-svg em-${1}" aria-role="presentation"></i>"#)),
                ]))
            ),
            (
                StandardTextModifier::Escape.identifier().clone(),
                Box::new(ReplacementRule::new(StandardTextModifier::Escape.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                ]))
            ),
            (
                StandardTextModifier::Reference.identifier().clone(),
                Box::new(ReferenceRule::new())
            ),
            (
                StandardTextModifier::Cite.identifier().clone(),
                Box::new(HtmlCiteRule::new())
            ),
        ]);

        let paragraph_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>> = HashMap::from([
            (
                StandardParagraphModifier::PageBreak.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::PageBreak.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="page-break"></div>"#)),
                ])) as Box<dyn CompilationRule>
            ),
            (
                StandardParagraphModifier::EmbeddedParagraphStyleWithId.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::EmbeddedParagraphStyleWithId.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="identifier embedded-paragraph-style" id="$2" style="$3" $nuid>"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div>"#)),
                ]).with_newline_fix(r"<br>".to_string())),
            ),
            (
                StandardParagraphModifier::EmbeddedParagraphStyle.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::EmbeddedParagraphStyle.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="embedded-paragraph-style" style="$2" $nuid>"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div>"#)),
                ]).with_newline_fix(r"<br>".to_string())),
            ),
            (
                StandardParagraphModifier::AbridgedEmbeddedParagraphStyleWithId.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::AbridgedEmbeddedParagraphStyleWithId.modifier_pattern().clone(),  vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="identifier abridged-embedded-paragraph-style" id="$2" $nuid style="color: $3; background-color: $4; font-family: $5;">"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div>"#)),
                ]).with_newline_fix(r"<br>".to_string())),
            ),
            (
                StandardParagraphModifier::AbridgedTodo.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::AbridgedTodo.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="todo abridged-todo" $nuid><div class="todo-title"></div><div class="todo-description">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::MultilineTodo.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::MultilineTodo.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="todo multiline-todo" $nuid><div class="todo-title"></div><div class="todo-description">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::AbridgedEmbeddedParagraphStyle.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::AbridgedEmbeddedParagraphStyle.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="abridged-embedded-paragraph-style" $nuid style="color: $2; background-color: $3; font-family: $4;">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div>"#)),
                ]).with_newline_fix(r"<br>".to_string())),
            ),
            (
                StandardParagraphModifier::ParagraphIdentifier.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::ParagraphIdentifier.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<span class="identifier" id="$2" $nuid>"#)).with_references_at(vec![2]),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</span>"#)),
                ]).with_newline_fix(r"<br>".to_string())),
            ),
            (
                StandardParagraphModifier::ExtendedBlockQuote.identifier().clone(),
                Box::new(HtmlExtendedBlockQuoteRule::new()),
            ),
            (
                StandardParagraphModifier::MathBlock.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::MathBlock.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<p class="math-block" $nuid>$$$$${1}$$$$</p>"#))
                ]))
            ),
            (
                StandardParagraphModifier::Image.identifier().clone(),
                Box::new(HtmlImageRule::new(StandardParagraphModifier::Image.identifier()))
            ),
            (
                StandardParagraphModifier::AbridgedImage.identifier().clone(),
                Box::new(HtmlImageRule::new(StandardParagraphModifier::AbridgedImage.identifier()))
            ),
            (
                StandardParagraphModifier::MultiImage.identifier().clone(),
                Box::new(HtmlImageRule::new(StandardParagraphModifier::MultiImage.identifier()))
            ),
            (
                StandardParagraphModifier::CodeBlock.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::CodeBlock.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<pre $nuid><code class="language-${1} code-block">"#)),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"$2"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</code></pre>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::List.identifier().clone(),
                Box::new(HtmlListRule::new()),
            ),
            (
                StandardParagraphModifier::FocusBlock.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::FocusBlock.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="focus-block focus-block-$1" $nuid><div class="focus-block-title focus-block-$1-title"></div><div class="focus-block-description focus-block-$1-description"">"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$2"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div></div>"#)),
                ]).with_newline_fix(r"<br>".to_string()))
            ),
            (
                StandardParagraphModifier::LineBreakDash.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::LineBreakDash.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<hr class="line-break line-break-dash" $nuid>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::LineBreakStar.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::LineBreakStar.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<hr class="line-break line-break-star" $nuid>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::LineBreakPlus.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::LineBreakPlus.modifier_pattern().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<hr class="line-break line-break-plus" $nuid>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::CommonParagraph.identifier().clone(),
                Box::new(ReplacementRule::new(StandardParagraphModifier::CommonParagraph.modifier_pattern_with_paragraph_separator().clone(), vec![
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"<p class="paragraph" $nuid>"#)),
                    ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                    ReplacementRuleReplacerPart::new_fixed(String::from(r#"</p>"#)),
                ]))
            ),
            (
                StandardParagraphModifier::Table.identifier().clone(),
                Box::new(HtmlTableRule::new())
            ),
        ]);

        let chapter_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>> = HashMap::new();

        Self::new(configuration, text_rules, paragraph_rules, chapter_rules, HashMap::new())
    }
}

#[cfg(test)]
mod test {

    use std::sync::{Arc, RwLock};

    use crate::{compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, Compiler}, loader::{loader_configuration::LoaderConfiguration, Loader}};

    use super::*;

    #[test]
    fn html_multiple_uses() {
        let codex = Codex::of_html(CodexConfiguration::default());

        let nmd_text = "This is a simple **nmd** text for test";
        let expected_result = r#"This is a simple <strong class="bold">nmd</strong> text for test"#;

        let outcome = Compiler::compile_str(&nmd_text, &OutputFormat::Html, &codex, &CompilationConfiguration::default(), Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

        assert_eq!(outcome.content(), expected_result);

        let nmd_text = "This is a simple *nmd* text for test";
        let expected_result = r#"This is a simple <em class="italic">nmd</em> text for test"#;

        let outcome = Compiler::compile_str(&nmd_text, &OutputFormat::Html, &codex, &CompilationConfiguration::default(), Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

        assert_eq!(outcome.content(), expected_result);
    }


    #[test]
    fn split_str_in_paragraphs() {
        let codex: Codex = Codex::of_html(CodexConfiguration::default());

        let nmd_text = 
r#"
```python

print("hello world")

```

`print("hello world)`
"#.trim();

        let paragraphs = Loader::load_paragraphs_from_str(nmd_text, &codex, &LoaderConfiguration::default()).unwrap();

        assert_eq!(paragraphs.len(), 2)
    }
}