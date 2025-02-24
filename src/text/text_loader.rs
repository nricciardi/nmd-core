use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{codex::Codex, dossier::document::chapter::{chapter_header::{chapter_header_loading_rule::ChapterHeaderLoadingRule, ChapterHeader}, heading::HeadingLevel, Chapter}, load::{load_configuration::LoadConfiguration, load_error::LoadError}, utility::datastruct::span::Span};
use super::{content_block::ContentBlock, content_block_loading_rule::ContentBlockLoadingRule, Text};

enum RawSpanContent<'a> {
    RawContentBlock(&'a str, &'a ContentBlockLoadingRule),
    RawHeader(&'a str, &'a ChapterHeaderLoadingRule),
    Unmatched(&'a str)
}

enum LoadedSpanContent {
    ContentBlock(Box<dyn ContentBlock>),
    Header(ChapterHeader)
}


#[derive(Debug)]
pub struct TextLoader<'a> {
    codex: &'a Codex,

    configuration: &'a LoadConfiguration
}


impl<'a> TextLoader<'a> {

    pub fn new(codex: &'a Codex, configuration: &'a LoadConfiguration) -> Self {
        Self {
            codex,
            configuration
        }
    }

    pub fn load(&self, raw_str: &str) -> Result<Text, LoadError> {
        let mut loaded_raw_spans = self.load_raw_spans_from_str_recursively(raw_str, 0, 0)?;

        self.build_text(loaded_raw_spans)
    }
    

