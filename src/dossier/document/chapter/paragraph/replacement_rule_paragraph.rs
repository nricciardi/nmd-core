use getset::{Getters, Setters};
use crate::{codex::Codex, compilable_text::CompilableText, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome, compilation_rule::{replacement_rule::ReplacementRule, CompilationRule}}, dossier::document::chapter::paragraph::Paragraph, output_format::OutputFormat, utility::nmd_unique_identifier::NmdUniqueIdentifier};



/// This paragraph uses a `ReplacementRule` to pre-compile the inner-text, after that, it will compile
/// compilable parts using `Compiler` and `Codex`
#[derive(Debug, Getters, Setters)]
pub struct ReplacementRuleParagraph {

    #[getset(set = "pub")]
    raw_content: String,

    #[getset(get = "pub", set = "pub")]
    replacement_rule: ReplacementRule,

    compilable_text: CompilableText,

}

impl ReplacementRuleParagraph {

    pub fn new(raw_content: String, compilable_text: CompilableText, replacement_rule: ReplacementRule,) -> Self {
        Self {
            raw_content,
            replacement_rule,
            compilable_text
        }
    }

}

impl Compilable for ReplacementRuleParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let mut outcome = self.replacement_rule.compile(&self.compilable_text, format, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        outcome.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
    }
}

impl Paragraph for ReplacementRuleParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.compilable_text.nuid().as_ref()
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.compilable_text.set_nuid(nuid);
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{codex::{modifier::{standard_paragraph_modifier::StandardParagraphModifier, ModifiersBucket}, Codex}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart, single_capture_group_replacement_rule_part::SingleCaptureGroupReplacementRuleReplacerPart}, ReplacementRule}}, content_bundle::ContentBundle, load::{LoadConfiguration, LoadConfigurationOverLay}, load_block::{LoadBlock, LoadBlockContent}, output_format::OutputFormat};

    use super::ReplacementRuleParagraph;


    fn load_and_compile_html(content: &str, expected_n: usize) -> String {
        
        let codex = Codex::of_html();
    
        let blocks = LoadBlock::load_from_str(content, &codex, &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();

        let mut bundle = ContentBundle::from(blocks);

        assert_eq!(bundle.preamble().len(), expected_n);

        let mut compiled_content = String::new();

        let cc = CompilationConfiguration::default();
        let cco = CompilationConfigurationOverLay::default();

        for paragraph in bundle.preamble_mut() {

            let outcome = paragraph.compile(&OutputFormat::Html, &codex, &cc, cco.clone()).unwrap();

            compiled_content.push_str(&outcome.content());
        }

        compiled_content
    }

    #[test]
    fn abridged_toto_load_and_compile() {

        let nmd_text = concat!(   "\n\n",
                                        "TODO\n\n",
                                    );

        let compiled_content = load_and_compile_html(nmd_text, 1);

        assert_eq!(compiled_content, r#"<div class="todo abridged-todo"><div class="todo-title"></div></div>"#);
    }


    #[test]
    fn common_paragraph_load_and_compile() {

        let nmd_text = concat!(  "\n\n",
                                            "p1\n\n\n",
                                            "p2\n\n\n",
                                            "p3a\np3b\np3c\n\n"
                                        );

        let compiled_content = load_and_compile_html(nmd_text, 3);

        assert_eq!(compiled_content, concat!(
            r#"<p class="paragraph">p1</p><p class="paragraph">p2</p><p class="paragraph">"#,
            "p3a\np3b\np3c",
            r#"</p>"#
        ));
    }

    #[test]
    fn paragraph_with_nuid() {

        let nmd_text = "\n\nThis is a **common paragraph**\n\n";

        let codex = Codex::of_html();

        let mut paragraphs = LoadBlock::load_from_str(
            &nmd_text,
            &codex,
            &LoadConfiguration::default(),
            LoadConfigurationOverLay::default(),
        ).unwrap();

        assert_eq!(paragraphs.len(), 1);

        let paragraph = &mut paragraphs[0];
        
        if let LoadBlockContent::Paragraph(ref mut paragraph) = paragraph.content_mut() {
            paragraph.set_nuid(Some(String::from("nuid-test")));

            paragraph.compile(
                &OutputFormat::Html,
                &codex,
                &CompilationConfiguration::default(),
                CompilationConfigurationOverLay::default()
            ).unwrap();

            assert_eq!(
                paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap().content(),
                r#"<p class="paragraph" data-nuid="nuid-test">This is a <strong class="bold">common paragraph</strong></p>"#
            )
        }

        panic!("paragraph not loaded");

    }

    #[test]
    fn paragraph_with_nuid_and_simple_codex() {

        let nmd_text = "\n\nThis is a **common paragraph**\n\n";

        let compilable_text = CompilableText::new_with_nuid(
            vec![
                CompilableTextPart::new_compilable(
                    nmd_text.to_string(),
                    ModifiersBucket::None
                )
            ],
            Some(String::from("nuid-test"))
        );

        let replacement_rule = ReplacementRule::new(
            StandardParagraphModifier::CommonParagraph.modifier_pattern(),
            vec![
                Arc::new(ClosureReplacementRuleReplacerPart::new(
                    Arc::new(
                        |_, compilable_text, _, _, _ | {

                            Ok(CompilableText::from(
                                CompilableTextPart::new_fixed(format!(r#"<p data-nuid="{}">"#, compilable_text.nuid().as_ref().unwrap()))
                            ))
                        }
                    )
                )),
                Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::from(1)
                            .with_incompatible_modifiers(StandardParagraphModifier::CommonParagraph.incompatible_modifiers())),
                Arc::new(FixedReplacementRuleReplacerPart::new(String::from("</p>")))
            ]
        );

        let mut paragraph = ReplacementRuleParagraph::new(
            nmd_text.to_string(),
            compilable_text,
            replacement_rule,
        );

        let codex = Codex::of_html();

        paragraph.compile(
            &OutputFormat::Html,
            &codex,
            &CompilationConfiguration::default(),
            CompilationConfigurationOverLay::default()
        ).unwrap();


        assert_eq!(
            paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap().content(),
            r#"<p data-nuid="nuid-test">This is a <strong>common paragraph</strong></p>"#
        )

    }

}