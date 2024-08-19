//! `Compiler` permits to transform a dossier or document based on output format and codex. Compiled dossier or document
//! should be assembled using appropriate `Assembler`


pub mod compilation_rule;
pub mod compilation_error;
pub mod compilation_result;
pub mod compilation_configuration;
pub mod compilation_result_accessor;
pub mod compilable;
pub mod self_compile;


use std::{sync::{Arc, RwLock}, time::Instant};
use compilable::{Compilable, GenericCompilable};
use compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration};
use compilation_error::CompilationError;
use compilation_result::{CompilationResult, CompilationResultPart};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use crate::{bibliography::{Bibliography, BIBLIOGRAPHY_FICTITIOUS_DOCUMENT}, codex::modifier::ModifiersBucket, dossier::{document::{chapter::heading::Heading, Chapter}, Document, Dossier}, output_format::OutputFormat, resource::{bucket::Bucket, resource_reference::ResourceReference}, table_of_contents::{TableOfContents, TOC_INDENTATION}};
use super::codex::Codex;



#[derive(Debug)]
enum Segment {
    Match(String),
    NonMatch(String),
}


#[derive(Debug)]
pub struct Compiler {
}

impl Compiler {

    /// Compile a dossier
    pub fn compile_dossier(dossier: &mut Dossier, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        log::info!("compile dossier {} with ({} documents, parallelization: {})", dossier.name(), dossier.documents().len(), compilation_configuration.parallelization());

        compilation_configuration_overlay.write().unwrap().set_dossier_name(Some(dossier.name().clone()));

        let fast_draft = compilation_configuration.fast_draft();

        if compilation_configuration.parallelization() {

            let compilation_configuration_overlay = Arc::clone(&compilation_configuration_overlay);

            let maybe_fails = dossier.documents_mut().par_iter_mut()
                .filter(|document| {
                    if fast_draft {
    
                        if let Some(subset) = compilation_configuration_overlay.read().unwrap().compile_only_documents() {

                            let skip = !subset.contains(document.name());
        
                            if skip {
                                log::info!("document {} compilation is skipped", document.name());
                            }

                            return !skip;
                        }
                    }

                    true
                })
                .map(|document| {

                    let now = Instant::now();

                    let res = Self::compile_document(document, format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay));

                    log::info!("document '{}' compiled in {} ms", document.name(), now.elapsed().as_millis());

                    res
                })
                .find_any(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
            
        } else {
            let maybe_fails = dossier.documents_mut().iter_mut()
                .filter(|document| {

                    if fast_draft {

                        if let Some(subset) = compilation_configuration_overlay.read().unwrap().compile_only_documents() {

                            let skip = !subset.contains(document.name());
        
                            if skip {
                                log::info!("document {} compilation is skipped", document.name());
                            }

                            return !skip;
                        }
                    }

                    true
                })
                .map(|document| {
                    let now = Instant::now();

                    let res = Self::compile_document(document, format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay));

                    log::info!("document '{}' compiled in {} ms", document.name(), now.elapsed().as_millis());

                    res
                })
                .find(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
        }

        if dossier.configuration().table_of_contents_configuration().include_in_output() {

            log::info!("dossier table of contents will be included in output");

            let mut headings: Vec<Heading> = Vec::new();

            for document in dossier.documents() {
                for chapter in document.chapters() {
                    headings.push(chapter.heading().clone());
                }
            }

            let mut table_of_contents = TableOfContents::new(
                dossier.configuration().table_of_contents_configuration().title().clone(),
                dossier.configuration().table_of_contents_configuration().page_numbers(),
                dossier.configuration().table_of_contents_configuration().plain(),
                dossier.configuration().table_of_contents_configuration().maximum_heading_level(),
                headings
            );

            Self::compile_table_of_contents(&mut table_of_contents, format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))?;
        
            dossier.set_table_of_contents(Some(table_of_contents));
        }

        if dossier.configuration().bibliography().include_in_output() {
            let mut bibliography = Bibliography::new(
                dossier.configuration().bibliography().title().clone(),
                dossier.configuration().bibliography().records().clone()
            );

            Self::compile_bibliography(&mut bibliography, format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))?;
        
