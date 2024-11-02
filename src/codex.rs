//! `Codex` is a set of associations used to transform the text using a `Compiler`


pub mod modifier;


use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;
use getset::{Getters, Setters};
use indexmap::IndexMap;
use modifier::base_modifier::BaseModifier;
use modifier::{Modifier, ModifierIdentifier};
use self::modifier::standard_paragraph_modifier::StandardParagraphModifier;
use self::modifier::standard_text_modifier::StandardTextModifier;
use crate::assembler::html_assembler::HtmlAssembler;
use crate::assembler::Assembler;
use crate::compilable_text::compilable_text_part::CompilableTextPart;
use crate::compilable_text::CompilableText;
use crate::compilation::compilation_rule::replacement_rule::replacement_rule_part::closure_replacement_rule_part::ClosureReplacementRuleReplacerPart;
use crate::compilation::compilation_rule::replacement_rule::replacement_rule_part::fixed_replacement_rule_part::FixedReplacementRuleReplacerPart;
use crate::compilation::compilation_rule::replacement_rule::replacement_rule_part::single_capture_group_replacement_rule_part::SingleCaptureGroupReplacementRuleReplacerPart;
use crate::compilation::compilation_rule::replacement_rule::ReplacementRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::block_quote_paragraph_loading_rule::BlockQuoteParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::common_paragraph_loading_rule::CommonParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::focus_block_paragraph_loading_rule::FocusBlockParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::image_paragraph_loading_rule::ImageParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::list_paragraph_loading_rule::ListParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::metadata_wrapper_paragraph_loading_rule::MetadataWrapperParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::replacement_rule_paragraph_loading_rule::ReplacementRuleParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::table_paragraph_loading_rule::TableParagraphLoadingRule;
use crate::dossier::document::chapter::paragraph::paragraph_loading_rule::{MultiParagraphLoadingRule, ParagraphLoadingRule};
use crate::output_format::OutputFormat;
use crate::resource::resource_reference::ResourceReference;
use crate::utility::text_utility;
use super::compilation::compilation_rule::constants::ESCAPE_HTML;
use super::compilation::compilation_rule::html_cite_rule::HtmlCiteRule;
use super::compilation::compilation_rule::html_greek_letter_rule::HtmlGreekLettersRule;
use super::compilation::compilation_rule::reference_rule::ReferenceRule;
use super::compilation::compilation_rule::CompilationRule;


pub type TextModifierOrderedMap = IndexMap<ModifierIdentifier, (Box<dyn Modifier>, Box<dyn CompilationRule>)>;
pub type ParagraphModifierOrderedMap = IndexMap<ModifierIdentifier, (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)>;
pub type FallbackParagraph = (ModifierIdentifier, Box<dyn MultiParagraphLoadingRule>);


/// Ordered collection of rules
/// A **rule** is defined as the actual text transformation
#[derive(Debug, Getters, Setters)]
pub struct Codex {

    #[getset(get = "pub", set = "pub")]
    text_modifiers: TextModifierOrderedMap,

    #[getset(get = "pub", set = "pub")]
    paragraph_modifiers: ParagraphModifierOrderedMap,

    #[getset(get = "pub", set = "pub")]
    fallback_paragraph: Option<FallbackParagraph>,
    
    #[getset(get = "pub", set = "pub")]
    assembler: Box<dyn Assembler>
}

impl Codex {

    pub fn from(format: &OutputFormat) -> Self {
        match format {
            OutputFormat::Html => Self::of_html()
        }
    }

    /// Create a new `Codex`
    pub fn new(text_modifiers: TextModifierOrderedMap, paragraph_modifiers: ParagraphModifierOrderedMap,
                fallback_paragraph_modifier: Option<FallbackParagraph>,
                assembler: Box<dyn Assembler>,) -> Self {

        Self {
            text_modifiers,
            paragraph_modifiers,
            fallback_paragraph: fallback_paragraph_modifier,
            assembler
        }
    }

    /// Remove all modifiers and rules except those passed 
    pub fn retain(&mut self, identifiers: HashSet<ModifierIdentifier>) {

        self.text_modifiers.retain(|id, _| identifiers.contains(id));
        self.paragraph_modifiers.retain(|id, _| identifiers.contains(id));
    }


