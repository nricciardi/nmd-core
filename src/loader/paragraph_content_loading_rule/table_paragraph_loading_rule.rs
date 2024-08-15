use once_cell::sync::Lazy;
use regex::Regex;

use super::ParagraphContentLoadingRule;
use crate::{codex::{modifier::constants::IDENTIFIER_PATTERN, Codex}, dossier::document::chapter::paragraph::{replacement_rule_paragraph::ReplacementRuleParagraph, paragraph_content::ParagraphContent, table_paragraph::{TableParagraph, TableParagraphContent, TableParagraphContentRow}, ParagraphTrait}, loader::{loader_configuration::LoaderConfiguration, LoadError, Loader}, resource::table::{Table, TableCell, TableCellAlignment}};


/// (caption, id, style)
type TableMetadata = (Option<String>, Option<String>, Option<String>);

static EXTRACT_TABLE_METADATA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!(r"(?:\[(.*)\])?(?:{})?(?:\{{(.*)\}})?", IDENTIFIER_PATTERN)).unwrap());


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

    fn build_row(row: &Vec<String>, alignments: &Vec<TableCellAlignment>, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Vec<TableCell<TableParagraphContentRow>>, LoadError> {

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

                let content = cell;

                let content = Loader::load_paragraphs_from_str(&content, codex, configuration)?;

                cells.push(TableCell::ContentCell { content, alignment: align});
            }
        }

        Ok(cells)
    }

    fn extract_table_metadata(&self, s: &str) -> TableMetadata {

        let captures = EXTRACT_TABLE_METADATA_REGEX.captures(s);

        if captures.is_none() {
            log::warn!("invalid table metadata: '{}'", s);
            return (None, None, None);
        }

        let captures = captures.unwrap();

        let mut caption: Option<String> = None;
        let mut id: Option<String> = None;
        let mut style: Option<String> = None;

        if let Some(_caption) = captures.get(1) {
            caption = Some(_caption.as_str().to_string());
        }

        if let Some(_id) = captures.get(2) {
            id = Some(String::from(_id.as_str()));
        }

        if let Some(_style) = captures.get(3) {
            style = Some(_style.as_str().to_string());
        }

        (caption, id, style)
    }
}

impl ParagraphContentLoadingRule for TableParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration) -> Result<Box<dyn ParagraphTrait>, LoadError> {

        let mut table: TableParagraphContent = Table::new_empty();

        let lines = raw_content.trim().lines();
        let lines_n = lines.clone().count();

        let mut there_is_header: bool = false;
        let mut there_is_footer: bool = false;
        let mut max_row_len: usize = 0;
        let mut alignments: Option<Vec<TableCellAlignment>> = None;

        let mut id: Option<String> = None;
        let mut caption: Option<String> = None;
        let mut style: Option<String> = None;

        for (index, line) in lines.enumerate() {

            // check if there are metadata
            let trim_line = line.trim_start();
            if trim_line.starts_with("[") || trim_line.starts_with("{") || trim_line.starts_with("#") {
                    
                (caption, id, style) = self.extract_table_metadata(trim_line);
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

            let row = Self::build_row(&row, alignments.as_ref().unwrap(), codex, configuration)?;

            table.append_to_body(row);
        }

        if there_is_header {
            table.shift_first_body_row_to_header();
        }

        // check if there is footer
        if there_is_footer {
            table.shift_last_body_row_to_footer();
        }

        // TODO: table metadata

        Ok(Box::new(TableParagraph::new(raw_content.to_string(), table, id, style, caption)))
    }
}