            dossier.set_bibliography(Some(bibliography));
        }

        Ok(())
    }

    /// Compile document
    pub fn compile_document(document: &mut Document, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        let parallelization = compilation_configuration.parallelization();

        log::info!("compile {} chapters of document: '{}'", document.chapters().len(), document.name());

        compilation_configuration_overlay.write().unwrap().set_document_name(Some(document.name().clone()));

        if parallelization {

            let maybe_one_failed: Option<Result<(), CompilationError>> = document.preamble_mut().par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }

            let maybe_one_failed: Option<Result<(), CompilationError>> = document.chapters_mut().par_iter_mut()
                .map(|chapter| {

                    Self::compile_chapter(chapter, format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        
        } else {

            let maybe_one_failed: Option<Result<(), CompilationError>> = document.preamble_mut().iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())

                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
            
            let maybe_one_failed: Option<Result<(), CompilationError>> = document.chapters_mut().iter_mut()
                .map(|chapter| {

                    Self::compile_chapter(chapter, format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))
                
                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        }

        Ok(())

    }

    /// Compile chapter
    pub fn compile_chapter(chapter: &mut Chapter, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        Self::compile_heading(chapter.heading_mut(), format, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))?;

        log::debug!("compile chapter:\n{:#?}", chapter);

        if compilation_configuration.parallelization() {

            let maybe_failed = chapter.paragraphs_mut().par_iter_mut()
                .map(|paragraph| {
                    
                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())

                })
                .find_any(|result| result.is_err());
    
            if let Some(result) = maybe_failed {
                return result
            }

        } else {

            let compilation_configuration_overlay = compilation_configuration_overlay.clone();
            
            let maybe_failed = chapter.paragraphs_mut().iter_mut()
                .map({
                    let compilation_configuration_overlay = compilation_configuration_overlay.clone();

                    move |paragraph| {
                        paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                    }
                })
                .find(|result| result.is_err());
    
            if let Some(result) = maybe_failed {
                return result
            }
        }

        Ok(())
    }

    /// Compile heading
    pub fn compile_heading(heading: &mut Heading, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {

        let cco = compilation_configuration_overlay.write().unwrap();
        let document_name = cco.document_name().as_ref();

        if document_name.is_none() {
            return Err(CompilationError::DocumentNameNotFound)
        }

        let document_name = document_name.unwrap();

        let id: ResourceReference = ResourceReference::of_internal_from_without_sharp(heading.title(), Some(&document_name))?;

        let compiled_title = Self::compile_str(heading.title(), format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

        let res = match format {
            OutputFormat::Html => {

                let nuid_attr: String;

                if let Some(nuid) = heading.nuid() {
                    nuid_attr = format!(r#"data-nuid="{}""#, nuid);
                } else {
                    nuid_attr = String::new();
                }

                let outcome = CompilationResult::new(vec![
                    CompilationResultPart::Fixed { content: format!(r#"<h{} class="heading-{}" id="{}" {}>"#, heading.level(), heading.level(), id.build_without_internal_sharp(), nuid_attr) },
                    CompilationResultPart::Mutable { content: compiled_title.content() },
                    CompilationResultPart::Fixed { content: format!(r#"</h{}>"#, heading.level()) },
                ]);

                outcome
            },
        };

        heading.set_compilation_result(Some(res));
        heading.set_resource_reference(Some(id));

        Ok(())
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

    /// Compile table of contents
    pub fn compile_table_of_contents(table_of_contents: &mut TableOfContents, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        if table_of_contents.headings().is_empty() {
            
            return Ok(());
        }

        if table_of_contents.page_numbers() {
            log::error!("table of contents with page numbers not already usable...");

            unimplemented!("table of contents with page numbers not already usable...");
        }
        
        match format {
            OutputFormat::Html => {
                let mut outcome = CompilationResult::new_empty();

                outcome.add_fixed_part(String::from(r#"<section class="toc">"#));
                outcome.add_fixed_part(String::from(r#"<div class="toc-title">"#));
                outcome.append_compilation_result(&mut Self::compile_str(table_of_contents.title(), format, codex, compilation_configuration, compilation_configuration_overlay.clone())?);
                outcome.add_fixed_part(String::from(r#"</div>"#));
                outcome.add_fixed_part(String::from(r#"<ul class="toc-body">"#));

                let mut total_li = 0;

                for heading in table_of_contents.headings() {

                    let heading_lv: u32 = heading.level();

                    if heading_lv > table_of_contents.maximum_heading_level() as u32 {
                        continue;
                    }

                    outcome.add_fixed_part(String::from(r#"<li class="toc-item">"#));

                    if !table_of_contents.plain() {

                        let min_heading_lv = Self::min_headers_lv(table_of_contents.headings());

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

                    let compilation_configuration_overlay = compilation_configuration_overlay.clone();

                    outcome.append_compilation_result(&mut Self::compile_str(
                                    &heading.title(),
                                    format,
                                    codex,
                                    compilation_configuration,
                                    Arc::clone(&compilation_configuration_overlay)
                                )?);

                    if let Some(_) = heading.resource_reference() {

                        outcome.add_fixed_part(String::from(r#"</a>"#));
                    }

                    outcome.add_fixed_part(String::from(r#"</span></li>"#));

                    total_li += 1;
                        
                }

                outcome.add_fixed_part(String::from(r#"</ul>"#));
                outcome.add_fixed_part(String::from(r#"</section>"#));

                table_of_contents.set_compilation_result(Some(outcome));

                log::info!("compiled table of contents ({} lines, {} skipped)", total_li, table_of_contents.headings().len() - total_li);

                Ok(())
            },
        }
    }

    /// Compile bibliography
    pub fn compile_bibliography(bibliography: &mut Bibliography, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        log::info!("compiling bibliography...");

        match format {
            OutputFormat::Html => {
                let mut compilation_result = CompilationResult::new_empty();

                compilation_result.add_fixed_part(String::from(r#"<section class="bibliography">"#));
                compilation_result.add_fixed_part(String::from(r#"<div class="bibliography-title">"#));
                compilation_result.append_compilation_result(&mut Self::compile_str(bibliography.title(), format, codex, compilation_configuration, compilation_configuration_overlay)?);
                compilation_result.add_fixed_part(String::from(r#"</div>"#));
                compilation_result.add_fixed_part(String::from(r#"<ul class="bibliography-body">"#));
        
                for (bib_key, bib_record) in bibliography.content().iter() {
                    compilation_result.add_fixed_part(format!(r#"<div class="bibliography-item" id="{}">"#, ResourceReference::of_internal_from_without_sharp(bib_key, Some(&BIBLIOGRAPHY_FICTITIOUS_DOCUMENT))?.build_without_internal_sharp()));
                    compilation_result.add_fixed_part(String::from(r#"<div class="bibliography-item-title">"#));
        
                    compilation_result.add_fixed_part(bib_record.title().to_string());
        
                    compilation_result.add_fixed_part(String::from(r#"</div>"#));
        
                    if let Some(authors) = bib_record.authors() {
        
                        compilation_result.add_fixed_part(String::from(r#"<div class="bibliography-item-authors">"#));
                        compilation_result.add_fixed_part(String::from(authors.join(", ")));
                        compilation_result.add_fixed_part(String::from(r#"</div>"#));
                    }
        
                    if let Some(year) = bib_record.year() {
        
                        compilation_result.add_fixed_part(String::from(r#"<div class="bibliography-item-year">"#));
                        compilation_result.add_fixed_part(String::from(year.to_string()));
                        compilation_result.add_fixed_part(String::from(r#"</div>"#));
                    }
        
                    if let Some(url) = bib_record.url() {
        
                        compilation_result.add_fixed_part(String::from(r#"<div class="bibliography-item-url">"#));
                        compilation_result.add_fixed_part(String::from(url.to_string()));
                        compilation_result.add_fixed_part(String::from(r#"</div>"#));
                    }
        
                    if let Some(description) = bib_record.description() {
        
                        compilation_result.add_fixed_part(String::from(r#"<div class="bibliography-item-description">"#));
                        compilation_result.add_fixed_part(String::from(description.to_string()));
                        compilation_result.add_fixed_part(String::from(r#"</div>"#));
                    }
        
                    compilation_result.add_fixed_part(String::from(r#"</div>"#));
                }
        
                compilation_result.add_fixed_part(String::from(r#"</ul>"#));
                compilation_result.add_fixed_part(String::from(r#"</section>"#));
        
                bibliography.set_compilation_result(Some(compilation_result));
        
                log::info!("bibliography compiled");
        
                Ok(())
            },
        }
    }

    /// Compile a string
    pub fn compile_str(content: &str, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        let excluded_modifiers = compilation_configuration_overlay.read().unwrap().excluded_modifiers().clone();

        log::debug!("start to compile content:\n{}\nexcluding: {:?}", content, excluded_modifiers);

        if excluded_modifiers == Bucket::All {
            log::debug!("compilation of content:\n{} is skipped because are excluded all modifiers", content);
            
            return Ok(CompilationResult::new_fixed(content.to_string()))
        }

        let mut content_parts: Vec<CompilationResultPart> = vec![CompilationResultPart::Mutable{ content: String::from(content) }];
        let mut content_parts_additional_excluded_modifiers: Vec<ModifiersBucket> = vec![ModifiersBucket::None];

        for (codex_identifier, text_modifier) in codex.text_modifiers() {

            assert_eq!(content_parts.len(), content_parts_additional_excluded_modifiers.len());

            if excluded_modifiers.contains(codex_identifier) {

                log::debug!("{:?} is skipped", text_modifier);
                continue;
            }

            let text_rule = codex.text_compilation_rules().get(codex_identifier);

            if text_rule.is_none() {
                log::warn!("text rule for {:#?} not found", text_modifier);
                continue;
            }

            let text_rule = text_rule.unwrap();

            let mut new_content_parts: Vec<CompilationResultPart> = Vec::new();
            let mut new_content_parts_additional_excluded_modifiers: Vec<ModifiersBucket> = Vec::new();
            
            for (content_part_index, content_part) in content_parts.iter().enumerate() {

                let current_excluded_modifiers = &content_parts_additional_excluded_modifiers[content_part_index];

                let mut no_match_fn = || {
                        
                    new_content_parts.push(content_part.clone());
                    new_content_parts_additional_excluded_modifiers.push(current_excluded_modifiers.clone());
                };

                if let CompilationResultPart::Fixed { content } = content_part {

                    log::debug!("{:?} is skipped for because '{}' fixed", text_modifier, content);

                    no_match_fn();
                    continue;
                }

                if current_excluded_modifiers.contains(codex_identifier) {
                    log::debug!("{:?} is skipped for '{}'", text_modifier, content_part.content());

                    no_match_fn();
                    continue;
                }

                let matches = text_rule.find_iter(&content_part.content());

                if matches.len() == 0 {
                    log::debug!("'{}' => no matches with {:?}", content_part.content(), text_rule);
                    
                    no_match_fn();
                    continue;
                }

                log::debug!("'{}' => there is a match with {:#?}", content_part.content(), text_rule);

                let mut last_end = 0;

                let mut elaborate_segment_fn = |segment: Segment| -> Result<(), CompilationError> {
                    match segment {
                        Segment::Match(m) => {

                            let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(m));
                            
                            let outcome = text_rule.compile(&compilable, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

                            for part in Into::<Vec<CompilationResultPart>>::into(outcome) {

                                new_content_parts.push(part);
            
                                let new_current_excluded_modifiers = current_excluded_modifiers.clone() + text_modifier.incompatible_modifiers().clone();
            
                                new_content_parts_additional_excluded_modifiers.push(new_current_excluded_modifiers);
                            
                            }
                        },
                        Segment::NonMatch(s) => {
                            new_content_parts.push(CompilationResultPart::Mutable { content: s.clone() });
                            new_content_parts_additional_excluded_modifiers.push(current_excluded_modifiers.clone());

                        },
                    }

                    Ok(())
                };

                for mat in matches {

                    if mat.start() > last_end {
                        let segment = Segment::NonMatch(content_part.content()[last_end..mat.start()].to_string());

                        elaborate_segment_fn(segment)?;
                    }

                    let segment = Segment::Match(mat.as_str().to_string());

                    elaborate_segment_fn(segment)?;

                    last_end = mat.end();
                }

                if last_end < content_part.content().len() {
                    let segment = Segment::NonMatch(content_part.content()[last_end..].to_string());

                    elaborate_segment_fn(segment)?;
                }
            }

            content_parts = new_content_parts;
            content_parts_additional_excluded_modifiers = new_content_parts_additional_excluded_modifiers;
            
        }
        
        Ok(CompilationResult::new(content_parts))
    }

}

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::constants::ESCAPE_HTML}, dossier::document::chapter::paragraph::{replacement_rule_paragraph::ReplacementRuleParagraph, Paragraph}, output_format::OutputFormat};

    use super::{compilation_rule::replacement_rule::{ReplacementRule, ReplacementRuleReplacerPart}, Compiler};


    #[test]
    fn compile_text() {

        let codex = Codex::of_html();
        let compilation_configuration = CompilationConfiguration::default();

        let content = "Text **bold text** `a **bold text** which must be not parsed`";

        let outcome = Compiler::compile_str(content, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();

        assert_eq!(outcome.content(), r#"Text <strong class="bold">bold text</strong> <code class="language-markup inline-code">a **bold text** which must be not parsed</code>"#)
    }

    #[test]
    fn compile_paragraph() {

        let codex = Codex::of_html();

        let mut paragraph = Box::new(ReplacementRuleParagraph::new(
            "\n\ntest\n\n".to_string(),
            Box::new(ReplacementRule::new(StandardParagraphModifier::CommonParagraph.modifier_pattern_with_paragraph_separator().clone(), vec![
                ReplacementRuleReplacerPart::new_fixed(String::from(r#"<p class="paragraph" data-nuid="$nuid">"#)),
                ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
                ReplacementRuleReplacerPart::new_fixed(String::from(r#"</p>"#)),
            ]))
        )) as Box<dyn Paragraph>;

        paragraph.set_nuid(Some("test-nuid".to_string()));

        paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))).unwrap();
        
        assert_eq!(paragraph.compilation_result().clone().unwrap().content(), concat!(
            r#"<p class="paragraph" data-nuid="test-nuid">"#,
            "test",
            r#"</p>"#
        ))
    }
}