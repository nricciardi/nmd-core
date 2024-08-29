//! `Codex` is a set of associations used to transform the text using a `Compiler`


pub mod modifier;


use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use getset::{Getters, Setters};
use indexmap::IndexMap;
use modifier::base_modifier::BaseModifier;
use modifier::Modifier;
use self::modifier::standard_paragraph_modifier::StandardParagraphModifier;
use self::modifier::standard_text_modifier::StandardTextModifier;
use crate::compilable_text::compilable_text_part::CompilableTextPart;
use crate::compilable_text::CompilableText;
use crate::compiler::compilation_rule::replacement_rule::replacement_rule_part::closure_replacement_rule_part::ClosureReplacementRuleReplacerPart;
use crate::compiler::compilation_rule::replacement_rule::replacement_rule_part::fixed_replacement_rule_part::FixedReplacementRuleReplacerPart;
use crate::compiler::compilation_rule::replacement_rule::replacement_rule_part::single_capture_group_replacement_rule_part::SingleCaptureGroupReplacementRuleReplacerPart;
use crate::compiler::compilation_rule::replacement_rule::ReplacementRule;
use crate::loader::paragraph_loading_rule::block_quote_paragraph_loading_rule::BlockQuoteParagraphLoadingRule;
use crate::loader::paragraph_loading_rule::image_paragraph_loading_rule::ImageParagraphLoadingRule;
use crate::loader::paragraph_loading_rule::list_paragraph_loading_rule::ListParagraphLoadingRule;
use crate::loader::paragraph_loading_rule::replacement_rule_paragraph_loading_rule::ReplacementRuleParagraphLoadingRule;
use crate::loader::paragraph_loading_rule::table_paragraph_loading_rule::TableParagraphLoadingRule;
use crate::loader::paragraph_loading_rule::ParagraphLoadingRule;
use crate::output_format::OutputFormat;
use crate::resource::resource_reference::ResourceReference;
use crate::utility::nmd_unique_identifier::NmdUniqueIdentifier;
use crate::utility::text_utility;
use super::compiler::compilation_rule::constants::ESCAPE_HTML;
use super::compiler::compilation_rule::html_cite_rule::HtmlCiteRule;
use super::compiler::compilation_rule::html_greek_letter_rule::HtmlGreekLettersRule;
use super::compiler::compilation_rule::reference_rule::ReferenceRule;
use super::compiler::compilation_rule::CompilationRule;


pub type CodexIdentifier = String;
pub type CodexModifiersMap = IndexMap<CodexIdentifier, Box<dyn Modifier>>;
pub type CodexCompilationRulesMap = HashMap<CodexIdentifier, Box<dyn CompilationRule>>;
pub type CodexLoadingRulesMap = HashMap<CodexIdentifier, Box<dyn ParagraphLoadingRule>>;


/// Ordered collection of rules
/// A **rule** is defined as the actual text transformation
#[derive(Debug, Getters, Setters)]
pub struct Codex {

    // #[getset(get = "pub", set = "pub")]
    // configuration: CodexConfiguration,


    #[getset(get = "pub", set = "pub")]
    text_modifiers: CodexModifiersMap,

    #[getset(get = "pub", set = "pub")]
    paragraph_modifiers: CodexModifiersMap,

    // #[getset(get = "pub", set = "pub")]
    // chapter_modifiers: CodexModifiersMap,

    #[getset(get = "pub", set = "pub")]
    text_compilation_rules: CodexCompilationRulesMap,

    #[getset(get = "pub", set = "pub")]
    paragraph_loading_rules: CodexLoadingRulesMap,

    // #[getset(get = "pub", set = "pub")]
    // document_rules: HashMap<ModifierIdentifier, Box<dyn CompilationRule>>,
    
}

impl Codex {

    pub fn from(format: &OutputFormat) -> Self {
        match format {
            OutputFormat::Html => Self::of_html()
        }
    }

    /// Create a new `Codex`
    pub fn new(text_modifiers: CodexModifiersMap, paragraph_modifiers: CodexModifiersMap,
                text_compilation_rules: CodexCompilationRulesMap, paragraph_loading_rules: CodexLoadingRulesMap,) -> Self {

        // TODO: check if there are all necessary rules based on theirs type

        Self {
            text_modifiers,
            paragraph_modifiers,
            text_compilation_rules,
            paragraph_loading_rules,
        }
    }

    /// Remove all modifiers and rules except those passed 
    pub fn retain(&mut self, identifiers: HashSet<CodexIdentifier>) {

        self.text_modifiers.retain(|id, _| identifiers.contains(id));
        self.paragraph_modifiers.retain(|id, _| identifiers.contains(id));

        self.text_compilation_rules.retain(|id, _| identifiers.contains(id));
        self.paragraph_loading_rules.retain(|id, _| identifiers.contains(id));
    }


