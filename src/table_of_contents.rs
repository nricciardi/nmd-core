pub mod content_tree;

use std::sync::{Arc, RwLock};
use getset::{CopyGetters, Getters, Setters};
use crate::compiler::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, Compiler};

use super::{codex::Codex, dossier::document::chapter::heading::Heading, output_format::OutputFormat};


pub const TOC_INDENTATION: &str = r#"<span class="toc-item-indentation"></span>"#;



#[derive(Debug, Clone, Getters, CopyGetters, Setters)]
pub struct TableOfContents {

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get_copy = "pub", set = "pub")]
    page_numbers: bool,

    #[getset(get_copy = "pub", set = "pub")]
    plain: bool,

    #[getset(get = "pub", set = "pub")]
    maximum_heading_level: usize,

    #[getset(get = "pub", set = "pub")]
    headings: Vec<Heading>,

    #[getset(get = "pub", set = "pub")]
    compilation_result: Option<CompilationResult>,
}

impl TableOfContents {
    pub fn new(title: String, page_numbers: bool, plain: bool, maximum_heading_level: usize, headings: Vec<Heading>) -> Self {
        Self {
            title,
            page_numbers,
            plain,
            maximum_heading_level,
            headings,
            compilation_result: None
        }
    }

    fn min_headers_lv(&self) -> Option<u32> {
        let mut m: Option<u32> = None;

        for h in &self.headings {
            
            if m.is_none() {
                m = Some(h.level());
                continue;
            }
            
            m = Some(m.unwrap().min(h.level()));
        }

        m
    }

    fn standard_html_compile(&mut self, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        let mut outcome = CompilationResult::new_empty();

        outcome.add_fixed_part(String::from(r#"<section class="toc">"#));
        outcome.add_fixed_part(String::from(r#"<div class="toc-title">"#));
        outcome.append_compilation_result(&mut Compiler::compile_str(&*codex, &self.title, Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))?);
        outcome.add_fixed_part(String::from(r#"</div>"#));
        outcome.add_fixed_part(String::from(r#"<ul class="toc-body">"#));

        let mut total_li = 0;

        for heading in &self.headings {

            let heading_lv: u32 = heading.level();

            if heading_lv > self.maximum_heading_level as u32 {
                continue;
            }

            outcome.add_fixed_part(String::from(r#"<li class="toc-item">"#));

            if !self.plain {

                let min_heading_lv = self.min_headers_lv();

                if let Some(m) = min_heading_lv {

                    outcome.add_fixed_part(TOC_INDENTATION.repeat((heading_lv - m) as usize));

                } else {
                    outcome.add_fixed_part(TOC_INDENTATION.repeat(heading_lv as usize));

                }
            }

            outcome.add_fixed_part(r#"<span class="toc-item-bullet">"#.to_string());
            outcome.add_fixed_part(r#"</span><span class="toc-item-content">"#.to_string());

            if let Some(id) = heading.resource_reference() {

                outcome.add_fixed_part(format!(r#"<a href="{}" class="link">"#, id.build()));
            
            } else {
                log::warn!("heading '{}' does not have a valid id", heading.title())
            }

            outcome.append_compilation_result(&mut Compiler::compile_str(&*codex, &heading.title(), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay))?);

            if let Some(_) = heading.resource_reference() {

                outcome.add_fixed_part(String::from(r#"</a>"#));
            }

            outcome.add_fixed_part(String::from(r#"</span></li>"#));

            total_li += 1;
                
        }

        outcome.add_fixed_part(String::from(r#"</ul>"#));
        outcome.add_fixed_part(String::from(r#"</section>"#));

        self.compilation_result = Some(outcome);

        log::info!("compiled table of contents ({} lines, {} skipped)", total_li, self.headings.len() - total_li);

        Ok(())
    }
}

impl Compilable for TableOfContents {
    fn standard_compile(&mut self, format: &OutputFormat, codex: Arc<Codex>, compilation_configuration: Arc<RwLock<CompilationConfiguration>>, compilation_configuration_overlay: Arc<Option<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        if self.headings.is_empty() {
            
            return Ok(());
        }

        if self.page_numbers {
            log::error!("table of contents with page numbers not already usable...");

            unimplemented!("table of contents with page numbers not already usable...");
        }
        
        match format {
            OutputFormat::Html => self.standard_html_compile(Arc::clone(&codex), Arc::clone(&compilation_configuration), Arc::clone(&compilation_configuration_overlay)),
        }
    }
}