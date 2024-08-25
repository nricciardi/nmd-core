use std::fmt::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use log;
use regex::{Captures, Regex, Replacer};
use crate::compiler::compilable::Compilable;
use crate::compiler::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compiler::compilation_configuration::CompilationConfiguration;
use crate::compiler::compilation_error::CompilationError;
use crate::compiler::compilation_result::{CompilationResult, CompilationResultPart, CompilationResultPartType, CompilationResultParts};
use crate::compiler::compilation_rule::constants::DOUBLE_NEW_LINE_REGEX;
use crate::output_format::OutputFormat;
use crate::resource::resource_reference::ResourceReference;
use crate::utility::text_utility;
use super::{CompilationRule, CompilationRuleResult};


type Closure = dyn Fn(Captures, CompilationResultParts, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> CompilationResultParts;

pub trait ReplacementRuleReplacerPart: Debug + Sync + Send {

    fn compile(&self, captures: Captures, compilable: CompilationResultParts, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> CompilationResultParts;
}

pub struct ClosureReplacementRuleReplacerPart<F>
where F: Fn(Captures, CompilationResultParts, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> CompilationResultParts {

    closure: F,
}

impl<F> ClosureReplacementRuleReplacerPart<F>
where F: Fn(Captures, CompilationResultParts, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> CompilationResultParts {

    pub fn new(closure: F) -> Self {
        Self {
            closure
        }
    }

}

impl<F> Debug for ClosureReplacementRuleReplacerPart<F>
where F: Fn(Captures, CompilationResultParts, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> CompilationResultParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureReplacementRuleReplacerPart").field("closure", &self.closure).finish()
    }
}

impl<F> ReplacementRuleReplacerPart for ClosureReplacementRuleReplacerPart<F>
where F: Sync + Send + Fn(Captures, CompilationResultParts, &OutputFormat, &CompilationConfiguration, CompilationConfigurationOverLay) -> CompilationResultParts {

    fn compile(&self, captures: Captures, compilable: CompilationResultParts, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> CompilationResultParts {
        todo!()
    }
}


/// Rule to replace a NMD text based on a specific pattern matching rule
#[derive(Debug, Getters, Setters, Clone)]
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
    fn standard_compile(&self, compilable: &Compilable, format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> CompilationRuleResult {

        log::debug!("compile:\n{:#?}\nusing '{}'->'{:?}'", compilable, self.search_pattern(), self.replacer_parts);

        let mut compiled_parts = CompilationResultParts::new();

        for captures in self.search_pattern_regex.captures_iter(&compilable.compilable_content()) {

            for replacer_part in self.replacer_parts {

                let m = captures.get(0).unwrap();

                let parts_to_compile = compilable.parts_slice(m.start(), m.end());

                compiled_parts.append(&mut replacer_part.compile(captures, parts_to_compile, format, compilation_configuration, compilation_configuration_overlay.clone()))
            }   
        }

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

    // use crate::{codex::{codex_configuration::CodexConfiguration, modifier::{standard_heading_modifier::StandardHeading, standard_paragraph_modifier::StandardParagraphModifier, standard_text_modifier::StandardTextModifier}}, compiler::compilable::GenericCompilable};

    // use super::*;

    // #[test]
    // fn bold_parsing() {

    //     let codex = Codex::of_html(CodexConfiguration::default());

    //     // valid pattern with a valid text modifier
    //     let parsing_rule = ReplacementRule::new(StandardTextModifier::BoldStarVersion.modifier_pattern().clone(), vec![
    //         ReplacementRuleReplacerPart::new_fixed(String::from("<strong>")),
    //         ReplacementRuleReplacerPart::new_mutable(String::from("$1")),
    //         ReplacementRuleReplacerPart::new_fixed(String::from("</strong>")),
    //     ]);

    //     let text_to_parse = r"A piece of **bold text** and **bold text2**";
    //     let compilation_configuration = CompilationConfiguration::default();

    //     let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(text_to_parse.to_string()));
        
    //     let parsed_text = parsing_rule.compile(&compilable, &OutputFormat::Html,&codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

    //     assert_eq!(parsed_text.content(), r"A piece of <strong>bold text</strong> and <strong>bold text2</strong>");

    //     // without text modifier
    //     let text_to_parse = r"A piece of text without bold text";

    //     let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(text_to_parse.to_string()));

    //     let parsed_text = parsing_rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

    //     assert_eq!(parsed_text.content(), r"A piece of text without bold text");


    // }

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