    /// Remove modifiers and rules 
    pub fn remove(&mut self, identifiers: HashSet<CodexIdentifier>) {
        identifiers.iter().for_each(|id: &CodexIdentifier| {

            self.text_modifiers.shift_remove(id);
            self.paragraph_modifiers.shift_remove(id);

            self.text_compilation_rules.remove(id);
            self.paragraph_loading_rules.remove(id);
        });
    }

    fn html_nuid_tag_or_nothing(nuid: Option<&NmdUniqueIdentifier>) -> String {
        if let Some(nuid) = nuid {
            return format!(r#"data-nuid="{}""#, nuid);
          }

          String::new()
    }

    /// Standard HTML `Codex`
    pub fn of_html() -> Self {

        let mut text_modifiers: CodexModifiersMap = CodexModifiersMap::new();

        StandardTextModifier::ordered().into_iter().for_each(|tm| {
            text_modifiers.insert(tm.identifier(), Box::new(Into::<BaseModifier>::into(tm)) as Box<dyn Modifier>);
        });

        let mut paragraph_modifiers: CodexModifiersMap = CodexModifiersMap::new();

        StandardParagraphModifier::ordered().into_iter().for_each(|tm| {
            paragraph_modifiers.insert(tm.identifier(), Box::new(Into::<BaseModifier>::into(tm)) as Box<dyn Modifier>);
        });

        let text_rules: CodexCompilationRulesMap = CodexCompilationRulesMap::from([
            (
                StandardTextModifier::Todo.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Todo.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="todo"><div class="todo-title"></div><div class="todo-description">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Todo.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#))),
                    ]
                )) as Box<dyn CompilationRule>
            ),
            (
                StandardTextModifier::BookmarkWithId.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::BookmarkWithId.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<div class="bookmark" id="{}"><div class="bookmark-title">"#,
                                    ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::BookmarkWithId.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div><div class="bookmark-description">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(3, ESCAPE_HTML.clone(), StandardTextModifier::BookmarkWithId.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Bookmark.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Bookmark.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="bookmark"><div class="bookmark-title">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Bookmark.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div><div class="bookmark-description">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(3, ESCAPE_HTML.clone(), StandardTextModifier::Bookmark.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::GreekLetter.identifier().clone(),
                Box::new(HtmlGreekLettersRule::new()),
            ),
            (
                StandardTextModifier::AbridgedBookmark.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::AbridgedBookmark.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="abridged-bookmark"><div class="abridged-bookmark-title">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::AbridgedBookmark.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::AbridgedBookmarkWithId.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::AbridgedBookmarkWithId.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<div class="abridged-bookmark" id="{}"><div class="abridged-bookmark-title">"#,
                                    ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::AbridgedBookmarkWithId.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::EmbeddedStyleWithId.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::EmbeddedStyleWithId.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<span class="identifier embedded-style" id="{}" style="{}">"#,
                                    ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                    captures.get(3).unwrap().as_str(),
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::EmbeddedStyleWithId.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::EmbeddedStyle.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::EmbeddedStyle.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, _| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<span class="identifier embedded-style" style="{}">"#,
                                    captures.get(2).unwrap().as_str(),
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::EmbeddedStyle.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::AbridgedEmbeddedStyleWithId.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::AbridgedEmbeddedStyleWithId.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                            let mut color_style = String::new();
                            if let Some(color) = captures.get(3) {
                                color_style = format!("color: {};", color.as_str());
                            }

                            let mut bg_style = String::new();
                            if let Some(bg) = captures.get(4) {
                                bg_style = format!("background-color: {};", bg.as_str());
                            }

                            let mut font_style = String::new();
                            if let Some(font) = captures.get(5) {
                                font_style = format!("font-family: {};", font.as_str());
                            }

                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<span class="identifier abridged-embedded-style" id="{}" style="{} {} {}">"#,
                                    ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                    color_style,
                                    bg_style,
                                    font_style,
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::AbridgedEmbeddedStyleWithId.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::AbridgedEmbeddedStyle.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::AbridgedEmbeddedStyle.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, _| {

                            let mut color_style = String::new();
                            if let Some(color) = captures.get(2) {
                                color_style = format!("color: {};", color.as_str());
                            }

                            let mut bg_style = String::new();
                            if let Some(bg) = captures.get(3) {
                                bg_style = format!("background-color: {};", bg.as_str());
                            }

                            let mut font_style = String::new();
                            if let Some(font) = captures.get(4) {
                                font_style = format!("font-family: {};", font.as_str());
                            }
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<span class="abridged-embedded-style" style="{} {} {}">"#,
                                    color_style,
                                    bg_style,
                                    font_style,
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::AbridgedEmbeddedStyle.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::Identifier.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Identifier.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<span class="identifier" id="{}">"#,
                                    ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Identifier.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::Highlight.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Highlight.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<mark class="highlight">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Highlight.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</mark>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::InlineMath.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::InlineMath.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<span class="inline-math">$$"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, vec![], StandardTextModifier::InlineMath.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"$$</span>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::InlineCode.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::InlineCode.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<code class="language-markup inline-code">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::InlineCode.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</code>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::BoldStarVersion.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::BoldStarVersion.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<strong class="bold">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::BoldStarVersion.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</strong>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::BoldUnderscoreVersion.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::BoldUnderscoreVersion.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<strong class="bold">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::BoldUnderscoreVersion.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</strong>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::ItalicStarVersion.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::ItalicStarVersion.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<em class="italic">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::ItalicStarVersion.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</em>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::ItalicUnderscoreVersion.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::ItalicUnderscoreVersion.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<em class="italic">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::ItalicUnderscoreVersion.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</em>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Strikethrough.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Strikethrough.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<del class="strikethrough">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Strikethrough.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</del>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Underlined.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Underlined.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<u class="underlined">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Underlined.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</u>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Superscript.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Superscript.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<sup class="superscript">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Superscript.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</sup>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Subscript.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Subscript.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<sub class="subscript">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Subscript.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</sub>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Link.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Link.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<a href="{}" class="link">"#,
                                    ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Link.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</a>"#)))
                    ]
                ))
            ),
            (
                StandardTextModifier::Comment.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Comment.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<!-- "#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Comment.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#" -->"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Checkbox.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Checkbox.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="checkbox checkbox-unchecked"></div>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::CheckboxChecked.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::CheckboxChecked.modifier_pattern().clone(),
                    vec![
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="checkbox checkbox-checked"></div>"#))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Emoji.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Emoji.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, _| {
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<i class="em-svg em-{}" aria-role="presentation"></i>"#,
                                    captures.get(1).unwrap().as_str(),
                                ))
                            ]))
                        }))),
                    ]
                ))
            ),
            (
                StandardTextModifier::Escape.identifier().clone(),
                Box::new(ReplacementRule::new(
                    StandardTextModifier::Escape.modifier_pattern().clone(),
                    vec![
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Escape.incompatible_modifiers())),
                    ]
                ))
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

        let paragraph_rules: CodexLoadingRulesMap = CodexLoadingRulesMap::from([
            (
                    StandardParagraphModifier::Table.identifier(),
                    Box::new(TableParagraphLoadingRule::new()) as Box<dyn ParagraphLoadingRule>
            ),
            (
                    StandardParagraphModifier::PageBreak.identifier().clone(),
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::PageBreak.modifier_pattern_with_paragraph_separator().clone(),
                            vec![
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="page-break"></div>"#)))
                            ]
                        )
                    ))
            ),
            (
                StandardParagraphModifier::EmbeddedParagraphStyleWithId.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::EmbeddedParagraphStyleWithId.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, cco| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="identifier embedded-paragraph-style" id="{}" style="{}" {}>"#,
                                        ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                        captures.get(3).unwrap().as_str(),
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::EmbeddedParagraphStyleWithId.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::EmbeddedParagraphStyle.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::EmbeddedParagraphStyle.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="embedded-paragraph-style" style="{}" {}>"#,
                                        captures.get(2).unwrap().as_str(),
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::EmbeddedParagraphStyle.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::AbridgedEmbeddedParagraphStyleWithId.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::AbridgedEmbeddedParagraphStyleWithId.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, cco| {

                                let mut color_style = String::new();
                                if let Some(color) = captures.get(3) {
                                    color_style = format!("color: {};", color.as_str());
                                }

                                let mut bg_style = String::new();
                                if let Some(bg) = captures.get(4) {
                                    bg_style = format!("background-color: {};", bg.as_str());
                                }

                                let mut font_style = String::new();
                                if let Some(font) = captures.get(5) {
                                    font_style = format!("font-family: {};", font.as_str());
                                }

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="identifier abridged-embedded-paragraph-style" id="{}" {} style="{} {} {}">"#,
                                        ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        color_style,
                                        bg_style,
                                        font_style,
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::AbridgedEmbeddedParagraphStyleWithId.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::AbridgedTodo.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::AbridgedTodo.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="todo abridged-todo" {}>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::AbridgedTodo.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::MultilineTodo.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::MultilineTodo.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="todo multiline-todo" {}><div class="todo-title"></div><div class="todo-description">"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::MultilineTodo.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::AbridgedEmbeddedParagraphStyle.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::AbridgedEmbeddedParagraphStyle.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {

                                let mut color_style = String::new();
                                if let Some(color) = captures.get(2) {
                                    color_style = format!("color: {};", color.as_str());
                                }

                                let mut bg_style = String::new();
                                if let Some(bg) = captures.get(3) {
                                    bg_style = format!("background-color: {};", bg.as_str());
                                }

                                let mut font_style = String::new();
                                if let Some(font) = captures.get(4) {
                                    font_style = format!("font-family: {};", font.as_str());
                                }

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="abridged-embedded-paragraph-style" {} style="{} {} {}">"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        color_style,
                                        bg_style,
                                        font_style,
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::AbridgedEmbeddedParagraphStyle.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::ParagraphIdentifier.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::ParagraphIdentifier.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, cco| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<span class="identifier" id="{}" {}>"#,
                                        ResourceReference::of_internal_from_without_sharp(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::ParagraphIdentifier.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::ExtendedBlockQuote.identifier().clone(),
                Box::new(BlockQuoteParagraphLoadingRule::new()),
            ),
            (
                StandardParagraphModifier::MathBlock.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::MathBlock.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<p class="math-block" {}>{}</p>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        captures.get(1).unwrap().as_str(),
                                    )
                                )
                                ]))
                            }))),
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::Image.identifier().clone(),
                Box::new(ImageParagraphLoadingRule::SingleImage)
            ),
            (
                StandardParagraphModifier::AbridgedImage.identifier().clone(),
                Box::new(ImageParagraphLoadingRule::AbridgedImage)
            ),
            (
                StandardParagraphModifier::MultiImage.identifier().clone(),
                Box::new(ImageParagraphLoadingRule::MultiImage)
            ),
            (
                StandardParagraphModifier::CodeBlock.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::CodeBlock.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {

                                let mut lang_class = String::from("language-markup");
                                if let Some(lang) = captures.get(1) {
                                    lang_class = format!("language-{}", lang.as_str());
                                }

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<pre {}><code class="{} code-block">{}</code></pre>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        lang_class,
                                        text_utility::replace(captures.get(2).unwrap().as_str(), &ESCAPE_HTML),
                                    )
                                )
                                ]))
                            }))),
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::List.identifier().clone(),
                Box::new(ListParagraphLoadingRule::new()),
            ),
            (
                StandardParagraphModifier::FocusBlock.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::FocusBlock.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {

                                let mut focus_block_type = String::from("quote");
                                if let Some(t) = captures.get(1) {
                                    focus_block_type = t.as_str().to_string();
                                }

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="focus-block focus-block-{}" {}><div class="focus-block-title focus-block-{}-title"></div><div class="focus-block-description focus-block-{}-description">"#,
                                        focus_block_type,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        focus_block_type,
                                        focus_block_type,
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::FocusBlock.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::LineBreakDash.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::LineBreakDash.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<hr class="line-break line-break-dash" {}>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    )
                                )
                                ]))
                            }))),
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::LineBreakStar.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::LineBreakDash.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<hr class="line-break line-break-star" {}>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    )
                                )
                                ]))
                            }))),
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::LineBreakPlus.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::LineBreakDash.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<hr class="line-break line-break-plus" {}>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    )
                                )
                                ]))
                            }))),
                        ]
                    )
                ))
            ),
            (
                StandardParagraphModifier::CommonParagraph.identifier().clone(),
                Box::new(ReplacementRuleParagraphLoadingRule::new(
                    ReplacementRule::new(
                        StandardParagraphModifier::CommonParagraph.modifier_pattern_with_paragraph_separator().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {

                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<p class="paragraph" {}>"#,
                                        Self::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::CommonParagraph.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</p>"#)))
                        ]
                    )
                ))
            ),
        ]);

        Self::new(
            text_modifiers,
            paragraph_modifiers,
            text_rules,
            paragraph_rules,
        )
    }
}

#[cfg(test)]
mod test {

    use indexmap::IndexMap;
    use modifier::base_modifier::BaseModifier;
    use super::*;


    #[test]
    fn correct_order() {

        let codex = Codex::new(
            IndexMap::from([
                (
                    String::from("a"),
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>
                ),
                (
                    String::from("b"),
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::AbridgedEmbeddedStyle)) as Box<dyn Modifier>
                ),
                (
                    String::from("e"),
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::BookmarkWithId)) as Box<dyn Modifier>
                ),
                (
                    String::from("c"),
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::ColoredText)) as Box<dyn Modifier>
                ),
                (
                    String::from("i"),
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Link)) as Box<dyn Modifier>
                ),
            ]),
            IndexMap::new(),
            HashMap::new(),
            HashMap::new(),
        );


        let ids: Vec<String> = codex.text_modifiers.into_iter().map(|tm| tm.0).collect();

        assert_eq!(ids.join(""), "abeci");
    }
}