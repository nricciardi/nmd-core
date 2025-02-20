use crate::{codex::Codex, dossier::document::chapter::chapter_header::ChapterHeader};
use super::content_block::ContentBlock;
use std::sync::RwLock;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use rayon::{iter::{IntoParallelIterator, ParallelIterator}, slice::ParallelSliceMut};



#[derive(Debug)]
pub enum TextSectionContent {
    ContentBlock(Box<dyn ContentBlock>),
    ChapterHeader(ChapterHeader)
}


#[derive(Debug, Getters, CopyGetters, MutGetters, Setters)]
pub struct TextSection {

    #[getset(get_copy = "pub", set = "pub")]
    start: usize,

    #[getset(get_copy = "pub", set = "pub")]
    end: usize,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    content: LoadBlockContent
}

impl TextSection {
    pub fn new(start: usize, end: usize, content: LoadBlockContent) -> Self {
        Self {
            start,
            end,
            content,
        }
    }

    

    /// Load content from `&str` based on `Codex`
    /// 
    /// Blocks are not sorted, sort if you want:
    /// 
    /// `blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));``
    /// 
    pub fn load_from_str(content: &str, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<Self>, LoadError> {
        Self::internal_load_from_str_recursively(content, 0, codex, 0, configuration)
    }

    /// Inner load method to load content from `&str` based on `Codex`
    /// 
    /// This method uses recursive algorithm, use `content_offset=0` and `paragraph_modifier_index=0` to start.
    fn internal_load_from_str_recursively(current_content: &str, offset: usize, codex: &Codex, paragraph_modifier_index: usize, configuration: LoadConfiguration) -> Result<Vec<LoadBlock>, LoadError> {

        if let Some((modifier_identifier, (paragraph_modifier, paragraph_loading_rule))) = codex.paragraph_modifiers().get_index(paragraph_modifier_index) {

            let (current_paragraph_blocks, unmatched_slices) = Self::load_paragraph_blocks_using_modifier(current_content, offset, paragraph_modifier_index, modifier_identifier, paragraph_modifier, paragraph_loading_rule, codex, configuration)?;    

            // load unmatched slices
            if configuration.parallelization() {

                Self::par_load_unmatched_slices(current_paragraph_blocks, unmatched_slices, offset, paragraph_modifier_index, modifier_identifier, paragraph_modifier, paragraph_loading_rule, codex, configuration)
            
            } else {

                Self::seq_load_unmatched_slices(current_paragraph_blocks, unmatched_slices, offset, paragraph_modifier_index, modifier_identifier, paragraph_modifier, paragraph_loading_rule, codex, configuration)
            }

            
        } else {    // => there are no other modifiers

            log::debug!("next content contains headings and/or fallback paragraph:\n{}", current_content);

            if codex.fallback_paragraph().is_none()  {

                log::warn!("there isn't fallback paragraph loading rule")
            }

            Self::load_header_and_fallback_blocks(current_content, offset, codex, configuration)
        }
    }

    fn position_in_global_content(position_in_current: usize, offset: usize) -> usize {
        position_in_current + offset
    }

    fn load_header_and_fallback_blocks(current_content: &str, offset: usize, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<LoadBlock>, LoadError> {
        
        // load headers
        let mut headers_blocks = ChapterHeader::load(current_content, codex, &configuration)?;

        headers_blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));

        let mut blocks: Vec<LoadBlock> = Vec::new();

        let mut add_fb_blocks = |raw_fb_paragraph: &str, start: usize, end: usize| -> Result<(), LoadError> {

            if let Some((fb_id, fallback_loading_rule)) = codex.fallback_paragraph() {

                log::debug!("fallback rule {} will be used to load:\n{}", fb_id, raw_fb_paragraph);

                let paragraphs = fallback_loading_rule.load(raw_fb_paragraph, codex, configuration.clone())?;

                let len = paragraphs.len();
                assert!((end - start) > len);

                for (index, paragraph) in paragraphs.into_iter().enumerate() {

                    let fake_start = start + ((end - start) / len * index); 
                    let fake_end = start + ((end - start) / len * (index + 1)); 

                    let block = LoadBlock::new(
                        fake_start,
                        fake_end,
                        LoadBlockContent::Paragraph(paragraph)
                    );

                    log::debug!("generated fallback blocks:\n{:#?}", block);

                    blocks.push(block);
                }
            }

            Ok(())
        };

        let mut last_position = 0;

        // assign fallback paragraph
        for header_block in headers_blocks.iter_mut() {

            if header_block.start() > last_position {

                let start = last_position;
                let global_start = Self::position_in_global_content(last_position, offset);
                let end = header_block.start();
                let global_end = Self::position_in_global_content(header_block.start(), offset);

                let s = &current_content[start..end];

                log::debug!("found not header slice between {} (global pos: {}) and {} (global pos: {}) of current content:\n{}", start, global_start, end, global_end, s);

                add_fb_blocks(
                    s,
                    global_start,
                    global_end
                )?;
            }

            last_position = header_block.end();

            header_block.set_start(Self::position_in_global_content(header_block.start(), offset));
            header_block.set_end(Self::position_in_global_content(header_block.end(), offset));
        }

        log::debug!("last heading found at position (of current content): {}/{}", last_position, current_content.len());

        if current_content.len() > last_position {

            let s = &current_content[last_position..];

            add_fb_blocks(
                s,
                Self::position_in_global_content(last_position, offset),
                Self::position_in_global_content(current_content.len(), offset)
            )?;
        }

        blocks.append(&mut headers_blocks);

        return Ok(blocks);
    }

    
    fn load_paragraph_blocks_using_modifier<'a>(current_content: &'a str, offset: usize, 
        paragraph_modifier_index: usize, modifier_identifier: ModifierIdentifier, paragraph_modifier: &Box<dyn Modifier>, 
        paragraph_loading_rule: ParagraphLoadingRule, codex: &Codex, configuration: LoadConfiguration
    ) -> Result<(Vec<LoadBlock>, Vec<(usize, &'a str)>), LoadError> {
        
        log::debug!("load using {}", modifier_identifier);

        let mut current_paragraph_blocks: Vec<LoadBlock> = Vec::new();

        let mut unmatched_slices: Vec<(usize, &str)> = Vec::new();
        let mut last_position: usize = 0;

        // elaborate content based on current paragraph modifier
        for m in paragraph_modifier.modifier_pattern_regex().find_iter(current_content) {

            assert!(!m.is_empty());

            let m_start = m.start();
            let m_end = m.end();

            log::debug!("match found between {} and {}", m_start, m_end);

            // save previous slice, it will be loaded after
            if m_start > last_position {
                unmatched_slices.push((Self::position_in_global_content(last_position, offset), &current_content[last_position..m_start]));
            }

            last_position = m_end;

            let paragraph = paragraph_loading_rule.load(m.as_str(), codex, configuration.clone())?;

            if !paragraph.is_empty() {

                let block = LoadBlock::new(
                    Self::position_in_global_content(m_start, offset),
                    Self::position_in_global_content(m_end, offset),
                    LoadBlockContent::Paragraph(paragraph)
                );

                log::debug!("added block:\n{:#?}", block);

                current_paragraph_blocks.push(block);
            }
        }

        // take last slice (if exists)
        if current_content.len() > last_position {
            unmatched_slices.push((Self::position_in_global_content(last_position, offset), &current_content[last_position..]));
        }

        Ok((current_paragraph_blocks, unmatched_slices))
    }

    fn par_load_unmatched_slices(mut current_paragraph_blocks: Vec<LoadBlock>, unmatched_slices: Vec<(usize, &str)>, offset: usize, paragraph_modifier_index: usize, modifier_identifier: ModifierIdentifier, paragraph_modifier: &Box<dyn Modifier>, paragraph_loading_rule: ParagraphLoadingRule, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<LoadBlock>, LoadError> {

        let unmatched_slices_blocks: RwLock<Vec<LoadBlock>> = RwLock::new(Vec::new());

        let errors: Vec<LoadError> = unmatched_slices.into_par_iter().map(|(offset, unmatched_slice)| -> Result<(), LoadError> {

            log::debug!("no matches using paragraph modifier {} on:\n{}\n(offset: {})", modifier_identifier, unmatched_slice, offset);

            let mut blocks = Self::internal_load_from_str_recursively(unmatched_slice, offset, codex, paragraph_modifier_index + 1, configuration.clone())?;
        
            unmatched_slices_blocks.write().unwrap().append(&mut blocks);

            Ok(())
        })
        .filter(|result| result.is_err())
        .map(|result| result.err().unwrap())
        .collect();

        if errors.len() > 0 {
            return Err(LoadError::BucketOfErrors(errors))
        }

        let mut unmatched_slices_blocks = unmatched_slices_blocks.into_inner().unwrap();

        current_paragraph_blocks.append(&mut unmatched_slices_blocks);

        Ok(current_paragraph_blocks)
    }

    fn seq_load_unmatched_slices(mut current_paragraph_blocks: Vec<LoadBlock>, unmatched_slices: Vec<(usize, &str)>, offset: usize, paragraph_modifier_index: usize, modifier_identifier: ModifierIdentifier, paragraph_modifier: &Box<dyn Modifier>, paragraph_loading_rule: ParagraphLoadingRule, codex: &Codex, configuration: LoadConfiguration) -> Result<Vec<LoadBlock>, LoadError> {
        
        let mut unmatched_slices_blocks: Vec<LoadBlock> = Vec::new();

        for (offset, unmatched_slice) in unmatched_slices {

            log::debug!("try next paragraph modifier on:\n{}\n(offset: {})", unmatched_slice, offset);

            let mut blocks = Self::internal_load_from_str_recursively(unmatched_slice, offset, codex, paragraph_modifier_index + 1, configuration.clone())?;
        
            unmatched_slices_blocks.append(&mut blocks);
        }

        current_paragraph_blocks.append(&mut unmatched_slices_blocks);

        Ok(current_paragraph_blocks)
    }
}

impl Into<LoadBlockContent> for TextSection {
    fn into(self) -> LoadBlockContent {
        self.content
    }
}

impl TryInto<Box<dyn ContentBlock>> for TextSection {
    type Error = String;

    fn try_into(self) -> Result<Box<dyn ContentBlock>, Self::Error> {
        if let LoadBlockContent::Paragraph(p) = self.content {
            return Ok(p)
        }

        Err(String::from("this block doesn't contain a paragraph"))
    }
}

#[derive(Debug)]
pub enum LoadBlockContent {
    Paragraph(Box<dyn ContentBlock>),
    ChapterHeader(ChapterHeader)
}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn paragraphs_from_str() {
        let content = concat!(
            "paragraph1",
            "\n\n",
            "paragraph2a\nparagraph2b",
            "\n\n",
            "paragraph3",
        );

        let codex = Codex::of_html();

        let paragraphs = LoadBlock::load_from_str(content, &codex, LoadConfiguration::default()).unwrap();

        assert_eq!(paragraphs.len(), 3)
    }
}