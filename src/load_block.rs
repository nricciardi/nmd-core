use getset::{CopyGetters, Getters, MutGetters, Setters};
use crate::{codex::Codex, dossier::document::chapter::{chapter_header::ChapterHeader, chapter_tag::ChapterTag, heading::Heading, paragraph::Paragraph}, load::{LoadConfiguration, LoadConfigurationOverLay, LoadError}};





#[derive(Debug, Getters, CopyGetters, MutGetters, Setters)]
pub struct LoadBlock {

    #[getset(get_copy = "pub", set = "pub")]
    start: usize,

    #[getset(get_copy = "pub", set = "pub")]
    end: usize,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    content: LoadBlockContent
}

impl LoadBlock {
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
    /// ```rust
    /// blocks.par_sort_by(|a, b| a.start().cmp(&b.start()));
    /// ```
    pub fn load_from_str(content: &str, codex: &Codex, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Vec<LoadBlock>, LoadError> {
        Self::inner_load_from_str(content, 0, codex, 0, configuration, configuration_overlay.clone())
    }

    /// Inner load method to load content from `&str` based on `Codex`
    /// 
    /// This method uses recursive algorithm, use `content_offset=0` and `paragraph_modifier_index=0` to start.
    fn inner_load_from_str(content: &str, content_offset: usize, codex: &Codex, paragraph_modifier_index: usize, configuration: &LoadConfiguration, configuration_overlay: LoadConfigurationOverLay) -> Result<Vec<LoadBlock>, LoadError> {

        if let Some((modifier_identifier, (paragraph_modifier, paragraph_loading_rule))) = codex.paragraph_modifiers().get_index(paragraph_modifier_index) {

            log::debug!("load using {}", modifier_identifier);

            let mut current_paragraph_blocks: Vec<LoadBlock> = Vec::new();

            let mut unmatched_slices: Vec<(usize, &str)> = Vec::new();
            let mut last_position: usize = 0;

            // elaborate content based on current paragraph modifier
            for m in paragraph_modifier.modifier_pattern_regex().find_iter(content) {

                assert!(!m.is_empty());

                let m_start = content_offset + m.start();
                let m_end = content_offset + m.end();

                // save previous slice, it will be loaded after
                if m_start > last_position {
                    unmatched_slices.push((last_position, &content[last_position..m_start]));
                }

                last_position = m_end;

                let paragraph = paragraph_loading_rule.load(m.as_str(), codex, configuration, configuration_overlay.clone())?;

                if !paragraph.is_empty() {
                    let block = LoadBlock::new(m_start, m_end, LoadBlockContent::Paragraph(paragraph));

                    log::debug!("added block:\n{:#?}", block);

                    current_paragraph_blocks.push(block);
                }
            }

            // take last slice (if exists)
            if content.len() > last_position {
                unmatched_slices.push((last_position, &content[last_position..]));
            }

            let mut unmatched_slices_blocks: Vec<LoadBlock> = Vec::new();

            // load unmatched slices
            for (offset, unmatched_slice) in unmatched_slices {
                let mut blocks = Self::inner_load_from_str(unmatched_slice, offset, codex, paragraph_modifier_index + 1, configuration, configuration_overlay.clone())?;
            
                unmatched_slices_blocks.append(&mut blocks);
            }

            current_paragraph_blocks.append(&mut unmatched_slices_blocks);

            return Ok(current_paragraph_blocks)

        } else {    // => there are no other modifiers 

            // load headings
            let mut headings_blocks = ChapterHeader::load_headings_and_chapter_tags_from_str(content, codex, configuration)?;

            let mut blocks: Vec<LoadBlock> = Vec::new();

            let mut last_position = 0;

            if codex.fallback_paragraph_modifier().is_none()  {

                log::warn!("there isn't fallback paragraph loading rule")
            }

            let mut add_fb_block = |s: &str, start: usize, end: usize| -> Result<(), LoadError> {

                if let Some((fb_id, fallback_loading_rule)) = codex.fallback_paragraph_modifier() {

                    log::debug!("fallback rule {}:{:?} will be used to load:\n{}", fb_id, fallback_loading_rule, s);

                    let paragraph = fallback_loading_rule.load(s, codex, configuration, configuration_overlay.clone())?;

                    blocks.push(LoadBlock::new(
                        start, 
                        end,
                        LoadBlockContent::Paragraph(paragraph)
                    ));                
                }

                Ok(())
            };

            // assign fallback paragraph
            for heading_block in headings_blocks.iter_mut() {

                if heading_block.start() > last_position {

                    let s = &content[last_position..heading_block.start()];

                    add_fb_block(s, content_offset + last_position, content_offset + heading_block.start())?;
                }

                last_position = heading_block.end();

                heading_block.set_start(heading_block.start() + content_offset);
                heading_block.set_end(heading_block.end() + content_offset);
            }

            if content.len() > last_position {

                let s = &content[last_position..];

                add_fb_block(s, content_offset + last_position, content_offset + content.len())?;
            }

            blocks.append(&mut headings_blocks);

            return Ok(blocks);
        }
    }

    
}

impl Into<LoadBlockContent> for LoadBlock {
    fn into(self) -> LoadBlockContent {
        self.content
    }
}

impl TryInto<Box<dyn Paragraph>> for LoadBlock {
    type Error = String;

    fn try_into(self) -> Result<Box<dyn Paragraph>, Self::Error> {
        if let LoadBlockContent::Paragraph(p) = self.content {
            return Ok(p)
        }

        Err(String::from("this block doesn't contain a paragraph"))
    }
}

impl TryInto<Heading> for LoadBlock {
    type Error = String;

    fn try_into(self) -> Result<Heading, Self::Error> {
        if let LoadBlockContent::Heading(h) = self.content {
            return Ok(h)
        }

        Err(String::from("this block doesn't contain an heading"))
    }
}

impl TryInto<ChapterTag> for LoadBlock {
    type Error = String;

    fn try_into(self) -> Result<ChapterTag, Self::Error> {
        if let LoadBlockContent::ChapterTag(t) = self.content {
            return Ok(t)
        }

        Err(String::from("this block doesn't contain a chapter tag"))
    }
}


#[derive(Debug)]
pub enum LoadBlockContent {
    Paragraph(Box<dyn Paragraph>),
    Heading(Heading),
    ChapterTag(ChapterTag)
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

        let paragraphs = LoadBlock::load_from_str(content, &codex, &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap();

        assert_eq!(paragraphs.len(), 3)
    }
}