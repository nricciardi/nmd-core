use std::sync::{Arc, RwLock};
use build_html::Container;
use build_html::ContainerType;
use build_html::HtmlContainer;
use build_html::TableCell as HtmlTableCell;
use build_html::TableRow as HtmlTableRow;
use getset::{Getters, Setters};
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use crate::resource::table::TableCellAlignment;
use crate::{codex::Codex, compiler::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor, self_compile::SelfCompile, Compiler}, dossier::document::chapter::paragraph::ParagraphTrait, output_format::OutputFormat, resource::{resource_reference::ResourceReference, table::{Table, TableCell}}, utility::nmd_unique_identifier::NmdUniqueIdentifier};


pub type TableParagraphContentRow = Vec<Box<dyn ParagraphTrait>>;
pub type TableParagraphContent = Table<TableParagraphContentRow, TableParagraphContentRow, TableParagraphContentRow>;


#[derive(Debug, Getters, Setters)]
pub struct TableParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    content: TableParagraphContent,

    raw_id: Option<String>,

    raw_style: Option<String>,

    raw_caption: Option<String>,

    #[getset(set = "pub")]
    compiled_content: Option<CompilationResult>,

}

impl TableParagraph {

    pub fn new(raw_content: String, content: TableParagraphContent, raw_id: Option<String>, raw_style: Option<String>, raw_caption: Option<String>,) -> Self {
        Self {
            raw_content,
            content,
            raw_caption,
            raw_id,
            raw_style,
            nuid: None,
            compiled_content: None
        }
    }

    fn load_html_row(html_row: &mut HtmlTableRow, cells: &Vec<TableCell<String>>, codex: &Codex, compilation_configuration: &CompilationConfiguration) -> Result<(), CompilationError> {

        for cell in cells {
            match cell {
                TableCell::None => {

                    html_row.add_cell(
                        HtmlTableCell::new(build_html::TableCellType::Data)
                                    .with_attributes(vec![
                                        ("class", "table-cell table-empty-cell")
                                    ])
                                    .with_raw("")
                    );                       

                },
                TableCell::ContentCell { content, alignment } => {

                    let align_class = match alignment {
                        TableCellAlignment::Left => String::from("table-left-cell"),
                        TableCellAlignment::Center => String::from("table-center-cell"),
                        TableCellAlignment::Right => String::from("table-right-cell"),
                    };

                    html_row.add_cell(
                        HtmlTableCell::new(build_html::TableCellType::Data)
                                    .with_attributes(vec![
                                        ("class", format!("table-cell {}", align_class).as_str())
                                    ])
                                    .with_raw(content)
                    );       
                },
            }
        }

        Ok(())
    }

