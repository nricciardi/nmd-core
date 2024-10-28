use build_html::Container;
use build_html::ContainerType;
use build_html::Html;
use build_html::HtmlContainer;
use build_html::TableCell as HtmlTableCell;
use build_html::TableRow as HtmlTableRow;
use getset::{Getters, Setters};
use crate::compilable_text::CompilableText;
use crate::compilation::compilation_outcome::CompilationOutcome;
use crate::content_bundle::ContentBundle;
use crate::resource::table::TableCellAlignment;
use crate::{codex::Codex, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilable::Compilable}, dossier::document::chapter::paragraph::Paragraph, output_format::OutputFormat, resource::{resource_reference::ResourceReference, table::{Table, TableCell}}, utility::nmd_unique_identifier::NmdUniqueIdentifier};


pub type TableParagraphContent = Table<ContentBundle, ContentBundle, ContentBundle>;


#[derive(Debug, Getters, Setters)]
pub struct TableParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,

    #[getset(get = "pub", set = "pub")]
    content: TableParagraphContent,

    #[getset(get = "pub", set = "pub")]
    raw_id: Option<String>,

    #[getset(get = "pub", set = "pub")]
    styles: Option<String>,

    #[getset(get = "pub", set = "pub")]
    classes: Option<String>,

    #[getset(get = "pub", set = "pub")]
    raw_caption: Option<String>,
}

impl TableParagraph {

    pub fn new(raw_content: String, content: TableParagraphContent, raw_id: Option<String>, styles: Option<String>, classes: Option<String>, raw_caption: Option<String>,) -> Self {
        Self {
            raw_content,
            content,
            raw_caption,
            raw_id,
            styles,
            classes,
            nuid: None,
        }
    }

    fn load_html_row(html_row: &mut HtmlTableRow, cells: &Vec<TableCell<String>>, _codex: &Codex, _compilation_configuration: &CompilationConfiguration) -> Result<(), CompilationError> {

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

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        let mut classes = String::from("table");

        if let Some(c) = &self.classes {
            classes.push_str(" ");
            classes.push_str(c);
        }

        let mut html_table_attrs: Vec<(String, String)> = vec![(String::from("class"), classes)];

        if let Some(ref id) = self.raw_id {

            html_table_attrs.push((String::from("id"), ResourceReference::of_internal_from_without_sharp(&id, compilation_configuration_overlay.document_name().as_ref())?.build_without_internal_sharp()));
        }

        if let Some(ref style) = self.styles {
            html_table_attrs.push((String::from("style"), String::from(style.as_str())));
        }
    
        if let Some(ref nuid) = self.nuid {
            html_table_attrs.push((String::from("data-nuid"), nuid.clone()));
        }

        let mut html_table = build_html::Table::new().with_attributes(html_table_attrs);

        let compile_cells_fn = |cells: &mut Vec<TableCell<ContentBundle>>| -> Result<Vec<TableCell<String>>, CompilationError> {
            let mut result: Vec<TableCell<String>> = Vec::new();

            for cell in cells.iter_mut() {
                match cell {
                    TableCell::None => result.push(TableCell::None),
                    TableCell::ContentCell { content, alignment } => {

                        let compiled_content = String::from(content.compile(
                            &OutputFormat::Html,
                            codex,
                            compilation_configuration,
                            compilation_configuration_overlay.clone()
                        )?.content());

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

            let caption = CompilableText::from(c as &str).compile(
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

        Ok(html_table.to_html_string())
    }
}

impl Compilable for TableParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }
    }
}


impl Paragraph for TableParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
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

    use crate::{codex::Codex, compilation::compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, dossier::document::chapter::paragraph::{paragraph_loading_rule::table_paragraph_loading_rule::TableParagraphLoadingRule, Paragraph}, load::{LoadConfiguration, LoadConfigurationOverLay}, output_format::OutputFormat};

    fn load_table(nmd_text: &str, codex: &Codex) -> Box<dyn Paragraph> {

        let rule = TableParagraphLoadingRule::new();

        rule.load(nmd_text, &codex, &LoadConfiguration::default(), LoadConfigurationOverLay::default()).unwrap()

    }

    #[test]
    fn head_body_foot() {
        let nmd_text = r#"
|                                                           | $x_1$ | $...$ | $x_n$ | $s_1$ | $...$ | $s_m$ | $a_1$ | $...$ |
|-----------------------------------------------------------|:-----:|:-----:|:-----:|:-----:|:-----:|-------|-------|:-----:|
| Riga avente $1$ nella colonna della variabile artificiale |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $0$  |  $1$  |  $0$  |
|---|
||footer|        
"#.trim();

        let compilation_configuration = CompilationConfiguration::default();
        let mut compilation_configuration_overlay = CompilationConfigurationOverLay::default();
        let codex = Codex::of_html();

        compilation_configuration_overlay.set_document_name(Some("test".to_string()));

        let mut paragraph = load_table(nmd_text, &codex);
        
        paragraph.compile(&OutputFormat::Html, &codex, &compilation_configuration, compilation_configuration_overlay).unwrap();
        
        let outcome = paragraph.compiled_text().as_ref().unwrap().content();

        assert!(outcome.contains("<thead"));
        assert!(outcome.contains("<tbody"));
        assert!(outcome.contains("<tfoot"));
    }

    #[test]
    fn table_with_inner_image() {

        let nmd_text = concat!(
            "\n\n",
            "|h1|h2|\n",
            "|---|---|\n",
            "|**a**|![Simple image](https://en.wikipedia.org/wiki/Main_Page)|",
            "\n\n"
        );


        let compilation_configuration = CompilationConfiguration::default();
        let mut compilation_configuration_overlay = CompilationConfigurationOverLay::default();
        let codex = Codex::of_html();

        compilation_configuration_overlay.set_document_name(Some("test".to_string()));

        let mut paragraph = load_table(nmd_text, &codex);
        
        paragraph.compile(&OutputFormat::Html, &codex, &compilation_configuration, compilation_configuration_overlay).unwrap();
        
        let outcome = paragraph.compiled_text().as_ref().unwrap().content();

        assert!(outcome.contains("<img"))
    }
}