    /// Remove modifiers and rules 
    pub fn remove(&mut self, identifiers: HashSet<ModifierIdentifier>) {
        identifiers.iter().for_each(|id: &ModifierIdentifier| {

            self.text_modifiers.shift_remove(id);
            self.paragraph_modifiers.shift_remove(id);
        });
    }

    /// Standard HTML `Codex`
    pub fn of_html() -> Self {

        let text_rules: TextModifierOrderedMap = TextModifierOrderedMap::from([
            (
                StandardTextModifier::Todo.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Todo)) as Box<dyn Modifier>,
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Todo.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="todo"><div class="todo-title"></div><div class="todo-description">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Todo.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#))),
                        ]
                    )) as Box<dyn CompilationRule>,
                ) as (Box<dyn Modifier>, Box<dyn CompilationRule>)
            ),
            (
                StandardTextModifier::Bookmark.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Bookmark)),
                    Box::new(ReplacementRule::new(
                    StandardTextModifier::Bookmark.modifier_pattern().clone(),
                    vec![
                        Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {

                            let mut identifier_class = String::new();
                            let mut id_attr = String::new();
                            if let Some(raw_id) = captures.get(2) {
                                id_attr = format!(
                                    r#"id="{}""#,
                                    ResourceReference::of_internal_from_without_sharp(raw_id.as_str(), cco.document_name().as_ref())?.build(),
                                );

                                identifier_class = String::from("identifier");
                            }
    
                            Ok(CompilableText::from(vec![
                                CompilableTextPart::new_fixed(format!(
                                    r#"<div class="{} bookmark" {}><div class="bookmark-title">"#,
                                    identifier_class,
                                    id_attr
                                ))
                            ]))
                        }))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Bookmark.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div><div class="bookmark-description">"#))),
                        Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(3, ESCAPE_HTML.clone(), StandardTextModifier::Bookmark.incompatible_modifiers())),
                        Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#))),
                    ])),
                )
            ),
            (
                StandardTextModifier::GreekLetter.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::GreekLetter)),
                    Box::new(HtmlGreekLettersRule::new())
                ),
            ),
            (
                StandardTextModifier::AbridgedBookmark.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::AbridgedBookmark)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::AbridgedBookmark.modifier_pattern().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                                let mut identifier_class = String::new();
                                let mut id_attr = String::new();
                                if let Some(raw_id) = captures.get(2) {
                                    id_attr = format!(
                                        r#"id="{}""#,
                                        ResourceReference::of_internal_from_without_sharp(raw_id.as_str(), cco.document_name().as_ref())?.build(),
                                    );
    
                                    identifier_class = String::from("identifier");
                                }
        
                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<div class="{} abridged-bookmark" {}><div class="abridged-bookmark-title">"#,
                                        identifier_class,
                                        id_attr,
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::AbridgedBookmark.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div></div>"#)))
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::EmbeddedStyle.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::EmbeddedStyle)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::EmbeddedStyle.modifier_pattern().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                                let (styles, classes) = text_utility::split_styles_and_classes(captures.get(3).unwrap().as_str());
    
                                let mut identifier_class = String::new();
                                let mut id_attr = String::new();
                                if let Some(raw_id) = captures.get(2) {
                                    id_attr = format!(
                                        r#"id="{}""#,
                                        ResourceReference::of_internal_from_without_sharp(raw_id.as_str(), cco.document_name().as_ref())?.build(),
                                    );
    
                                    identifier_class = String::from("identifier");
                                } 
    
                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<span class="{} embedded-style {}" {} style="{}">"#,
                                        identifier_class,
                                        classes.unwrap_or(String::new()),
                                        id_attr,
                                        styles.unwrap_or(String::new())
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::EmbeddedStyle.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</span>"#)))
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::AbridgedEmbeddedStyle.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::AbridgedEmbeddedStyle)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::AbridgedEmbeddedStyle.modifier_pattern().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
    
                                let mut identifier_class = String::new();
                                let mut id_attr = String::new();
                                if let Some(raw_id) = captures.get(2) {
                                    id_attr = format!(
                                        r#"id="{}""#,
                                        ResourceReference::of_internal_from_without_sharp(raw_id.as_str(), cco.document_name().as_ref())?.build(),
                                    );
    
                                    identifier_class = String::from("identifier");
                                } 
        
                                let mut color_style = String::new();
                                if let Some(color) = captures.get(4) {
                                    color_style = format!("color: {};", color.as_str());
                                }
    
                                let mut bg_style = String::new();
                                if let Some(bg) = captures.get(5) {
                                    bg_style = format!("background-color: {};", bg.as_str());
                                }
    
                                let mut font_style = String::new();
                                if let Some(font) = captures.get(6) {
                                    font_style = format!("font-family: {};", font.as_str());
                                }
    
                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<span class="{} abridged-embedded-style" {} style="{} {} {}">"#,
                                        identifier_class,
                                        id_attr,
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
                )
            ),
            (
                StandardTextModifier::Identifier.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Identifier)),
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
                )
            ),
            (
                StandardTextModifier::Highlight.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Highlight)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Highlight.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<mark class="highlight">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Highlight.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</mark>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::InlineMath.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::InlineMath)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::InlineMath.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<span class="inline-math">$"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, vec![], StandardTextModifier::InlineMath.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"$</span>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::InlineCode.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::InlineCode)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::InlineCode.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<code class="language-markup inline-code">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::InlineCode.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</code>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::BoldStarVersion.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::BoldStarVersion.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<strong class="bold">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::BoldStarVersion.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</strong>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::BoldUnderscoreVersion.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldUnderscoreVersion)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::BoldUnderscoreVersion.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<strong class="bold">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::BoldUnderscoreVersion.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</strong>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::ItalicStarVersion.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::ItalicStarVersion)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::ItalicStarVersion.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<em class="italic">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::ItalicStarVersion.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</em>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::ItalicUnderscoreVersion.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::ItalicUnderscoreVersion)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::ItalicUnderscoreVersion.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<em class="italic">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::ItalicUnderscoreVersion.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</em>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Strikethrough.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Strikethrough)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Strikethrough.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<del class="strikethrough">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Strikethrough.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</del>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Underlined.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Underlined)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Underlined.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<u class="underlined">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Underlined.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</u>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Superscript.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Superscript)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Superscript.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<sup class="superscript">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Superscript.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</sup>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Subscript.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Subscript)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Subscript.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<sub class="subscript">"#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Subscript.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</sub>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Link.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Link)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Link.modifier_pattern().clone(),
                        vec![
                            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, _, _, _, cco| {
        
                                Ok(CompilableText::from(vec![
                                    CompilableTextPart::new_fixed(format!(
                                        r#"<a href="{}" class="link">"#,
                                        ResourceReference::of(captures.get(2).unwrap().as_str(), cco.document_name().as_ref())?.build(),
                                    ))
                                ]))
                            }))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Link.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</a>"#)))
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Comment.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Comment)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Comment.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<!-- "#))),
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Comment.incompatible_modifiers())),
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#" -->"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Checkbox.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Checkbox)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Checkbox.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="checkbox checkbox-unchecked"></div>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::CheckboxChecked.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::CheckboxChecked)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::CheckboxChecked.modifier_pattern().clone(),
                        vec![
                            Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="checkbox checkbox-checked"></div>"#))),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Emoji.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Emoji)),
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
                )
            ),
            (
                StandardTextModifier::Escape.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Escape)),
                    Box::new(ReplacementRule::new(
                        StandardTextModifier::Escape.modifier_pattern().clone(),
                        vec![
                            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardTextModifier::Escape.incompatible_modifiers())),
                        ]
                    ))
                )
            ),
            (
                StandardTextModifier::Reference.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Reference)),
                    Box::new(ReferenceRule::new())
                )
            ),
            (
                StandardTextModifier::Cite.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardTextModifier::Cite)),
                    Box::new(HtmlCiteRule::new())
                )
            ),
        ]);

        let paragraph_rules: ParagraphModifierOrderedMap = ParagraphModifierOrderedMap::from([
            (
                StandardParagraphModifier::CodeBlock.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::CodeBlock)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::CodeBlock.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {
    
                                    let mut lang_class = String::from("language-markup");
                                    if let Some(lang) = captures.get(1) {
                                        lang_class = format!("language-{}", lang.as_str());
                                    }
    
                                    Ok(CompilableText::from(vec![
                                        CompilableTextPart::new_fixed(format!(
                                            r#"<pre{}><code class="{} code-block">{}</code></pre>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                            lang_class,
                                            text_utility::replace(captures.get(2).unwrap().as_str(), &ESCAPE_HTML),
                                        )
                                    )
                                    ]))
                                }))),
                            ]
                        )
                    )) as Box<dyn ParagraphLoadingRule>
                ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
            ),
            (
                StandardParagraphModifier::MathBlock.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::MathBlock)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::MathBlock.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(vec![
                                        CompilableTextPart::new_fixed(format!(
                                            r#"<p class="math-block"{}>$${}$$</p>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                            captures.get(1).unwrap().as_str(),
                                        )
                                    )
                                    ]))
                                }))),
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::EmbeddedParagraphStyle.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::EmbeddedParagraphStyle)) as Box<dyn Modifier>,
                    Box::new(MetadataWrapperParagraphLoadingRule::new(
                        StandardParagraphModifier::EmbeddedParagraphStyle.modifier_pattern_regex().clone(),
                        1,
                        Some(2),
                        Some(3),
                        Some(Arc::new(|style, there_is_id| {
    
                            if there_is_id {
    
                                text_utility::split_styles_and_classes_with_default(style, (None, Some(String::from("identifier embedded-paragraph-style"))))
                            
                            } else {
    
                                text_utility::split_styles_and_classes_with_default(style, (None, Some(String::from("embedded-paragraph-style"))))
                            }
    
                        })),
                    ))
                )
            ),
            (
                StandardParagraphModifier::ParagraphIdentifier.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::ParagraphIdentifier)) as Box<dyn Modifier>,
                    Box::new(MetadataWrapperParagraphLoadingRule::new(
                        StandardParagraphModifier::ParagraphIdentifier.modifier_pattern_regex().clone(),
                        1,
                        Some(2),
                        None,
                        None,
                    ))
                )
            ),
            (
                    StandardParagraphModifier::Table.identifier(),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::Table)) as Box<dyn Modifier>,
                        Box::new(TableParagraphLoadingRule::new()) as Box<dyn ParagraphLoadingRule>
                    ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
            ),
            (
                StandardParagraphModifier::ExtendedBlockQuote.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::ExtendedBlockQuote)) as Box<dyn Modifier>,
                    Box::new(BlockQuoteParagraphLoadingRule::new()),
                )
            ),
            (
                StandardParagraphModifier::FocusBlock.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::FocusBlock)) as Box<dyn Modifier>,
                    Box::new(FocusBlockParagraphLoadingRule::new(StandardParagraphModifier::FocusBlock.modifier_pattern_regex().clone())),
                )
            ),
            (
                StandardParagraphModifier::List.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::List)) as Box<dyn Modifier>,
                    Box::new(ListParagraphLoadingRule::new()),
                )
            ),
            (
                StandardParagraphModifier::AbridgedTodo.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::AbridgedTodo)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::AbridgedTodo.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(CompilableTextPart::new_fixed(format!(
                                            r#"<div class="todo abridged-todo"{}><div class="todo-title"></div></div>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        ))
                                    ))
                                }))),
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::MultilineTodo.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::MultilineTodo)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::MultilineTodo.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(CompilableTextPart::new_fixed(format!(
                                            r#"<div class="todo multiline-todo"{}><div class="todo-title"></div><div class="todo-description">"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        ))
                                    ))
                                }))),
                                Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::MultilineTodo.incompatible_modifiers())),
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::Todo.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::Todo)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::Todo.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(CompilableTextPart::new_fixed(format!(
                                            r#"<div class="todo"{}><div class="todo-title"></div>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        ))
                                    ))
                                }))),
                                Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), StandardParagraphModifier::Todo.incompatible_modifiers())),
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"</div>"#)))
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::MultiImage.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::MultiImage)) as Box<dyn Modifier>,
                    Box::new(ImageParagraphLoadingRule::MultiImage)
                )
            ),
            (
                StandardParagraphModifier::Image.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::Image)) as Box<dyn Modifier>,
                    Box::new(ImageParagraphLoadingRule::SingleImage)
                )
            ),
            (
                StandardParagraphModifier::AbridgedImage.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::AbridgedImage)) as Box<dyn Modifier>,
                    Box::new(ImageParagraphLoadingRule::AbridgedImage)
                )
            ),
            (
                    StandardParagraphModifier::PageBreak.identifier().clone(),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::PageBreak)) as Box<dyn Modifier>,
                        Box::new(ReplacementRuleParagraphLoadingRule::new(
                            ReplacementRule::new(
                                StandardParagraphModifier::PageBreak.modifier_pattern().clone(),
                                vec![
                                    Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<div class="page-break"></div>"#)))
                                ]
                            )
                        ))
                    )
            ),
            (
                StandardParagraphModifier::LineBreakDash.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::LineBreakDash)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::LineBreakDash.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(vec![
                                        CompilableTextPart::new_fixed(format!(
                                            r#"<hr class="line-break line-break-dash"{}>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        )
                                    )
                                    ]))
                                }))),
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::LineBreakStar.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::LineBreakStar)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::LineBreakDash.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(vec![
                                        CompilableTextPart::new_fixed(format!(
                                            r#"<hr class="line-break line-break-star"{}>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        )
                                    )
                                    ]))
                                }))),
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::LineBreakPlus.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::LineBreakPlus)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::LineBreakDash.modifier_pattern().clone(),
                            vec![
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|_, compilable, _, _, _| {
    
                                    Ok(CompilableText::from(vec![
                                        CompilableTextPart::new_fixed(format!(
                                            r#"<hr class="line-break line-break-plus"{}>"#,
                                            text_utility::html_nuid_tag_or_nothing(compilable.nuid().as_ref()),
                                        )
                                    )
                                    ]))
                                }))),
                            ]
                        )
                    ))
                )
            ),
            (
                StandardParagraphModifier::CommentBlock.identifier().clone(),
                (
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::CommentBlock)) as Box<dyn Modifier>,
                    Box::new(ReplacementRuleParagraphLoadingRule::new(
                        ReplacementRule::new(
                            StandardParagraphModifier::CommentBlock.modifier_pattern().clone(),
                            vec![
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"<!--"#))),
                                Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, Vec::new(), StandardParagraphModifier::CommentBlock.incompatible_modifiers())),
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from(r#"-->"#))),
                            ]
                        )
                    ))
                )
            ),
        ]);

        Self::new(
            text_rules,
            paragraph_rules,
            Some(
                (
                    StandardParagraphModifier::CommonParagraph.identifier().clone(),
                    Box::new(CommonParagraphLoadingRule::new())
                ),
            ),
            Box::new(HtmlAssembler::new())
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
            IndexMap::new(),
            IndexMap::from([
                (
                    String::from("a"),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>,
                        Box::new(ReplacementRuleParagraphLoadingRule::new(
                            ReplacementRule::new(
                                String::from("fake"),
                                vec![
                                ]
                            )
                        )) as Box<dyn ParagraphLoadingRule>
                    ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
                ),
                (
                    String::from("b"),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>,
                        Box::new(ReplacementRuleParagraphLoadingRule::new(
                            ReplacementRule::new(
                                String::from("fake"),
                                vec![
                                ]
                            )
                        )) as Box<dyn ParagraphLoadingRule>
                    ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
                ),
                (
                    String::from("e"),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>,
                        Box::new(ReplacementRuleParagraphLoadingRule::new(
                            ReplacementRule::new(
                                String::from("fake"),
                                vec![
                                ]
                            )
                        )) as Box<dyn ParagraphLoadingRule>
                    ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
                ),
                (
                    String::from("c"),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>,
                        Box::new(ReplacementRuleParagraphLoadingRule::new(
                            ReplacementRule::new(
                                String::from("fake"),
                                vec![
                                ]
                            )
                        )) as Box<dyn ParagraphLoadingRule>
                    ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
                ),
                (
                    String::from("i"),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>,
                        Box::new(ReplacementRuleParagraphLoadingRule::new(
                            ReplacementRule::new(
                                String::from("fake"),
                                vec![
                                ]
                            )
                        )) as Box<dyn ParagraphLoadingRule>
                    ) as (Box<dyn Modifier>, Box<dyn ParagraphLoadingRule>)
                ),
            ]),
            None,
            Box::new(HtmlAssembler::new())
        );


        let ids: Vec<String> = codex.paragraph_modifiers.into_iter().map(|tm| tm.0).collect();

        assert_eq!(ids.join(""), "abeci");
    }
}