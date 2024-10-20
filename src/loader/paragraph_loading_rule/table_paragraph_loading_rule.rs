use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;

use super::ParagraphLoadingRule;
use crate::{codex::{modifier::constants::{IDENTIFIER_PATTERN, STYLE_PATTERN}, Codex}, dossier::document::chapter::paragraph::{table_paragraph::{TableParagraph, TableParagraphContent, TableParagraphContentRow}, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError, Loader}, resource::table::{Table, TableCell, TableCellAlignment}, utility::text_utility};


/// (caption, id, styles, classes)
type TableMetadata = (Option<String>, Option<String>, Option<String>, Option<String>);

static EXTRACT_TABLE_METADATA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"(?:\[(.*)\])?(?:{})?(?:\{{\{{{}\}}\}})?", IDENTIFIER_PATTERN, STYLE_PATTERN)).unwrap());


#[derive(Debug)]
pub struct TableParagraphLoadingRule {

}

impl TableParagraphLoadingRule {

    pub fn new() -> Self {
        Self {}
    }
    
    fn extract_table_row_content_from_line(line: &str) -> Option<Vec<String>> {
        if line.trim().is_empty() {
            return None;
        }

        let line = line.trim_start();

        if !line.starts_with('|') {
            return None;
        }

        let line = &line[1..];      // remove first |

        let mut row: Vec<String> = Vec::new();

        let cells: Vec<&str> = line.split("|").collect();
        let cells_n = cells.len();
        for (index, cell) in cells.iter().enumerate() {

            if index == cells_n - 1 {
                break;
            }

            row.push(String::from(*cell));
        }

        Some(row)
    }

    fn extract_table_alignments_from_row(row: &Vec<String>) -> Option<Vec<TableCellAlignment>> {

        let mut alignments = vec![TableCellAlignment::default(); row.len()];

        for (index, cell) in row.iter().enumerate() {
            let cell = cell.trim();

            if cell.starts_with(":-") && cell.ends_with("-:") {
                alignments[index] = TableCellAlignment::Center;
                continue;
            }
            
            if cell.starts_with(":-") && cell.ends_with("-") {
                alignments[index] = TableCellAlignment::Left;
                continue;
            }
            
            if cell.starts_with("-") && cell.ends_with("-:") {
                alignments[index] = TableCellAlignment::Right;
                continue;
            }

            if cell.starts_with("-") && cell.ends_with("-") {
                alignments[index] = TableCellAlignment::default();
                continue;
            }

            return None;
        }

        Some(alignments)
    }

    fn build_row(row: &Vec<String>, alignments: &Vec<TableCellAlignment>, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Vec<TableCell<TableParagraphContentRow>>, LoadError> {

        let mut cells: Vec<TableCell<TableParagraphContentRow>> = Vec::new();

        for (index, cell) in row.iter().enumerate() {

            let mut cell = String::from(cell);

            if cell.is_empty() {

                cells.push(TableCell::None);

            } else {

                let mut align = alignments.get(index).unwrap_or(&TableCellAlignment::default()).clone();

                if cell.starts_with(":") && cell.ends_with(":") {
                    align = TableCellAlignment::Center;

                    cell.remove(0);
                    cell.remove(cell.len() - 1);
                }
                
                if cell.starts_with(":") && !cell.ends_with(":") {
                    align = TableCellAlignment::Left;

                    cell.remove(0);
                }

                if !cell.starts_with(":") && cell.ends_with(":") {
                    align = TableCellAlignment::Right;

                    cell.remove(cell.len() - 1);
                }

                let paragraph_blocks = Loader::load_paragraphs_from_str_with_workaround(&cell, codex, configuration, configuration_overlay.clone())?;

                let paragraphs: Vec<Box<dyn Paragraph>> = paragraph_blocks.into_par_iter().map(|block| TryInto::<Box<dyn Paragraph>>::try_into(block).unwrap()).collect();

                cells.push(TableCell::ContentCell { content: paragraphs, alignment: align});
            }
        }

        Ok(cells)
    }

    fn extract_table_metadata(&self, s: &str) -> TableMetadata {

        let captures = EXTRACT_TABLE_METADATA_REGEX.captures(s);

        if captures.is_none() {
            log::warn!("invalid table metadata: '{}'", s);
            return (None, None, None, None);
        }

        let captures = captures.unwrap();

        let mut caption: Option<String> = None;
        let mut id: Option<String> = None;
        let mut styles: Option<String> = None;
        let mut classes: Option<String> = None;

        if let Some(_caption) = captures.get(1) {
            caption = Some(_caption.as_str().to_string());
        }

        if let Some(_id) = captures.get(2) {
            id = Some(String::from(_id.as_str()));
        }

        if let Some(style) = captures.get(3) {

            (styles, classes) = text_utility::split_styles_and_classes(style.as_str());
        }

        (caption, id, styles, classes)
    }
}

impl ParagraphLoadingRule for TableParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {

