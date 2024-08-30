pub mod replacement_rule_part;


use std::fmt::Debug;
use std::sync::Arc;
use getset::{Getters, Setters};
use log;
use regex::Regex;
use replacement_rule_part::ReplacementRuleReplacerPart;
use crate::compilable_text::CompilableText;
use crate::compiler::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compiler::compilation_configuration::CompilationConfiguration;
use crate::output_format::OutputFormat;
use super::CompilationRule;
use crate::compiler::compilation_error::CompilationError;


pub type ReplacementRuleParts = Vec<Arc<dyn ReplacementRuleReplacerPart>>;


/// Rule to replace a NMD text based on a specific pattern matching rule
#[derive(Debug, Clone, Getters, Setters)]
pub struct ReplacementRule {

    #[getset(set)]
    search_pattern: String,

    #[getset(set)]
    search_pattern_regex: Regex,

    #[getset(get = "pub", set = "pub")]
    replacer_parts: ReplacementRuleParts,
}

impl ReplacementRule {
    
    /// Returns a new instance having a search pattern and a replication pattern
    pub fn new(searching_pattern: String, replacers: ReplacementRuleParts) -> Self {

        log::debug!("created new compilation rule with search_pattern: '{}'", searching_pattern);

        Self {
            search_pattern_regex: Regex::new(&searching_pattern).unwrap(),
            search_pattern: searching_pattern,
            replacer_parts: replacers,
        }
    }

}

impl CompilationRule for ReplacementRule {

    /// Compile the content using internal search and replacement pattern
    fn standard_compile(&self, compilable: &CompilableText, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {

        log::debug!("compile:\n{:#?}\nusing '{}'->'{:?}'", compilable, self.search_pattern(), self.replacer_parts);

        let mut compiled_parts = Vec::new();

        let compilable_content = compilable.compilable_content();

        let captures_matches = self.search_pattern_regex.captures_iter(&compilable_content);

        for captures in captures_matches {

            for replacer_part in &self.replacer_parts {

                compiled_parts.append(&mut replacer_part.compile(&captures, compilable, format, compilation_configuration, compilation_configuration_overlay.clone())?.into())
            }   
        }

        Ok(CompilableText::new(compiled_parts))
    }
    
    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}



#[cfg(test)]
mod test {

    use std::sync::Arc;

