use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{codex::{modifier::{constants::NEW_LINE, standard_paragraph_modifier::StandardParagraphModifier}, Codex}, compiler::{compilation_configuration::CompilationConfiguration, compilation_error::CompilationError, compilation_result::{CompilationResult, CompilationResultPart}}, utility::text_utility};

use super::{constants::{DOUBLE_NEW_LINE_REGEX, ESCAPE_HTML}, ParsingRule};

static CHECK_EXTENDED_BLOCK_QUOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^(?m:^> \[!(.*)\]))").unwrap());


#[derive(Debug)]
pub struct HtmlExtendedBlockQuoteRule {
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl HtmlExtendedBlockQuoteRule {
    pub fn new() -> Self {
        Self {
            search_pattern: StandardParagraphModifier::ExtendedBlockQuote.modifier_pattern_with_paragraph_separator(),
            search_pattern_regex: Regex::new(&StandardParagraphModifier::ExtendedBlockQuoteLine.modifier_pattern_with_paragraph_separator()).unwrap(),
        }
    }
}

impl ParsingRule for HtmlExtendedBlockQuoteRule {

    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }
    fn standard_parse(&self, content: &str, codex: &Codex, parsing_configuration: Arc<RwLock<CompilationConfiguration>>) -> Result<CompilationResult, CompilationError> {

        let content = content.trim();
        let mut lines: Vec<&str> = content.lines().collect();

        let there_is_quote_type = CHECK_EXTENDED_BLOCK_QUOTE_REGEX.is_match(content);
        let mut quote_type: String = String::from("quote");

        if there_is_quote_type {

            quote_type = CHECK_EXTENDED_BLOCK_QUOTE_REGEX.captures(content).unwrap().get(1).unwrap().as_str().to_string().to_lowercase();

            lines.remove(0);
        }

        let mut tag_body = String::new();

        for line in lines {
            if !line.starts_with(">") {
                if parsing_configuration.read().unwrap().strict_focus_block_check() {
                    log::warn!("invalid line in focus (quote) block: {}", line);
                    continue;
                } else {
                    log::error!("invalid line in focus (quote) block: {}", line);
                    panic!("invalid line in focus (quote) block");
                }
            }

            let mut c = String::from(line[1..].trim_start());

            if c.is_empty() {
                c = format!("{}{}", NEW_LINE, NEW_LINE);
            }

            tag_body.push_str(c.as_str());
        }

        let tag_body = text_utility::replace(&tag_body, &ESCAPE_HTML);
        let tag_body = DOUBLE_NEW_LINE_REGEX.replace_all(&tag_body, "<br>").to_string();

        let outcome = CompilationResult::new(vec![
            CompilationResultPart::Fixed { content: format!(r#"
                <div class="focus-quote-block focus-quote-block-{}">
                <div class="focus-quote-block-title focus-quote-block-{}-title"></div>
                <div class="focus-quote-block-description focus-quote-block-{}-description">"#, quote_type, quote_type, quote_type) },
            CompilationResultPart::Mutable { content: tag_body },
            CompilationResultPart::Fixed { content: String::from("</div></div>") }
        ]);

        Ok(outcome)
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }

}