        let mut table: TableParagraphContent = Table::new_empty();

        let lines = raw_content.trim().lines();
        let lines_n = lines.clone().count();

        let mut there_is_header: bool = false;
        let mut there_is_footer: bool = false;
        let mut max_row_len: usize = 0;
        let mut alignments: Option<Vec<TableCellAlignment>> = None;

        let mut id: Option<String> = None;
        let mut caption: Option<String> = None;
        let mut styles: Option<String> = None;
        let mut classes: Option<String> = None;

        for (index, line) in lines.enumerate() {

            // check if there are metadata
            let trim_line = line.trim_start();
            if trim_line.starts_with("[") || trim_line.starts_with("{") || trim_line.starts_with("#") {
                    
                (caption, id, styles, classes) = self.extract_table_metadata(trim_line);
            }

            let row = Self::extract_table_row_content_from_line(line);
    
            if row.is_none() {
                continue;
            }
    
            let row = row.unwrap();

            max_row_len = max_row_len.max(row.len());

            if alignments.is_none() {
                alignments = Some(vec![TableCellAlignment::default(); max_row_len])
            }

            if let Some(mut aligns) = Self::extract_table_alignments_from_row(&row) {

                if table.body().len() == 1 {
                    there_is_header = true;
                }

                if index == lines_n - 2 {
                    there_is_footer = true;
                }

                while aligns.len() < max_row_len {
                    aligns.push(TableCellAlignment::default());
                }

                alignments = Some(aligns);
                
                continue;
            }

            let row = Self::build_row(&row, alignments.as_ref().unwrap(), codex, configuration, configuration_overlay.clone())?;

            table.append_to_body(row);
        }

        if there_is_header {
            table.shift_first_body_row_to_header();
        }

        // check if there is footer
        if there_is_footer {
            table.shift_last_body_row_to_footer();
        }

        Ok(Box::new(TableParagraph::new(raw_content.to_string(), table, id, styles, classes, caption)))
    }
}



#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use indexmap::IndexMap;

    use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_paragraph_modifier::StandardParagraphModifier, Modifier}, Codex}, loader::{block::BlockContent, loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, paragraph_loading_rule::ParagraphLoadingRule, Loader}};

    use super::TableParagraphLoadingRule;

    fn codex() -> Codex {
        Codex::new(
            IndexMap::new(),
            IndexMap::from([
                (
                    String::from("table"),
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::Table)) as Box<dyn Modifier>
                )
            ]),
            HashMap::new(),
            HashMap::from([
                (
                    String::from("table"),
                    Box::new(TableParagraphLoadingRule::new()) as Box<dyn ParagraphLoadingRule>
                )
            ]),
            Some(StandardParagraphModifier::CommonParagraph.identifier())
        )
    }

    #[test]
    fn generic_loading() {
        let nmd_text = concat!(
            "\n\n",
            "|                | $x_1$ | $...$ | $x_n$ | $s_1$ | $...$ | $s_m$ | $a_1$ | $...$ |",
            "|----------------|:-----:|:-----:|:-----:|:-----:|:-----:|-------|-------|:-----:|",
            "| This is a line |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $1$  |  $0$  |",
            "|---|",
            "||footer|",
            "\n\n"
        );

        let codex = codex();
        
        let paragraphs = Loader::load_paragraphs_from_str_with_workaround(&nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();

        assert_eq!(paragraphs.len(), 1);

        if let BlockContent::Paragraph(p) = &paragraphs[0].content() {

            assert_eq!(p.raw_content(), nmd_text);

        }
    }

    #[test]
    fn load_table_with_metadata() {
        let nmd_text = concat!(
            "\n\n",
            "|                | $x_1$ | $...$ | $x_n$ | $s_1$ | $...$ | $s_m$ | $a_1$ | $...$ |\n",
            "|----------------|:-----:|:-----:|:-----:|:-----:|:-----:|-------|-------|:-----:|\n",
            "| This is a line |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $1$  |  $0$  |\n",
            "|---|\n",
            "||footer|\n",
            "[Caption]#table-id{{color:red;}}",
            "\n\n"
        );

        let codex = codex();
        
        let paragraphs = Loader::load_paragraphs_from_str_with_workaround(&nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();

        assert_eq!(paragraphs.len(), 1);

        if let BlockContent::Paragraph(p) = &paragraphs[0].content() {

            assert_eq!(p.raw_content(), nmd_text);

        }
    }

}

