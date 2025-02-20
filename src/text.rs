pub mod text_section;
pub mod compilable_string;
pub mod content_block;
pub mod content_block_loading_rule;


use content_block::ContentBlock;
use getset::{Getters, MutGetters, Setters};
use rayon::{iter::{IntoParallelRefMutIterator, ParallelIterator}, slice::ParallelSliceMut};
use serde::Serialize;
use text_section::TextSection;

use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, dossier::document::{chapter::heading::HeadingLevel, Chapter}, output_format::OutputFormat};


#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct Text {
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    #[serde(skip)]      // TODO
    preamble: Vec<Box<dyn ContentBlock>>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    chapters: Vec<Chapter>,
}


impl Text {

    pub fn new(preamble: Vec<Box<dyn ContentBlock>>, chapters: Vec<Chapter>,) -> Self {
        Self {
            preamble,
            chapters,
        }
    }

}

impl From<Vec<TextSection>> for Text {
    fn from(mut blocks: Vec<TextSection>) -> Self {
        if !blocks.windows(2).all(|w| {

            assert!(w[0].start() <= w[0].end());
            assert!(w[1].start() <= w[1].end());

            w[0].start() <= w[1].start()        // TODO: assert!(w[0].start() <= w[1].start())
        }) {
            
            blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));
        }

        let mut preamble: Vec<Box<dyn ContentBlock>> = Vec::new();
        let mut current_chapter: Option<Chapter> = None;
        let mut chapters: Vec<Chapter> = Vec::new();
        let mut last_heading_level: u32 = 0;

        for block in blocks {

            match Into::<TextSection>::into(block) {
                TextSection::ContentBlock(paragraph) => {

                    if let Some(ref mut cc) = current_chapter {

                        cc.paragraphs_mut().push(paragraph);
      
                    } else {

                        preamble.push(paragraph);
                    }

                },
                TextSection::ChapterHeader(mut header) => {

                    if let Some(cc) = current_chapter.take() {
                        chapters.push(cc);
                    }

                    assert!(current_chapter.is_none());
      
                    let level = match header.heading().level() {
                        HeadingLevel::Minor => {
                            
                            let l;
                            if last_heading_level < 1 {
                                log::warn!("minor heading found, but last heading has level {}, so it is set as 1", last_heading_level);
                                
                                l = HeadingLevel::Explicit(1)
                            
                            } else {

                                l = HeadingLevel::Explicit(last_heading_level - 1);
                            }
                            
                            l
                        },
                        HeadingLevel::Major => {
                            let l;
                            if last_heading_level < 1 {
                                log::warn!("major heading found, but last heading has level {}, so it is set as 1", last_heading_level);
                                
                                l = HeadingLevel::Explicit(1)
                            
                            } else {

                                l = HeadingLevel::Explicit(last_heading_level + 1);
                            }
                            
                            l
                        },
                        HeadingLevel::Same => {
                            let l;
                            if last_heading_level < 1 {
                                log::warn!("same heading found, but last heading has level {}, so it is set as 1", last_heading_level);
                                
                                l = HeadingLevel::Explicit(1)
                            
                            } else {

                                l = HeadingLevel::Explicit(last_heading_level);
                            }
                            
                            l
                        },
                        HeadingLevel::Explicit(l) => HeadingLevel::Explicit(*l)
                    };

                    if let HeadingLevel::Explicit(l) = &level {
                    
                        last_heading_level = *l;
                    
                    } else {

                        unreachable!("heading level must be made 'explicit' now");
                    }

                    header.heading_mut().set_level(level);

                    current_chapter = Some(Chapter::new(header, Vec::new()));
                },
            }
        }

        if let Some(cc) = current_chapter.take() {
            chapters.push(cc);
        }

        Self::new(preamble, chapters)
    }
}


impl Compilable for Text {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        if compilation_configuration_overlay.document_name().is_none() {
            return Err(CompilationError::DocumentNameNotFound)
        }
        
        let parallelization = compilation_configuration.parallelization();

        let mut preamble_outcomes: Vec<CompilationOutcome> = Vec::new();
        let mut chapter_outcomes: Vec<CompilationOutcome> = Vec::new();

        if parallelization {

            let preamble_results: Vec<Result<CompilationOutcome, CompilationError>> = self.preamble.par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).collect();

            let mut preamble_errors: Vec<CompilationError> = Vec::new();

            preamble_results.into_iter().for_each(|result| {

                match result {
                    Ok(outcome) => preamble_outcomes.push(outcome),
                    Err(err) => preamble_errors.push(err),
                }
            });

            if !preamble_errors.is_empty() {
                return Err(CompilationError::BucketOfErrors(preamble_errors))
            }

            let chapter_results: Vec<Result<CompilationOutcome, CompilationError>> = self.chapters.par_iter_mut()
                .map(|chapter| {

                    chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).collect();

            let mut chapter_errors: Vec<CompilationError> = Vec::new();

            chapter_results.into_iter().for_each(|result| {

                match result {
                    Ok(outcome) => chapter_outcomes.push(outcome),
                    Err(err) => chapter_errors.push(err),
                }
            });

            if !chapter_errors.is_empty() {
                return Err(CompilationError::BucketOfErrors(chapter_errors))
            }
        
        } else {

            for paragraph in self.preamble.iter_mut() {

                preamble_outcomes.push(paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?);
            }
            
            for chapter in self.chapters.iter_mut() {
                
                chapter_outcomes.push(chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?);
            }
        }

        Ok(CompilationOutcome::from(codex.assembler().assemble_bundle(&preamble_outcomes, &chapter_outcomes, compilation_configuration_overlay.assembler_configuration())?))
    }
}



#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{assembler::html_assembler::HtmlAssembler, codex::{modifier::{base_modifier::BaseModifier, standard_text_modifier::StandardTextModifier, Modifier, ModifiersBucket}, Codex, ContentBlockModifierOrderedMap, TextModifierOrderedMap}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::{replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart}, ReplacementRule}, CompilationRule}}, output_format::OutputFormat};


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
            TextModifierOrderedMap::from([
                (
                    StandardTextModifier::BoldStarVersion.identifier(),
                    (
                        Box::new(Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)) as Box<dyn Modifier>,
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
                    ) as (Box<dyn Modifier>, Box<dyn CompilationRule>)
                )
            ]),
            ContentBlockModifierOrderedMap::new(),
            None,
            Box::new(HtmlAssembler::new())
        );

        compilable_text.compile(
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


}