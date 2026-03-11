use std::fs::File;

use crate::{data::table::PAGE_SIZE, errors::PagerError};

#[derive(Debug)]
pub struct Pager {
    pub file: File,
    pub file_length: u64,
    pub pages: Vec<Option<[u8; PAGE_SIZE]>>,
}

impl Pager {
    pub fn new(db_file_path: &str) -> Self {
        let file = File::open(db_file_path)
            .expect("failed to open database file. Please make sure it exists..");

        let file_length = file.metadata().unwrap().len();

        Self {
            file,
            file_length,
            pages: vec![None],
        }
    }

    pub fn get_page(&mut self, page_num: usize) -> Result<&mut [u8; PAGE_SIZE], PagerError> {
        let page = self
            .pages
            .get_mut(page_num)
            .ok_or(PagerError::PageNotFound {
                page_seeked: page_num,
            })?;

        let page = page.as_mut().ok_or(PagerError::PageNotFound {
            page_seeked: page_num,
        })?;

        Ok(page)
    }

    pub fn allocate_page(&mut self, page_num: usize) {
        if page_num >= self.pages.len() {
            self.pages.resize(page_num + 1, None);
        }

        self.pages[page_num] = Some([0; PAGE_SIZE])
    }
}
