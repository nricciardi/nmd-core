use getset::{Getters, Setters};
use crate::{codex::Codex, compilable_text::CompilableText, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_rule::{replacement_rule::ReplacementRule, CompilationRule}, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile, Compiler}, dossier::document::chapter::paragraph::Paragraph, output_format::OutputFormat, utility::nmd_unique_identifier::NmdUniqueIdentifier};



/// This paragraph uses a `ReplacementRule` to pre-compile the inner-text, after that, it will compile
/// compilable parts using `Compiler` and `Codex`
#[derive(Debug, Getters, Setters)]
pub struct ReplacementRuleParagraph {

    #[getset(set = "pub")]
    raw_content: String,

    replacement_rule: ReplacementRule,

    #[getset(set = "pub")]
    compiled_content: Option<CompilableText>,

    compilable_text: CompilableText,

}

impl ReplacementRuleParagraph {

    pub fn new(raw_content: String, compilable_text: CompilableText, replacement_rule: ReplacementRule,) -> Self {
        Self {
            raw_content,
            replacement_rule,
            compiled_content: None,
            compilable_text
        }
    }

}

impl SelfCompile for ReplacementRuleParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        let mut outcome = self.replacement_rule.compile(&self.compilable_text, format, compilation_configuration, compilation_configuration_overlay.clone())?;
        
        Compiler::compile_compilable_text(&mut outcome, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

        self.compiled_content = Some(outcome);

        Ok(())
    }
}


impl CompiledTextAccessor for ReplacementRuleParagraph {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compiled_content.as_ref()
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

    use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_paragraph_modifier::StandardParagraphModifier, standard_text_modifier::StandardTextModifier, Modifier, ModifiersBucket}, Codex, CodexCompilationRulesMap, CodexLoadingRulesMap, CodexModifiersMap}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::{replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart, single_capture_group_replacement_rule_part::SingleCaptureGroupReplacementRuleReplacerPart}, ReplacementRule}, CompilationRule}, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile}, output_format::OutputFormat};

    use super::ReplacementRuleParagraph;


    #[test]
    fn paragraph_with_nuid() {

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
            StandardParagraphModifier::CommonParagraph.modifier_pattern_with_paragraph_separator(),
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

        let codex = Codex::new(
            CodexModifiersMap::from([
                (
                    StandardTextModifier::BoldStarVersion.identifier(),
                    Box::new(
                        Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)
                    ) as Box<dyn Modifier>
                )
            ]),
            CodexModifiersMap::new(),
            CodexCompilationRulesMap::from([
                (
                    StandardTextModifier::BoldStarVersion.identifier(),
                    Box::new(
                        ReplacementRule::new(
                            StandardTextModifier::BoldStarVersion.modifier_pattern(),
                            vec![
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from("<strong>"))),
                                Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, Vec::new(), StandardTextModifier::BoldStarVersion.incompatible_modifiers())),
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from("</strong>"))),
                            ]
                        )
                    ) as Box<dyn CompilationRule>
                )
            ]),
            CodexLoadingRulesMap::new(),
        );

        paragraph.compile(
            &OutputFormat::Html,
            &codex,
            &CompilationConfiguration::default(),
            CompilationConfigurationOverLay::default()
        ).unwrap();


        assert_eq!(
            paragraph.compiled_text().unwrap().content(),
            r#"<p data-nuid="nuid-test">This is a <strong>common paragraph</strong></p>"#
        )

    }

}