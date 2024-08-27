//! `Compiler` permits to transform a dossier or document based on output format and codex. Compiled dossier or document
//! should be assembled using appropriate `Assembler`


pub mod compilation_rule;
pub mod compilation_error;
pub mod compilation_configuration;
pub mod compiled_text_accessor;
pub mod self_compile;


use std::time::Instant;
use compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration};
use compilation_error::CompilationError;
use compilation_rule::CompilationRule;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use regex::Match;
use crate::{bibliography::{Bibliography, BIBLIOGRAPHY_FICTITIOUS_DOCUMENT}, codex::{modifier::ModifiersBucket, CodexIdentifier}, compilable_text::{compilable_text_part::{CompilableTextPart, CompilableTextPartType}, CompilableText}, dossier::{document::{chapter::heading::Heading, Chapter}, Document, Dossier}, output_format::OutputFormat, resource::{bucket::Bucket, resource_reference::ResourceReference}, table_of_contents::{TableOfContents, TOC_INDENTATION}};
use super::codex::Codex;


enum LoopIteration {
    Match {
        match_start: usize,
        match_end: usize,
        match_found: bool,
        matched_parts: Vec<CompilableTextPart>
    },
    EndParts,
}


#[derive(Debug)]
pub struct Compiler {
}

impl Compiler {