#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, sync::{Arc, RwLock}};

    use crate::{codex::{modifier::{base_modifier::BaseModifier, standard_paragraph_modifier::StandardParagraphModifier, Modifier}, Codex}, compiler::{compilable::{Compilable, GenericCompilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_result_accessor::CompilationResultAccessor, compilation_rule::CompilationRule, Compiler}, dossier::document::Paragraph, loader::{loader_configuration::LoaderConfiguration, paragraph_content_loading_rule::ParagraphContentLoadingRule, Loader}, output_format::OutputFormat, resource::table::{self, Table, TableCell, TableCellAlignment}};

    use super::TableParagraphLoadingRule;

    fn codex() -> Codex {
        Codex::new(
            BTreeMap::new(),
            BTreeMap::from([
                (
                    String::from("table"),
                    Box::new(Into::<BaseModifier>::into(StandardParagraphModifier::Table)) as Box<dyn Modifier>
                )
            ]),
            BTreeMap::new(),
            BTreeMap::from([
                (
                    String::from("table"),
                    Box::new(TableParagraphLoadingRule::new()) as Box<dyn ParagraphContentLoadingRule>
                )
            ]),
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
        
        let paragraphs = Loader::load_paragraphs_from_str(&nmd_text, &codex, &LoaderConfiguration::default()).unwrap();

        assert_eq!(paragraphs.len(), 1);

        let table_paragraph = &paragraphs[0];

        assert_eq!(table_paragraph.raw_content(), nmd_text);
    }

    #[test]
    fn table_with_inner_image() {

        todo!()

        // let nmd_text = concat!(
        //     "\n\n",
        //     "|h1|h2|\n",
        //     "|---|---|\n",
        //     "|**a**|![Simple image](https://en.wikipedia.org/wiki/Main_Page)|",
        //     "\n\n"
        // );

        // let rule = HtmlTableRule::new();
        // let codex = Codex::of_html(CodexConfiguration::default());
        // let compilation_configuration = CompilationConfiguration::default();
        // let mut compilation_configuration_overlay = CompilationConfigurationOverLay::default();

        // compilation_configuration_overlay.set_document_name(Some("test".to_string()));

        // let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(nmd_text.to_string()));

        // let outcome = rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(compilation_configuration_overlay))).unwrap();
        // let outcome = outcome.content();

        // let expected_result = HtmlTableRule::build_html_table(
        //     None,
        //     None,
        //     None,
        //     None,
        //     Table::new(
        //         Some(vec![
        //             TableCell::ContentCell {
        //                 content: "h1".to_string(),
        //                 alignment: TableCellAlignment::Center
        //             },
        //             TableCell::ContentCell {
        //                 content: "h2".to_string(),
        //                 alignment: TableCellAlignment::Center
        //             },
        //         ]),
        //         vec![
        //             vec![
        //                 TableCell::ContentCell {
        //                     content: Compiler::compile_str(
        //                         "**a**",
        //                         &OutputFormat::Html,
        //                         &codex,
        //                         &CompilationConfiguration::default(),
        //                         Arc::new(RwLock::new(CompilationConfigurationOverLay::default()))
        //                     ).unwrap().content(),
        //                     alignment: TableCellAlignment::Center
        //                 },
        //                 TableCell::ContentCell {
        //                     content: {

        //                         let mut p = Paragraph::new(
        //                             "![Simple image](https://en.wikipedia.org/wiki/Main_Page)".to_string(),
        //                             StandardParagraphModifier::Image.identifier()
        //                         );

        //                         let mut conf_over = CompilationConfigurationOverLay::default();

        //                         conf_over.set_document_name(Some("test".to_string()));

        //                         Compiler::compile_paragraph(
        //                             &mut p,
        //                             &OutputFormat::Html,
        //                             &codex,
        //                             &CompilationConfiguration::default(),
        //                             Arc::new(RwLock::new(conf_over))
        //                         ).unwrap();

        //                         let r = p.compilation_result().clone().unwrap().content();

        //                         r
        //                     },
        //                     alignment: TableCellAlignment::Center
        //                 },
        //             ]
        //         ],
        //         None
        //     ),
        //     &codex,
        //     &CompilationConfiguration::default()
        // );

        // assert_eq!(outcome, expected_result);
    }
}

