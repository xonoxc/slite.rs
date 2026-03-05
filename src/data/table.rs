use crate::data::row::ROW_SIZE;

pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;

pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

#[derive(Debug)]
pub struct Table {
    pub rows: usize,
    pub pages: [Option<[u8; PAGE_SIZE]>; TABLE_MAX_PAGES],
}

impl Table {
    pub fn new() -> Self {
        Self {
            rows: 0,
            pages: [(); TABLE_MAX_PAGES].map(|_| None),
        }
    }

    pub fn get_row_slot(&mut self, row_num: usize) -> &mut [u8] {
        let page_num = row_num / ROWS_PER_PAGE;

        if self.pages[page_num].is_none() {
            self.pages[page_num] = Some([0; PAGE_SIZE]);
        }

        let page = self.pages[page_num].as_mut().unwrap();

        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        &mut page[byte_offset..byte_offset + ROW_SIZE]
    }
}
