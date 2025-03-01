pub(self) mod text_loader;

use getset::{Getters, MutGetters, Setters};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Serialize;
use text_loader::TextLoader;

use crate::{codex::Codex, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, dossier::document::chapter::{content_block::ContentBlock, Chapter}, load::{load_configuration::LoadConfiguration, load_error::LoadError}, output_format::OutputFormat};


/// Structured text which represents the base of loaded text. It has a `preamble` which contains the first content blocks and a list of chapters.
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

    pub fn load_from_str(raw_str: &str, codex: &Codex, configuration: &LoadConfiguration) -> Result<Self, LoadError> {

        let text_loader = TextLoader::new(codex, configuration);

        text_loader.load(raw_str)
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

    #[test]
    fn compile_fake_paragraph_with_bold_text() {

        todo!()
        /*let mut compilable_text = CompilableText::new(
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
        )*/
    }


}