    /// Compile a dossier
    pub fn compile_dossier(dossier: &mut Dossier, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, mut compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {

        log::info!("compile dossier {} with ({} documents, parallelization: {})", dossier.name(), dossier.documents().len(), compilation_configuration.parallelization());

        compilation_configuration_overlay.set_dossier_name(Some(dossier.name().clone()));

        let fast_draft = compilation_configuration.fast_draft();

        if compilation_configuration.parallelization() {

            let compile_only_documents = compilation_configuration_overlay.compile_only_documents();

            let maybe_fails = dossier.documents_mut().par_iter_mut()
                .filter(|document| {
                    if fast_draft {
    
                        if let Some(subset) = compile_only_documents {

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

                    let res = Self::compile_document(document, format, codex, compilation_configuration, compilation_configuration_overlay.clone());

                    log::info!("document '{}' compiled in {} ms", document.name(), now.elapsed().as_millis());

                    res
                })
                .find_any(|result| result.is_err());

                if let Some(Err(fail)) = maybe_fails {
                    return Err(fail)
                }
            
        } else {

            let compile_only_documents = compilation_configuration_overlay.compile_only_documents();

            let maybe_fails = dossier.documents_mut().iter_mut()
                .filter(|document| {

                    if fast_draft {

                        if let Some(subset) = compile_only_documents {

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

                    let res = Self::compile_document(document, format, codex, compilation_configuration, compilation_configuration_overlay.clone());

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

            Self::compile_table_of_contents(&mut table_of_contents, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
            dossier.set_table_of_contents(Some(table_of_contents));
        }

        if dossier.configuration().bibliography().include_in_output() {
            let mut bibliography = Bibliography::new(
                dossier.configuration().bibliography().title().clone(),
                dossier.configuration().bibliography().records().clone()
            );

            Self::compile_bibliography(&mut bibliography, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
        
            dossier.set_bibliography(Some(bibliography));
        }

        Ok(())
    }

    /// Compile document
    pub fn compile_document(document: &mut Document, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, mut compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {

        let parallelization = compilation_configuration.parallelization();

        log::info!("compile {} chapters of document: '{}'", document.chapters().len(), document.name());

        compilation_configuration_overlay.set_document_name(Some(document.name().clone()));

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

                    Self::compile_chapter(chapter, format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
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

                    Self::compile_chapter(chapter, format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        }

        Ok(())

    }

    /// Compile chapter
    pub fn compile_chapter(chapter: &mut Chapter, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {

        Self::compile_heading(chapter.heading_mut(), format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

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
    pub fn compile_heading(heading: &mut Heading, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {

        let document_name = compilation_configuration_overlay.document_name().as_ref();

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

                let outcome = CompilableText::new(vec![

                    CompilableTextPart::new(
                        format!(r#"<h{} class="heading-{}" id="{}" {}>"#, heading.level(), heading.level(), id.build_without_internal_sharp(), nuid_attr),
                        CompilableTextPartType::Fixed
                    ),
                    CompilableTextPart::new(
                        compiled_title.content(),
                        CompilableTextPartType::Compilable{ incompatible_modifiers: ModifiersBucket::None }
                    ),
                    CompilableTextPart::new(
                        format!(r#"</h{}>"#, heading.level()),
                        CompilableTextPartType::Fixed
                    ),
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
    pub fn compile_table_of_contents(table_of_contents: &mut TableOfContents, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        if table_of_contents.headings().is_empty() {
            
            return Ok(());
        }

        if table_of_contents.page_numbers() {
            log::error!("table of contents with page numbers not already usable...");

            unimplemented!("table of contents with page numbers not already usable...");
        }
        
        match format {
            OutputFormat::Html => {
                let mut outcome = CompilableText::new_empty();

                outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"<section class="toc"><div class="toc-title">"#)));
                outcome.parts_mut().append(&mut Self::compile_str(table_of_contents.title(), format, codex, compilation_configuration, compilation_configuration_overlay.clone())?.parts_mut());
                outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</div><ul class="toc-body">"#)));

                let mut total_li = 0;

                for heading in table_of_contents.headings() {

                    let heading_lv: u32 = heading.level();

                    if heading_lv > table_of_contents.maximum_heading_level() as u32 {
                        continue;
                    }

                    outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"<li class="toc-item">"#)));

                    if !table_of_contents.plain() {

                        let min_heading_lv = Self::min_headers_lv(table_of_contents.headings());

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

                    outcome.parts_mut().append(Self::compile_str(heading.title(), format, codex, compilation_configuration, compilation_configuration_overlay.clone())?.parts_mut());

                    if let Some(_) = heading.resource_reference() {

                        outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</a>"#)));
                    }

                    outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</span></li>"#)));

                    total_li += 1;
                        
                }

                outcome.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</ul></section>"#)));

                table_of_contents.set_compilation_result(Some(outcome));

                log::info!("compiled table of contents ({} lines, {} skipped)", total_li, table_of_contents.headings().len() - total_li);

                Ok(())
            },
        }
    }

    /// Compile bibliography
    pub fn compile_bibliography(bibliography: &mut Bibliography, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        log::info!("compiling bibliography...");

        match format {
            OutputFormat::Html => {
                let mut compilation_result = CompilableText::new_empty();

                compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"<section class="bibliography"><div class="bibliography-title">"#)));
                compilation_result.parts_mut().append(&mut Self::compile_str(bibliography.title(), format, codex, compilation_configuration, compilation_configuration_overlay)?.parts_mut());
                compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</div><ul class="bibliography-body">"#)));
        
                for (bib_key, bib_record) in bibliography.content().iter() {
                    compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(
                        r#"<div class="bibliography-item" id="{}"><div class="bibliography-item-title">{}</div>"#,
                        ResourceReference::of_internal_from_without_sharp(bib_key,
                            Some(&BIBLIOGRAPHY_FICTITIOUS_DOCUMENT))?.build_without_internal_sharp(),
                            bib_record.title()
                        )));
                
                    if let Some(authors) = bib_record.authors() {
        
                        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<div class="bibliography-item-authors">{}</div>"#, authors.join(", "))));
                    }
        
                    if let Some(year) = bib_record.year() {

                        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<div class="bibliography-item-year">{}</div>"#, year)));
                    }
        
                    if let Some(url) = bib_record.url() {
        
                        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<div class="bibliography-item-url">{}</div>"#, url)));
                    }
        
                    if let Some(description) = bib_record.description() {

                        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<div class="bibliography-item-description">{}</div>"#, description)));
        
                    }
        
                    compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</div>"#)));
                }
        
                compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</ul></section>"#)));
        
                bibliography.set_compilation_result(Some(compilation_result));
        
                log::info!("bibliography compiled");
        
                Ok(())
            },
        }
    }

    /// Compile a string
    pub fn compile_str(content: &str, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilableText, CompilationError> {

        let excluded_modifiers = compilation_configuration_overlay.excluded_modifiers().clone();

        log::debug!("start to compile content:\n{}\nexcluding: {:?}", content, excluded_modifiers);

        let mut compilable_text = CompilableText::from(CompilableTextPart::new_compilable(
            content.to_string(),
            ModifiersBucket::None
        ));

        if excluded_modifiers == Bucket::All {
            log::debug!("compilation of content:\n{} is skipped because are excluded all modifiers", content);
            
            return Ok(compilable_text)
        }

        Self::compile_compilable_text(&mut compilable_text, format, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

        Ok(compilable_text)
    }

    pub fn compile_compilable_text(compilable_text: &mut CompilableText, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        let excluded_modifiers = compilation_configuration_overlay.excluded_modifiers().clone();

        log::debug!("start to compile content:\n{:?}\nexcluding: {:?}", compilable_text, excluded_modifiers);

        if excluded_modifiers == Bucket::All {
            log::debug!("compilation of content:\n{:?} is skipped because are excluded all modifiers", compilable_text);
            
            return Ok(())
        }

        for (codex_identifier, text_modifier) in codex.text_modifiers() {

            if excluded_modifiers.contains(codex_identifier) {

                log::debug!("{:?} is skipped", text_modifier);
                continue;
            }

            if let Some(text_rule) = codex.text_compilation_rules().get(codex_identifier) {

                Self::compile_compilation_text_with_compilation_rule(compilable_text, (codex_identifier, text_rule), format, compilation_configuration, compilation_configuration_overlay.clone())?;

            } else {

                log::warn!("text rule for {:#?} not found", text_modifier);
                continue;
            }
        }

        Ok(())
    }

    /// Compile parts and return the new compiled parts or `None` if there are not matches using
    /// provided rule
    pub fn compile_compilation_text_with_compilation_rule(compilable_text: &mut CompilableText, (rule_identifier, rule): (&CodexIdentifier, &Box<dyn CompilationRule>), format: &OutputFormat, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
    
        let parts = compilable_text.parts();

        let mut compilable_content = String::new();
        let mut compilable_content_end_parts_positions: Vec<usize> = Vec::new();

        parts.iter()
                .filter(|part| {
                    match &part.part_type() {
                        CompilableTextPartType::Fixed => false,
                        CompilableTextPartType::Compilable{ incompatible_modifiers } => {
                            if incompatible_modifiers.contains(&rule_identifier) {
                                return false
                            } else {
                                return true
                            }
                        },
                    }
                })
                .for_each(|part| {

                    compilable_content.push_str(part.content());

                    let last_pos = *compilable_content_end_parts_positions.last().unwrap_or(&0);

                    compilable_content_end_parts_positions.push(last_pos + part.content().len());
                });

        let matches = rule.find_iter(&compilable_content);

        if matches.len() == 0 {
            log::debug!("'{}' => no matches with {:?}", compilable_content, rule);
            
            return Ok(());
        }

        log::debug!("'{}' => there is a match with {:#?}", compilable_content, rule);

        let mut compiled_parts: Vec<CompilableTextPart> = Vec::new();     // final output

        let mut parts_index: usize = 0;
        let mut compilable_parts_index: usize = 0;

        // only for compilable parts
        let mut part_start_position_in_compilable_content: usize = 0;
        let mut part_end_position_in_compilable_content: usize;

        let mut match_index: usize = 0;

        while parts_index < parts.len() {      // there are other parts

            let match_start_end: Option<(usize, usize)>;        // start and end

            if match_index < matches.len() {

                let current_evaluated_match = matches[match_index];

                match_index += 1;    
            
                match_start_end = Some((
                    current_evaluated_match.start(),
                    current_evaluated_match.end()
                ));

            } else {

                match_start_end = None;
            }

            let mut match_found = false;

            let mut matched_parts: Vec<CompilableTextPart> = Vec::new();
            
            'parts_loop: while parts_index < parts.len() {

                let part = &parts[parts_index];

                parts_index += 1;   // for next iteration

                match part.part_type() {
                    CompilableTextPartType::Fixed => {

                        if let Some((_start, _end)) = match_start_end {

                            if match_found {        // matching end cannot be in a fixed part

                                matched_parts.push(part.clone());
        
                                continue 'parts_loop;
                            
                            } else {
                                
                                compiled_parts.push(part.clone());      // direct in compiled_parts
    
                                continue 'parts_loop;
                            }
                        
                        } else {
                            compiled_parts.push(part.clone());      // direct in compiled_parts

                            continue 'parts_loop;
                        }
                    },
                    CompilableTextPartType::Compilable{ incompatible_modifiers } => {

                        part_end_position_in_compilable_content = compilable_content_end_parts_positions[compilable_parts_index];
                        
                        compilable_parts_index += 1;

                        if let Some((match_start, match_end)) = match_start_end {

                            if !match_found && part_end_position_in_compilable_content <= match_start {      // there is no match in this part
                            
                                let sub_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];

                                compiled_parts.push(CompilableTextPart::new(
                                    sub_part.to_string(),
                                    CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                ));
    
                            } else {
                                // ...part has a match
    
                                if !match_found     // first part in which current match is found
                                    && part_start_position_in_compilable_content <= match_start
                                    && match_start < part_end_position_in_compilable_content {

                                    // === pre-matched part ==
                                    let pre_matched_part = &compilable_content[part_start_position_in_compilable_content..match_start];
                                                                        
                                    if !pre_matched_part.is_empty() {
                                        compiled_parts.push(CompilableTextPart::new(
                                            pre_matched_part.to_string(),
                                            CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                        ));
                                    }

                                    part_start_position_in_compilable_content = match_start;

                                    // === matched part ===
                                    let matched_part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content.min(match_end)];

                                    matched_parts.push(CompilableTextPart::new(
                                        matched_part.to_string(),
                                        CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                    ));
                                }
                                
                                if match_end <= part_end_position_in_compilable_content {       // matching end is in this part

                                    if match_found {   // the matching end is in another part respect of matching start

                                        let matched_part = &compilable_content[part_start_position_in_compilable_content..match_end];

                                        matched_parts.push(CompilableTextPart::new(
                                            matched_part.to_string(),
                                            CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                        ));
                                    }

                                    // compile and append found matched parts
                                    compiled_parts.append(
                                        &mut rule.compile(
                                            &CompilableText::from(matched_parts),
                                            format,
                                            compilation_configuration,
                                            compilation_configuration_overlay.clone()
                                        )?.parts_mut() 
                                    );

                                    // re-start next parts loop from this part
                                    parts_index -= 1;       
                                    compilable_parts_index -= 1;

                                    part_start_position_in_compilable_content = match_end;

                                    break 'parts_loop;

                                } else {

                                    if match_found {        // simple matched part in matched parts 

                                        matched_parts.push(part.clone());
                                    }
                                }

                                match_found = true;     // update to check if match is found in next iterations
                            }
        
                            // update start position
                            part_start_position_in_compilable_content = part_end_position_in_compilable_content;

                        } else {
                            
                            let part = &compilable_content[part_start_position_in_compilable_content..part_end_position_in_compilable_content];
                                                                            
                            if !part.is_empty() {
                                compiled_parts.push(CompilableTextPart::new(
                                    part.to_string(),
                                    CompilableTextPartType::Compilable{ incompatible_modifiers: incompatible_modifiers.clone() }
                                ));
                            }
                        }
                    }

                }
            }
        }

        compilable_text.set_parts(compiled_parts);
        
        Ok(())
    }

}

#[cfg(test)]
mod test {
    
    use std::{collections::HashSet, sync::Arc};
    use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_paragraph_modifier::StandardParagraphModifier, standard_text_modifier::StandardTextModifier, Modifier, ModifiersBucket}, Codex, CodexCompilationRulesMap, CodexLoadingRulesMap, CodexModifiersMap}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::constants::ESCAPE_HTML, Compiler}, dossier::document::chapter::paragraph::{replacement_rule_paragraph::ReplacementRuleParagraph, Paragraph}, output_format::OutputFormat};

    use super::compilation_rule::{replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart}, ReplacementRule}, CompilationRule};


    #[test]
    fn compile_fake_paragraph_with_bold_text() {

        let mut compilable_text = CompilableText::new(
            vec![
                CompilableTextPart::new_fixed(String::from("<p>")),
                CompilableTextPart::new_compilable(
                    String::from("This is a **bold text**!"),
                    ModifiersBucket::None
                ),
                CompilableTextPart::new_fixed(String::from(" &euro; ")),
                CompilableTextPart::new_compilable(
                    String::from("**again"),
                    ModifiersBucket::None
                ),
                CompilableTextPart::new_fixed(String::from(" &euro;")),
                CompilableTextPart::new_compilable(
                    String::from("**"),
                    ModifiersBucket::None
                ),
                CompilableTextPart::new_fixed(String::from("</p>")),
            ],
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
                                Arc::new(ClosureReplacementRuleReplacerPart::new(Arc::new(|captures, compilable, _, _, _| {
                
                                    let capture1 = captures.get(1).unwrap();
                                    
                                    let slice = compilable.parts_slice(capture1.start(), capture1.end())?;
                    
                                    Ok(CompilableText::new(slice))
                                }))),
                                Arc::new(FixedReplacementRuleReplacerPart::new(String::from("</strong>"))),
                            ]
                        )
                    ) as Box<dyn CompilationRule>
                )
            ]),
            CodexLoadingRulesMap::new(),
        );

        Compiler::compile_compilable_text(
            &mut compilable_text,
            &OutputFormat::Html,
            &codex,
            &CompilationConfiguration::default(),
            CompilationConfigurationOverLay::default()
        ).unwrap();

        assert_eq!(
            compilable_text.content(),
            "<p>This is a <strong>bold text</strong>! &euro; <strong>again &euro;</strong></p>"
        )
    }


    #[test]
    fn compile_nested_modifiers() {

        let mut codex = Codex::of_html();

        codex.retain(HashSet::from([
            StandardTextModifier::BoldStarVersion.identifier(),
            StandardTextModifier::BoldUnderscoreVersion.identifier(),
            StandardTextModifier::ItalicStarVersion.identifier(),
            StandardTextModifier::ItalicUnderscoreVersion.identifier(),
            StandardTextModifier::InlineCode.identifier(),
        ]));

        let compilation_configuration = CompilationConfiguration::default();

        let content = "A piece of **bold text**, *italic text*, `a **(fake) bold text** which must be not parsed` and *nested **bold text***";

        let outcome = Compiler::compile_str(content, &OutputFormat::Html, &codex, &compilation_configuration, CompilationConfigurationOverLay::default()).unwrap();       

        assert_eq!(outcome.content(), concat!(
            "A piece of ",
            r#"<strong class="bold">bold text</strong>, "#,
            r#"<em class="italic">italic text</em>, "#,
            r#"<code class="language-markup inline-code">a **(fake) bold text** which must be not parsed</code>"#,
            r#" and "#,
            r#"<em class="italic">nested <strong class="bold">bold text</strong></em>"#,
        ));
    }

    #[test]
    fn compile_paragraph() {

        todo!()

        // let codex = Codex::of_html();

        // let mut paragraph = Box::new(ReplacementRuleParagraph::new(
        //     "\n\ntest\n\n".to_string(),
        //     Box::new(ReplacementRule::new(StandardParagraphModifier::CommonParagraph.modifier_pattern_with_paragraph_separator().clone(), vec![
        //         ReplacementRuleReplacerPart::new_fixed(String::from(r#"<p class="paragraph" data-nuid="$nuid">"#)),
        //         ReplacementRuleReplacerPart::new_mutable(String::from(r#"$1"#)).with_post_replacing(Some(ESCAPE_HTML.clone())),
        //         ReplacementRuleReplacerPart::new_fixed(String::from(r#"</p>"#)),
        //     ]))
        // )) as Box<dyn Paragraph>;

        // paragraph.set_nuid(Some("test-nuid".to_string()));

        // paragraph.compile(&OutputFormat::Html, &codex, &CompilationConfiguration::default(), CompilationConfigurationOverLay::default()).unwrap();
        
        // assert_eq!(paragraph.compilation_result().clone().unwrap().content(), concat!(
        //     r#"<p class="paragraph" data-nuid="test-nuid">"#,
        //     "test",
        //     r#"</p>"#
        // ))
    }
}