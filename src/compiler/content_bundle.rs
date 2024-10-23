use getset::{Getters, MutGetters, Setters};
use rayon::{iter::{IntoParallelRefMutIterator, ParallelIterator}, slice::ParallelSliceMut};
use serde::Serialize;

use crate::{codex::Codex, compiler::compilation_error::CompilationError, dossier::document::{chapter::paragraph::Paragraph, Chapter}, loader::load_block::{LoadBlock, LoadBlockContent}, output_format::OutputFormat};

use super::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, self_compile::SelfCompile};


#[derive(Debug, Getters, MutGetters, Setters, Serialize)]
pub struct ContentBundle {
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    #[serde(skip)]      // TODO
    preamble: Vec<Box<dyn Paragraph>>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    chapters: Vec<Chapter>
}


impl ContentBundle {

    pub fn new(preamble: Vec<Box<dyn Paragraph>>, chapters: Vec<Chapter>,) -> Self {
        Self {
            preamble,
            chapters
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

                    current_chapter = Some(Chapter::new(heading, Vec::new(), Vec::new()));
                },
                LoadBlockContent::ChapterTag(chapter_tag) => {

                    assert!(current_chapter.is_some());

                    current_chapter.as_mut().unwrap().tags_mut().push(chapter_tag);

                },
            }
        }

        if let Some(cc) = current_chapter.take() {
            chapters.push(cc);
        }

        Self::new(preamble, chapters)
    }
}



impl SelfCompile for ContentBundle {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        if compilation_configuration_overlay.document_name().is_none() {
            return Err(CompilationError::DocumentNameNotFound)
        }
        
        let parallelization = compilation_configuration.parallelization();

        if parallelization {

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.preamble.par_iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.chapters.par_iter_mut()
                .map(|chapter| {

                    chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find_any(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        
        } else {

            let maybe_one_failed: Option<Result<(), CompilationError>> = self.preamble.iter_mut()
                .map(|paragraph| {

                    paragraph.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())

                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
            
            let maybe_one_failed: Option<Result<(), CompilationError>> = self.chapters.iter_mut()
                .map(|chapter| {

                    chapter.compile(format, codex, compilation_configuration, compilation_configuration_overlay.clone())
                
                }).find(|result| result.is_err());

            if let Some(result) = maybe_one_failed {
                return result;
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_paragraph_modifier::StandardParagraphModifier, standard_text_modifier::StandardTextModifier, Modifier, ModifiersBucket}, Codex, CodexCompilationRulesMap, CodexLoadingRulesMap, CodexModifiersOrderedMap}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_rule::{replacement_rule::{replacement_rule_part::{closure_replacement_rule_part::ClosureReplacementRuleReplacerPart, fixed_replacement_rule_part::FixedReplacementRuleReplacerPart}, ReplacementRule}, CompilationRule}, self_compile::SelfCompile}, output_format::OutputFormat};


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
            CodexModifiersOrderedMap::from([
                (
                    StandardTextModifier::BoldStarVersion.identifier(),
                    Box::new(
                        Into::<BaseModifier>::into(StandardTextModifier::BoldStarVersion)
                    ) as Box<dyn Modifier>
                )
            ]),
            CodexModifiersOrderedMap::new(),
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
            Some(StandardParagraphModifier::CommonParagraph.identifier())
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