use getset::{Getters, MutGetters, Setters};
use rayon::{iter::{IntoParallelRefMutIterator, ParallelIterator}, slice::ParallelSliceMut};
use serde::Serialize;
use crate::{codex::Codex, compilable_text::CompilableText, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome, compiled_text_accessor::CompiledTextAccessor}, dossier::document::{chapter::{chapter_header::ChapterHeader, paragraph::Paragraph}, Chapter}, load_block::{LoadBlock, LoadBlockContent}, output_format::OutputFormat};


#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct ContentBundle {
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    #[serde(skip)]      // TODO
    preamble: Vec<Box<dyn Paragraph>>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    chapters: Vec<Chapter>,

    compiled_content: Option<CompilableText>,
}


impl ContentBundle {

    pub fn new(preamble: Vec<Box<dyn Paragraph>>, chapters: Vec<Chapter>,) -> Self {
        Self {
            preamble,
            chapters,
            compiled_content: None
        }
    }

}

impl From<Vec<LoadBlock>> for ContentBundle {
    fn from(mut blocks: Vec<LoadBlock>) -> Self {
        if !blocks.windows(2).all(|w| {

            assert!(w[0].start() <= w[0].end());
            assert!(w[1].start() <= w[1].end());

            w[0].start() <= w[1].start()
        }) {
            
            blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));
        }

        let mut preamble: Vec<Box<dyn Paragraph>> = Vec::new();
        let mut current_chapter: Option<Chapter> = None;
        let mut chapters: Vec<Chapter> = Vec::new(); 

        for block in blocks {

            match Into::<LoadBlockContent>::into(block) {
                LoadBlockContent::Paragraph(paragraph) => {

                    if let Some(ref mut cc) = current_chapter {

                        cc.paragraphs_mut().push(paragraph);

                    } else {

                        preamble.push(paragraph);
                    }

                },
                LoadBlockContent::Heading(heading) => {

                    if let Some(cc) = current_chapter.take() {
                        chapters.push(cc);
                    }

                    assert!(current_chapter.is_none());

                    current_chapter = Some(Chapter::new(ChapterHeader::new(heading, Vec::new()), Vec::new()));
                },
                LoadBlockContent::ChapterTag(chapter_tag) => {

                    assert!(current_chapter.is_some());

                    current_chapter.as_mut().unwrap().header_mut().tags_mut().push(chapter_tag);

                },
            }
        }

        if let Some(cc) = current_chapter.take() {
            chapters.push(cc);
        }

        Self::new(preamble, chapters)
    }
}


impl Compilable for ContentBundle {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        if compilation_configuration_overlay.document_name().is_none() {
            return Err(CompilationError::DocumentNameNotFound)
        }
        
        let parallelization = compilation_configuration.parallelization();

        if parallelization {

            let preamble_results: Vec<Result<CompilationOutcome, CompilationError>> = self.preamble.par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).collect();

            let mut preamble_errors: Vec<CompilationError> = Vec::new();
            let mut preamble_outcomes: Vec<CompilationOutcome> = Vec::new();

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
            let mut chapter_outcomes: Vec<CompilationOutcome> = Vec::new();

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

            let preamble_outcomes: Vec<CompilationOutcome> = Vec::new();
            for paragraph in self.preamble.iter_mut() {

                preamble_outcomes.push(paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?);
            }
            
            let chapter_outcomes: Vec<CompilationOutcome> = Vec::new();
            for chapter in self.chapters.iter_mut() {
                
                chapter_outcomes.push(chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())?);
            }
        }

        // TODO

        Ok(())
    }
}



#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_text_modifier::StandardTextModifier, Modifier, ModifiersBucket}, Codex, ParagraphModifierOrderedMap, TextModifierOrderedMap}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::{replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart}, ReplacementRule}, CompilationRule}, compilable::Compilable}, output_format::OutputFormat};


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
            ParagraphModifierOrderedMap::new(),
            None
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