    use crate::{codex::modifier::{standard_text_modifier::StandardTextModifier, ModifiersBucket}, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::{constants::ESCAPE_HTML, replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart, single_capture_group_replacement_rule_part::SingleCaptureGroupReplacementRuleReplacerPart, ReplacementRuleReplacerPart}, ReplacementRule}, CompilationRule}}, output_format::OutputFormat};


    #[test]
    fn bold_compiling() {

        // valid pattern with a valid text modifier
        let replacement_rule = ReplacementRule::new(StandardTextModifier::BoldStarVersion.modifier_pattern(), vec![
            Arc::new(FixedReplacementRuleReplacerPart::new(String::from("<strong>"))) as Arc<dyn ReplacementRuleReplacerPart>,
            Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {
                
                let capture1 = captures.get(1).unwrap();
                
                let slice = compilable.parts_slice(capture1.start(), capture1.end())?;

                Ok(CompilableText::new(slice))
            }))),
            Arc::new(FixedReplacementRuleReplacerPart::new(String::from("</strong>"))),
        ]);

        let text_to_compile = r"A piece of **bold text** and **bold text2**";
        let compilation_configuration = CompilationConfiguration::default();

        let compilable = CompilableText::new(
            vec![
                CompilableTextPart::new(
                    text_to_compile.to_string(),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                )
        ]);
        
        let outcome = replacement_rule.compile(&compilable, &OutputFormat::Html, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();

        assert_eq!(outcome.content(), r"<strong>bold text</strong><strong>bold text2</strong>");

        // without text modifier
        let text_to_compile = r"A piece of text without bold text";

        let compilable = CompilableText::new(
            vec![
                CompilableTextPart::new(
                    text_to_compile.to_string(),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                )
        ]);

        let outcome = replacement_rule.compile(&compilable, &OutputFormat::Html, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();

        assert_eq!(outcome.content(), r"");


    }

    #[test]
    fn input_with_fixed_parts() {
        let replacement_rule = ReplacementRule::new(StandardTextModifier::ItalicStarVersion.modifier_pattern(), vec![
            Arc::new(FixedReplacementRuleReplacerPart::new(String::from("<em>"))) as Arc<dyn ReplacementRuleReplacerPart>,
            Arc::new(SingleCaptureGroupReplacementRuleReplacerPart::new(1, ESCAPE_HTML.clone(), ModifiersBucket::None)),
            Arc::new(FixedReplacementRuleReplacerPart::new(String::from("</em>"))),
        ]);

        let compilation_configuration = CompilationConfiguration::default();

        // ==== case 1 ====
        let compilable = CompilableText::new(
            vec![
                CompilableTextPart::new(
                    String::from("*start "),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                ),
                CompilableTextPart::new_fixed(String::from("<strong>")),
                CompilableTextPart::new_compilable(String::from("fixed"), ModifiersBucket::None),
                CompilableTextPart::new_fixed(String::from("</strong>")),
                CompilableTextPart::new(
                    String::from(" end*"),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                ),
        ]);
        
        let outcome = replacement_rule.compile(&compilable, &OutputFormat::Html, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();

        assert_eq!(outcome.content(), r"<em>start <strong>fixed</strong> end</em>");


        // ==== case 2 ====
        let compilable = CompilableText::new(
            vec![
                CompilableTextPart::new(
                    String::from("*start "),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                ),
                CompilableTextPart::new_fixed(String::from("<strong>")),
                CompilableTextPart::new_compilable(String::from("fixed"), ModifiersBucket::None),
                CompilableTextPart::new_fixed(String::from("</strong>")),
                CompilableTextPart::new(
                    String::from("*"),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                ),
        ]);
        
        let outcome = replacement_rule.compile(&compilable, &OutputFormat::Html, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();

        assert_eq!(outcome.content(), r"<em>start <strong>fixed</strong></em>");


        // ==== case 3 ====
        let compilable = CompilableText::new(
            vec![
                CompilableTextPart::new(
                    String::from("*"),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                ),
                CompilableTextPart::new_fixed(String::from("<strong>")),
                CompilableTextPart::new_compilable(String::from("fixed"), ModifiersBucket::None),
                CompilableTextPart::new_fixed(String::from("</strong>")),
                CompilableTextPart::new(
                    String::from(" end*"),
                    CompilableTextPartType::Compilable { incompatible_modifiers: ModifiersBucket::None }
                ),
        ]);
        
        let outcome = replacement_rule.compile(&compilable, &OutputFormat::Html, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();

        assert_eq!(outcome.content(), r"<em><strong>fixed</strong> end</em>");

    }

    // #[test]
    // fn heading_parsing() {

    //     let codex = Codex::of_html(CodexConfiguration::default());

    //     let compilation_configuration = CompilationConfiguration::default();

    //     let parsing_rule = ReplacementRule::new(StandardHeading::HeadingGeneralExtendedVersion(6).modifier_pattern().clone(), vec![
    //         ReplacementRuleReplacerPart::new_fixed(String::from("<h6>")),
    //         ReplacementRuleReplacerPart::new_mutable(String::from("$1")),
    //         ReplacementRuleReplacerPart::new_fixed(String::from("</h6>")),
    //     ]);

    //     let text_to_parse = r"###### title 6";

    //     let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(text_to_parse.to_string()));

    //     let parsed_text = parsing_rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

    //     assert_eq!(parsed_text.content(), r"<h6>title 6</h6>");
    // }

    // #[test]
    // fn code_block() {

    //     let codex = Codex::of_html(CodexConfiguration::default());

    //     let compilation_configuration = CompilationConfiguration::default();

    //     let parsing_rule = ReplacementRule::new(StandardParagraphModifier::CodeBlock.modifier_pattern_with_paragraph_separator().clone(), vec![
    //         ReplacementRuleReplacerPart::new_fixed(String::from(r#"<pre><code class="language-$1 codeblock">"#)),
    //         ReplacementRuleReplacerPart::new_mutable(String::from("$2")),
    //         ReplacementRuleReplacerPart::new_fixed(String::from("</code></pre>")),
    //     ]);

    //     let text_to_parse = concat!(
    //         "\n\n",
    //         "```python\n\n",
    //         r#"print("hello world")"#,
    //         "\n\n```\n\n"
    //     );
        
    //     let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(text_to_parse.to_string()));

    //     let parsed_text = parsing_rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

    //     assert_eq!(parsed_text.content(), "<pre><code class=\"language-python codeblock\">print(\"hello world\")</code></pre>");
    // }

    
}