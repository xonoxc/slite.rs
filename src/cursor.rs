use crate::data::table::{ROWS_PER_PAGE, Table};
use crate::{data::row::ROW_SIZE, errors::PagerError};

#[derive(Debug)]
pub struct Cursor<'a> {
    pub table: &'a mut Table,
    pub row_num: usize,
    pub at_table_end: bool,
}

impl<'a> Cursor<'a> {
    pub fn new(table: &'a mut Table) -> Self {
        let at_table_end = table.rows == 0;

        Self {
            table,
            row_num: 0,
            at_table_end,
        }
    }

    pub fn new_table_end(table: &'a mut Table) -> Self {
        let row_num = table.rows;

        Self {
            table: table,
            row_num,
            at_table_end: false,
        }
    }

    pub fn advance(&mut self) {
        self.row_num += 1;
        if self.row_num >= self.table.rows {
            self.at_table_end = true
        }
    }

    pub fn curr_value(&mut self) -> Result<&mut [u8], PagerError> {
        let page_num = self.row_num / ROWS_PER_PAGE;

        if let Err(PagerError::PageNotFound { .. }) = self.table.pager.get_page(page_num) {
            self.table.pager.allocate_page(page_num);
        }

        let page = self.table.pager.get_page(page_num)?;

        let row_offset = self.row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        Ok(&mut page[byte_offset..byte_offset + ROW_SIZE])
    }
}
