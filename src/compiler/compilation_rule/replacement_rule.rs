use std::fmt::Debug;
use getset::{Getters, Setters};
use log;
use regex::{Captures, Regex};
use crate::compilable_text::compilable_text_part::{CompilableTextPart, CompilableTextPartType};
use crate::compilable_text::CompilableText;
use crate::compiler::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compiler::compilation_configuration::CompilationConfiguration;
use crate::compiler::compilation_error::CompilationError;
use crate::output_format::OutputFormat;
use super::{CompilationRule, CompilationRuleResult};


type Closure = dyn Sync + Send + Fn(&Captures, &CompilableText, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError>;

pub trait ReplacementRuleReplacerPart: Debug + Sync + Send {

    fn compile(&self, captures: &Captures, compilable: &CompilableText, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError>;
}

pub struct ClosureReplacementRuleReplacerPart {

    closure: Box<Closure>,
}

impl ClosureReplacementRuleReplacerPart {

    pub fn new(closure: Box<Closure>) -> Self {
        Self {
            closure
        }
    }

}

impl Debug for ClosureReplacementRuleReplacerPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureReplacementRuleReplacerPart").finish()
    }
}

impl ReplacementRuleReplacerPart for ClosureReplacementRuleReplacerPart {
    fn compile(&self, captures: &Captures, compilable: &CompilableText, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        (self.closure)(captures, compilable, format, compilation_configuration, compilation_configuration_overlay.clone())
    }
}

#[derive(Debug, Getters, Setters)]
pub struct FixedReplacementRuleReplacerPart {

    #[getset(get = "pub", set = "pub")]
    content: String
}

impl FixedReplacementRuleReplacerPart {

    pub fn new(content: String) -> Self {
        Self {
            content
        }
    }

}

impl ReplacementRuleReplacerPart for FixedReplacementRuleReplacerPart {
    fn compile(&self, _captures: &Captures, _compilable: &CompilableText, _format: &OutputFormat, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {
        Ok(CompilableText::new(vec![
            CompilableTextPart::new(self.content.clone(), CompilableTextPartType::Fixed)
        ]))
    }
}




/// Rule to replace a NMD text based on a specific pattern matching rule
#[derive(Debug, Getters, Setters)]
pub struct ReplacementRule {

    #[getset(set)]
    search_pattern: String,

    #[getset(set)]
    search_pattern_regex: Regex,

    #[getset(get = "pub", set = "pub")]
    replacer_parts: Vec<Box<dyn ReplacementRuleReplacerPart>>,

    #[getset(get = "pub", set = "pub")]
    newline_fix_pattern: Option<String>,

    #[getset(get = "pub", set = "pub")]
    nuid_placeholder: String,
}

impl ReplacementRule {
    
    /// Returns a new instance having a search pattern and a replication pattern
    pub fn new(searching_pattern: String, replacers: Vec<Box<dyn ReplacementRuleReplacerPart>>) -> Self {

        log::debug!("created new compilation rule with search_pattern: '{}'", searching_pattern);

        Self {
            search_pattern_regex: Regex::new(&searching_pattern).unwrap(),
            search_pattern: searching_pattern,
            replacer_parts: replacers,
            newline_fix_pattern: None,
            nuid_placeholder: String::from("$nuid"),
        }
    }

    pub fn with_newline_fix(mut self, pattern: String) -> Self {
        self.newline_fix_pattern = Some(pattern);

        self
    }
}

impl CompilationRule for ReplacementRule {

