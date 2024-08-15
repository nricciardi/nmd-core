use getset::{Getters, MutGetters, Setters};


#[derive(Debug, Clone, Default)]
pub enum TableCellAlignment {
    Left,
    #[default] Center,
    Right
}



#[derive(Debug, Clone, Default)]
pub enum TableCell<T> {
    #[default] None,
    ContentCell{content: T, alignment: TableCellAlignment}
}



#[derive(Debug, Clone, Getters, MutGetters, Setters)]
pub struct Table<H, B, F> {

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    header: Option<Vec<TableCell<H>>>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    body: Vec<Vec<TableCell<B>>>,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    footer: Option<Vec<TableCell<F>>>
}

impl<H, B, F> Table<H, B, F>
where B: Into<H> + Into<F> {

    pub fn new(header: Option<Vec<TableCell<H>>>, body: Vec<Vec<TableCell<B>>>, footer: Option<Vec<TableCell<F>>>) -> Self {
        Self {
            header,
            body,
            footer
        }
    }

    pub fn new_empty() -> Self {
        Self {
            header: None,
            body: Vec::new(),
            footer: None
        }
    }

    pub fn append_to_body(&mut self, row: Vec<TableCell<B>>) {
        
        self.body.push(row);
    }

    pub fn shift_first_body_row_to_header(&mut self) {

        let first_row = self.body.remove(0);

        let mut header: Vec<TableCell<H>> = Vec::new();

        for table_cell in first_row {

            match table_cell {
                TableCell::None => header.push(TableCell::None),
                TableCell::ContentCell { content, alignment } => {
                    header.push(TableCell::ContentCell {
                        content: Into::<H>::into(content),
                        alignment
                    })
                },
            }
        }

        self.header = Some(header);

    }

    pub fn shift_last_body_row_to_footer(&mut self) {

        let last_row = self.body.remove(self.body.len() - 1);

        let mut footer: Vec<TableCell<F>> = Vec::new();

        for table_cell in last_row {

            match table_cell {
                TableCell::None => footer.push(TableCell::None),
                TableCell::ContentCell { content, alignment } => {
                    footer.push(TableCell::ContentCell {
                        content: Into::<F>::into(content),
                        alignment
                    })
                },
            }
        }

        self.footer = Some(footer);

    }
}