    fn standard_compile_html(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        let mut html_table_attrs: Vec<(String, String)> = vec![(String::from("class"), String::from("table"))];

        if let Some(ref id) = self.raw_id {

            html_table_attrs.push((String::from("id"), ResourceReference::of_internal(&id, compilation_configuration_overlay.read().unwrap().document_name().as_ref())?.build_without_internal_sharp()));
        }

        if let Some(ref style) = self.raw_style {
            html_table_attrs.push((String::from("style"), String::from(style.as_str())));
        }
    
        if let Some(ref nuid) = self.nuid {
            html_table_attrs.push((String::from("data-nuid"), nuid.clone()));
        }

        let mut html_table = build_html::Table::new().with_attributes(html_table_attrs);

        let mut compile_cells_fn = |cells: &mut Vec<TableCell<Vec<Box<dyn ParagraphTrait>>>>| -> Result<Vec<TableCell<String>>, CompilationError> {
            let mut result: Vec<TableCell<String>> = Vec::new();

            for cell in cells.iter_mut() {
                match cell {
                    TableCell::None => result.push(TableCell::None),
                    TableCell::ContentCell { content: paragraphs, alignment } => {

                        let mut compiled_content = String::new();

                        for paragraph in paragraphs {
                            paragraph.compile(
                                &OutputFormat::Html,
                                codex,
                                compilation_configuration,
                                compilation_configuration_overlay.clone()
                            )?;

                            compiled_content.push_str(&paragraph.compilation_result().as_ref().unwrap().content());
                        }

                        let cell = TableCell::ContentCell { content: compiled_content, alignment: alignment.clone() };

                        result.push(cell);
                    },
                }
            }

            Ok(result)
        };

        // ==== HEADER ====
        if let Some(ref mut header_cells) = self.content.header_mut() {

            html_table.add_thead_attributes(vec![
                                                ("class", "table-header")
                                            ]);

            let mut html_table_header = HtmlTableRow::new()
                                                    .with_attributes(vec![
                                                        ("class", "table-header-row")
                                                    ]);

            let new_header_cells = compile_cells_fn(header_cells)?;
            
            Self::load_html_row(&mut html_table_header, &new_header_cells, codex, &compilation_configuration).unwrap();

            html_table.add_custom_header_row(html_table_header);
        }

        // ==== BODY ====
        html_table = html_table.with_tbody_attributes(vec![
            ("class", "table-body")
        ]);

        for row in self.content.body_mut() {

            let mut html_body_row = HtmlTableRow::new()
                                                .with_attributes(vec![
                                                    ("class", "table-body-row")
                                                ]);

            let compiled_row = compile_cells_fn(row)?;

            Self::load_html_row(&mut html_body_row, &compiled_row, codex, &compilation_configuration).unwrap();

            html_table.add_custom_body_row(html_body_row);
        }

        // ==== FOOTER ====
        if let Some(ref mut footer_cells) = self.content.footer_mut() {

            html_table.add_tfoot_attributes(vec![
                ("class", "table-footer")
            ]);

            let mut html_table_footer = HtmlTableRow::new()
                                                .with_attributes(vec![
                                                    ("class", "table-footer-row")
                                                ]);

            let new_footer_cells = compile_cells_fn(footer_cells)?;

            Self::load_html_row(&mut html_table_footer, &new_footer_cells, codex, &compilation_configuration).unwrap();

            html_table.add_custom_footer_row(html_table_footer);
        }

        // ==== CAPTION ====
        if let Some(ref c) = self.raw_caption {

            let caption = Compiler::compile_str(
                c,
                &OutputFormat::Html,
                codex,
                compilation_configuration,
                compilation_configuration_overlay.clone()
            )?;

            let html_caption = Container::new(ContainerType::Div)
                                                .with_attributes(vec![
                                                    ("class", "table-caption")
                                                ])
                                                .with_raw(caption.content());

            html_table.add_caption(html_caption);
        }

        Ok(())
    }
}

impl SelfCompile for TableParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.standard_compile_html(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }
    }
}


impl CompilationResultAccessor for TableParagraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compiled_content
    }
}

impl ParagraphTrait for TableParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> &Option<NmdUniqueIdentifier> {
        &self.nuid
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}



#[cfg(test)]
mod test {
//     use std::sync::{Arc, RwLock};

//     use crate::{codex::{codex_configuration::CodexConfiguration, modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, compiler::{compilable::{Compilable, GenericCompilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_result_accessor::CompilationResultAccessor, compilation_rule::CompilationRule, Compiler}, dossier::document::Paragraph, output_format::OutputFormat, resource::table::{self, Table, TableCell, TableCellAlignment}};

//     use super::HtmlTableRule;

//     #[test]
//     fn head_body_foot() {
//         let nmd_table = r#"
// |                                                           | $x_1$ | $...$ | $x_n$ | $s_1$ | $...$ | $s_m$ | $a_1$ | $...$ |
// |-----------------------------------------------------------|:-----:|:-----:|:-----:|:-----:|:-----:|-------|-------|:-----:|
// | Riga avente $1$ nella colonna della variabile artificiale |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $1$  |  $0$  |
// |---|
// ||footer|        
// "#.trim();


//         let rule = HtmlTableRule::new();
//         let codex = Codex::of_html(CodexConfiguration::default());
//         let compilation_configuration = CompilationConfiguration::default();
//         let mut compilation_configuration_overlay = CompilationConfigurationOverLay::default();

//         compilation_configuration_overlay.set_document_name(Some("test".to_string()));

//         let compilable: Box<dyn Compilable> = Box::new(GenericCompilable::from(nmd_table.to_string()));

//         let outcome = rule.compile(&compilable, &OutputFormat::Html, &codex, &compilation_configuration, Arc::new(RwLock::new(compilation_configuration_overlay))).unwrap();
//         let outcome = outcome.content();

//         assert!(outcome.contains("<thead"));
//         assert!(outcome.contains("<tbody"));
//         assert!(outcome.contains("<tfoot"));
//     }

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