    /// Compile the content using internal search and replacement pattern
    fn standard_compile(&self, compilable: &CompilableText, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> CompilationRuleResult {

        log::debug!("compile:\n{:#?}\nusing '{}'->'{:?}'", compilable, self.search_pattern(), self.replacer_parts);

        let mut compiled_parts = Vec::new();

        for captures in self.search_pattern_regex.captures_iter(&compilable.compilable_content()) {

            for replacer_part in &self.replacer_parts {

                compiled_parts.append(&mut replacer_part.compile(&captures, compilable, format, compilation_configuration, compilation_configuration_overlay.clone())?.into())
            }   
        }

        Ok(CompilableText::new(compiled_parts))

        // log::debug!("compile:\n{:#?}\nusing '{}'->'{:?}'", compilable, self.search_pattern(), self.replacer_parts);

        // let content = compilable.compilable_content();

        // let mut outcome = CompilationResult::new_empty();
        // let mut last_match = 0;

        // for captures in self.search_pattern_regex.captures_iter(content) {

        //     let mut replacers = self.replacer_parts.clone(); 

        //     // replace references
        //     for index in 0..self.replacer_parts.len() {

        //         for reference_at in self.replacer_parts[index].references_at() {

        //             let reference = captures.get(reference_at.clone()).unwrap().as_str();

        //             let reference = ResourceReference::of(reference, compilation_configuration_overlay.document_name().as_ref())?;
    
        //             let reference = reference.build();

        //             let r = replacers[index].replacer().replace(&format!("${}", reference_at), &reference);
        //             replacers[index].set_replacer(r);

        //             let r = replacers[index].replacer().replace(&format!("${{{}}}", reference_at), &reference);
        //             replacers[index].set_replacer(r);

        //             log::debug!("id: '{}', new replacer: {:?}", reference, replacers[index]);
        //         }

        //         if let Some(nuid) = compilable.nuid() {

        //             let r = replacers[index].replacer().replace(&self.nuid_placeholder, nuid);

        //             replacers[index].set_replacer(r);
        //         }
        //     }

        //     let matched_content = captures.get(0).unwrap();

        //     if last_match < matched_content.start() {
        //         outcome.add_compilable_part(content[last_match..matched_content.start()].to_string());
        //     }

        //     last_match = matched_content.end();

        //     for replacer in replacers {
        //         let compilation_result = self.search_pattern_regex.replace_all(matched_content.as_str(), replacer.replacer());

        //         let mut compilation_result = compilation_result.to_string();

        //         if let Some(post_replacing) = replacer.post_replacing() {
        //             compilation_result = text_utility::replace(&compilation_result, post_replacing);
        //         }
                
        //         if replacer.fixed {

        //             outcome.add_fixed_part(compilation_result);
    
        //         } else {
    
        //             outcome.add_compilable_part(compilation_result);
        //         }
        //     }   
        // }

        // if last_match < content.len() {
        //     outcome.add_compilable_part(content[last_match..content.len()].to_string());
        // }

        // if let Some(newline_fix_pattern) = self.newline_fix_pattern.as_ref() {

        //     for part in outcome.parts_mut().iter_mut() {
        //         let new_result = DOUBLE_NEW_LINE_REGEX.replace_all(&part.content(), newline_fix_pattern).to_string();
        
        //         match part {
        //             CompilationResultPart::Fixed { content: _ } => *part = CompilationResultPart::Fixed { content: new_result },
        //             CompilationResultPart::Compilable { content: _ } => *part = CompilationResultPart::Compilable { content: new_result },
        //         };
        //     }
        // }

        // log::debug!("result:\n{:?}", outcome);
        
        // Ok(outcome)
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

    use crate::{codex::modifier::{standard_text_modifier::StandardTextModifier, ModifiersBucket}, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::{replacement_rule::{ClosureReplacementRuleReplacerPart, FixedReplacementRuleReplacerPart, ReplacementRule, ReplacementRuleReplacerPart}, CompilationRule}}, output_format::OutputFormat};


    #[test]
    fn bold_compiling() {

        // valid pattern with a valid text modifier
        let replacement_rule = ReplacementRule::new(StandardTextModifier::BoldStarVersion.modifier_pattern(), vec![
            Box::new(FixedReplacementRuleReplacerPart::new(String::from("<strong>"))) as Box<dyn ReplacementRuleReplacerPart>,
            Box::new(ClosureReplacementRuleReplacerPart::new(Box::new(|captures, compilable, _, _, _| {
                
                let capture1 = captures.get(1).unwrap();
                
                let slice = compilable.parts_slice(capture1.start(), capture1.end())?;

                Ok(CompilableText::new(slice))
            }))),
            Box::new(FixedReplacementRuleReplacerPart::new(String::from("</strong>"))),
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
    // fn paragraph_parsing() {

    //     let codex = Codex::of_html(CodexConfiguration::default());

    //     let compilation_configuration = CompilationConfiguration::default();

    //     let parsing_rule = ReplacementRule::new(StandardParagraphModifier::CommonParagraph.modifier_pattern_with_paragraph_separator().clone(), vec![
    //         ReplacementRuleReplacerPart::new_fixed(String::from("<p>")),
    //         ReplacementRuleReplacerPart::new_mutable(String::from("$1")),
    //         ReplacementRuleReplacerPart::new_fixed(String::from("</p>")),
    //     ]);

    //     let text_to_parse = concat!(  "\n\n",
    //                                         "p1\n\n\n",
    //                                         "p2\n\n\n",
    //                                         "p3a\np3b\np3c\n\n"
    //                                     );

    //     let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(text_to_parse.to_string()));

    //     let parsed_text = parsing_rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

    //     assert_eq!(parsed_text.content(), "<p>p1</p><p>p2</p><p>p3a\np3b\np3c</p>");
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

    // #[test]
    // fn focus_block() {

    //     let codex = Codex::of_html(CodexConfiguration::default());
        
    //     let compilation_configuration = CompilationConfiguration::default();

    //     let parsing_rule = ReplacementRule::new(StandardParagraphModifier::FocusBlock.modifier_pattern_with_paragraph_separator().clone(), vec![
    //         ReplacementRuleReplacerPart::new_fixed(String::from(r#"<div class="focus-block focus-block-$1">$2</div>"#)),
    //         ReplacementRuleReplacerPart::new_mutable(String::from(r#"$2"#)),
    //         ReplacementRuleReplacerPart::new_fixed(String::from(r#"</div>"#)),
    //     ]).with_newline_fix(r"<br>".to_string());

    //     let text_to_parse = concat!(
    //         "# title 1",
    //         "::: warning\nnew warning\n\nmultiline\n:::",
    //         "\n",
    //     );
        
    //     let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(text_to_parse.to_string()));

    //     let parsed_text = parsing_rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();
    //     let parsed_text = parsed_text.content();

    //     assert_ne!(parsed_text, text_to_parse);
     
    // }
}