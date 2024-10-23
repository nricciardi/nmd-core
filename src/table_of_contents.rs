pub mod content_tree;

use getset::{CopyGetters, Getters, Setters};
use serde::Serialize;
use crate::{codex::Codex, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile}, output_format::OutputFormat};
use super::dossier::document::chapter::heading::Heading;


pub const TOC_INDENTATION: &str = r#"<span class="toc-item-indentation"></span>"#;



#[derive(Debug, Clone, Getters, CopyGetters, Setters, Serialize)]
pub struct TableOfContents {

    #[getset(get = "pub", set = "pub")]
    title: String,

    #[getset(get_copy = "pub", set = "pub")]
    page_numbers: bool,

    #[getset(get_copy = "pub", set = "pub")]
    plain: bool,

    #[getset(get_copy = "pub", set = "pub")]
    maximum_heading_level: usize,

    #[getset(get = "pub", set = "pub")]
    headings: Vec<Heading>,

    #[getset(set = "pub")]
    compilation_result: Option<CompilableText>,
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

    /// Return minimum header level (if exists)
    fn min_headers_lv(headings: &Vec<Heading>) -> Option<u32> {
        let mut m: Option<u32> = None;

        for h in headings {
            
            if m.is_none() {
                m = Some(h.level());
                continue;
            }
            
            m = Some(m.unwrap().min(h.level()));
        }

        m
    }
}

impl CompiledTextAccessor for TableOfContents {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compilation_result.as_ref()
    }
}

impl SelfCompile for TableOfContents {

    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        if self.headings().is_empty() {
            
            return Ok(());
        }

        if self.page_numbers() {
            log::error!("table of contents with page numbers not already usable...");

            unimplemented!("table of contents with page numbers not already usable...");
        }
        
        match format {
            OutputFormat::Html => {
                let mut outcome = CompilableText::new_empty();

                let mut compiled_title = CompilableText::from(self.title.clone());

                compiled_title.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

                outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"<section class="toc"><div class="toc-title">"#)));
                outcome.parts_mut().append(compiled_title.parts_mut());
                outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</div><ul class="toc-body">"#)));

                let mut total_li = 0;

                for heading in self.headings() {

                    let heading_lv: u32 = heading.level();

                    if heading_lv > self.maximum_heading_level() as u32 {
                        continue;
                    }

                    outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"<li class="toc-item">"#)));

                    if !self.plain() {

                        let min_heading_lv = Self::min_headers_lv(self.headings());

                        if let Some(m) = min_heading_lv {

                            outcome.parts_mut().push(CompilableTextPart::new_fixed(TOC_INDENTATION.repeat((heading_lv - m) as usize)));

                        } else {
                            outcome.parts_mut().push(CompilableTextPart::new_fixed(TOC_INDENTATION.repeat(heading_lv as usize)));

                        }
                    }

                    outcome.parts_mut().push(CompilableTextPart::new_fixed(r#"<span class="toc-item-bullet"></span><span class="toc-item-content">"#.to_string()));

                    if let Some(id) = heading.resource_reference() {

                        outcome.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<a href="{}" class="link">"#, id.build())));
                    
                    } else {
                        log::warn!("heading '{}' does not have a valid id", heading.title())
                    }

                    let compilation_configuration_overlay = compilation_configuration_overlay.clone();

                    let mut compiled_heading_title = CompilableText::from(self.title.clone());
                    
                    compiled_heading_title.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

                    outcome.parts_mut().append(compiled_heading_title.parts_mut());

                    if let Some(_) = heading.resource_reference() {

                        outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</a>"#)));
                    }

                    outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</span></li>"#)));

                    total_li += 1;
                        
                }

                outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</ul></section>"#)));

                self.set_compilation_result(Some(outcome));

                log::info!("compiled table of contents ({} lines, {} skipped)", total_li, self.headings().len() - total_li);

                Ok(())
            },
        }
    }

}