    fn load_raw_spans_from_str_recursively(&self, current_str_slice: &str, offset: usize, loading_rule_index: usize) -> Result<Vec<Span<RawSpanContent<'_>>>, LoadError> {
        if let Some((modifier_identifier, (_, _))) = self.codex.paragraph_modifiers().get_index(loading_rule_index) {
            log::debug!("load using {}", modifier_identifier);

            return self.process_using_loading_rule(current_str_slice, offset, loading_rule_index);

        } else {

            log::debug!("next content contains headings and/or fallback paragraph:\n{}", current_str_slice);

            if self.codex.fallback_paragraph().is_none()  {

                log::warn!("there isn't fallback paragraph loading rule")
            }

            return self.process_headers_and_fallback(current_str_slice, offset);
        }
    }

    fn process_using_loading_rule(&self, current_str_slice: &str, offset: usize, loading_rule_index: usize) -> Result<Vec<Span<RawSpanContent<'_>>>, LoadError> {

        let loading_rule: &ContentBlockLoadingRule = self.codex.paragraph_modifiers().get_index(loading_rule_index).unwrap();
        
        let current_content_block_spans = loading_rule.find(
            current_str_slice,
            &self.codex,
            &self.configuration
        )?;

        let unmatched_spans: Vec<Span<RawSpanContent<'a>>> = Self::get_unmatched_spans(
                                                            current_str_slice,
                                                            &current_content_block_spans,
                                                            offset
                                                        );

        let current_content_block_spans: Vec<Span<RawSpanContent<'a>>> = current_content_block_spans.into_iter().map(|span| {
            Span::new(
                span.start() + offset,
                span.end() + offset,
                RawSpanContent::RawContentBlock(span.content(), loading_rule)
            )
        }).collect();
        
        // load unmatched slices
        let loaded_unmatched_slices: Vec<Span<RawSpanContent<'a>>> = self.load_unmatched_slices(unmatched_spans, offset, loading_rule_index)?;
        Ok([current_content_block_spans, loaded_unmatched_slices].concat())
    }

    fn get_unmatched_spans(str_slice: &'a str, matched_spans: &mut Vec<Span<&str>>, offset: usize) -> Vec<Span<RawSpanContent<'a>>> {

        matched_spans.sort_by(|a, b| {
                                                            assert!(a.start() < a.end());
                                                            assert!(b.start() < b.end());

                                                            a.start().cmp(&b.start())
                                                        });

        let mut content_block_spans: Vec<Span<RawSpanContent>> = Vec::new();
        let mut unmatched_spans: Vec<Span<RawSpanContent>> = Vec::new();

        let mut slice_position: usize = 0;
        for content_block_span in matched_spans {

            if content_block_span.start() > slice_position {
                unmatched_spans.push(
                    Span::new(
                        slice_position + offset,
                        content_block_span.start() + offset,
                        RawSpanContent::Unmatched(&str_slice[slice_position..content_block_span.start()])
                    )
                );
            }

            slice_position = content_block_span.end();
        }

        if slice_position < str_slice.len() {       // obtain last unmatched
            unmatched_spans.push(
                Span::new(
                    slice_position + offset,
                    str_slice.len() + offset,
                    RawSpanContent::Unmatched(&str_slice[slice_position..str_slice.len()])
                )
            );
        }

        unmatched_spans
    }

    fn load_unmatched_slices(&self, unmatched_slices: Vec<Span<RawSpanContent<'a>>>, offset: usize, current_loading_rule_index: usize) -> Result<Vec<Span<RawSpanContent<'a>>>, LoadError> {
        
        let loaded_unmatched_slices: Vec<Span<RawSpanContent<'a>>>;
        if self.configuration.parallelization() {

            loaded_unmatched_slices = self.par_load_unmatched_slices(
                unmatched_slices,
                offset,
                current_loading_rule_index
            )?;
        
        } else {

            loaded_unmatched_slices = self.seq_load_unmatched_slices(
                unmatched_slices,
                offset,
                current_loading_rule_index
            )?;
        }

        Ok(loaded_unmatched_slices)
    }

    fn par_load_unmatched_slices(&self, unmatched_slices: Vec<Span<RawSpanContent<'a>>>, offset: usize, current_loading_rule_index: usize) -> Result<Vec<Span<RawSpanContent<'a>>>, LoadError> {
        
        let mut loaded_spans: Vec<Result<Span<RawSpanContent<'a>>, LoadError>> = unmatched_slices.into_par_iter()
                .map(|span| {
                    if let RawSpanContent::Unmatched(unmatched_str) = span {

                        log::debug!("try next paragraph modifier on:\n{}\n(offset: {})", unmatched_str, offset);
        
                        return self.load_raw_spans_from_str_recursively(unmatched_str, offset, current_loading_rule_index + 1);
        
                    } else {
        
                        unreachable!("only unmatched spans are allowed");
                    }
                })
                .collect();

        let error = loaded_spans.par_iter().find_any(|res| res.is_err());

        if let Some(err) = error {
            return Err(err.err())
        }

        Ok(loaded_spans.into_par_iter().map(|res| res.ok()).collect())
    }

    fn seq_load_unmatched_slices(&self, unmatched_slices: Vec<Span<RawSpanContent<'a>>>, offset: usize, current_loading_rule_index: usize) -> Result<Vec<Span<RawSpanContent<'a>>>, LoadError> {
        
        let mut loaded_spans: Vec<Span<RawSpanContent<'a>>> = Vec::new();

        for unmatched_slice in unmatched_slices {

            if let RawSpanContent::Unmatched(unmatched_str) = unmatched_slice {

                log::debug!("try next paragraph modifier on:\n{}\n(offset: {})", unmatched_str, offset);

                let spans = self.load_raw_spans_from_str_recursively(unmatched_str, offset, current_loading_rule_index + 1)?;

                loaded_spans.extend(spans);

            } else {

                unreachable!("only unmatched spans are allowed");
            }
        }

        Ok(loaded_spans)
    }

    fn process_headers_and_fallback(&self, current_str_slice: &str, offset: usize) -> Result<Vec<Span<RawSpanContent<'_>>>, LoadError> {
        
        let mut headers_spans: Vec<Span<RawSpanContent<'a>>> = Vec::new();

        for (modifier_identifier, header_loading_rule) in self.codex.header_modifiers() {

            let spans = header_loading_rule.find(
                current_str_slice,
                &self.codex,
                &self.configuration
            )?;

            headers_spans.extend(spans);
        }
        
        let unmatched_spans = Self::get_unmatched_spans(current_str_slice, headers_spans, offset);

        // TODO: backlog: ad hoc method can be created to prevent double iterations
        // TODO: parallelization?
        unmatched_spans.into_iter().map(|span| {
            match span.content() {
                RawSpanContent::RawContentBlock(_, _) | RawSpanContent::RawHeader(_, _) => unreachable!("unexpected matched span"),
                RawSpanContent::Unmatched(raw_str) => RawSpanContent::RawContentBlock(&raw_str, self.codex.fallback_paragraph()),
            }
        })
        .collect()
    }

    fn process_raw_spans(&self, spans: Vec<Span<RawSpanContent<'a>>>) -> Result<Vec<Span<LoadedSpanContent>>, LoadError> {
        if self.configuration.parallelization() {
            self.par_process_raw_spans(spans)
        } else {
            self.seq_process_raw_spans(spans)
        }
    }

    fn par_process_raw_spans(&self, spans: Vec<Span<RawSpanContent<'a>>>) -> Result<Vec<Span<LoadedSpanContent>>, LoadError> {

        let loaded_spans: Vec<Result<Span<LoadedSpanContent>, LoadError>> = spans.into_par_iter()
            .map(|span| {
                match span.content() {
                    RawSpanContent::RawContentBlock(raw_str, loading_rule) | RawSpanContent::RawHeader(raw_str, loading_rule) => {
                        loading_rule.load(raw_str, &self.codex, &self.configuration)
                    }
                    RawSpanContent::Unmatched(_) => unreachable!("unmatched span is found"),
                }
            })
            .collect();

        loaded_spans
    }

    fn seq_process_raw_spans(&self, spans: Vec<Span<RawSpanContent<'a>>>) -> Result<Vec<Span<LoadedSpanContent>>, LoadError> {

        let loaded_spans: Vec<Span<LoadedSpanContent>> = Vec::new();

        for span in spans {
            match span.content() {
                RawSpanContent::RawContentBlock(raw_str, loading_rule) | RawSpanContent::RawHeader(raw_str, loading_rule) => {
                    loaded_spans.push(loading_rule.load(raw_str, &self.codex, &self.configuration)?);
                },
                RawSpanContent::Unmatched(_) => unreachable!("unmatched span is found"),
            }
        }

        Ok(loaded_spans)
    }

    fn build_text(&self, spans: Vec<Span<RawSpanContent<'a>>>) -> Result<Text, LoadError> {

        if self.configuration.parallelization() {
            spans.par_sort_by(|a, b| {

                assert!(a.start() <= a.end());
                assert!(b.start() <= b.end());

                a.start().cmp(&b.start())
            });

        } else {
            spans.sort_by(|a, b| {

                assert!(a.start() <= a.end());
                assert!(b.start() <= b.end());

                a.start().cmp(&b.start())
            });
        }

        let mut preamble: Vec<Box<dyn ContentBlock>> = Vec::new();
        let mut current_chapter: Option<Chapter> = None;
        let mut chapters: Vec<Chapter> = Vec::new();
        let mut last_heading_level: u32 = 0;

        for span in spans {

            match span {
                LoadedSpanContent::ContentBlock(content_block) => {

                    if let Some(ref mut cc) = current_chapter {

                        cc.paragraphs_mut().push(content_block);
      
                    } else {

                        preamble.push(content_block);
                    }

                },
                LoadedSpanContent::ChapterHeader(mut header) => {

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





#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn paragraphs_from_str() {

        todo!()

        // let content = concat!(
        //     "paragraph1",
        //     "\n\n",
        //     "paragraph2a\nparagraph2b",
        //     "\n\n",
        //     "paragraph3",
        // );

        // let codex = Codex::of_html();

        // let paragraphs = LoadBlock::load_from_str(content, &codex, LoadConfiguration::default()).unwrap();

        // assert_eq!(paragraphs.len(), 3)
    }
}