use crate::{data::row::ROW_SIZE, errors::PagerError, pager::Pager, trees::page_node::Page};

pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;

pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

pub const MAX_USERNAME_SIZE: usize = 32;
pub const MAX_EMAIL_SIZE: usize = 32;

#[derive(Debug)]
pub struct Table {
    pub pager: Pager,
    pub root_page_num: usize,
}

impl Table {
    /*
     * now the constructor will do some
     * more things
     * **/
    pub fn new(db_file_path: &str) -> Result<Self, PagerError> {
        let mut pager = Pager::new(db_file_path)?;
        let root_page_num = 0;

        if pager.num_pages == 0 {
            /*
             * new database file  initialize page 0 as a leaf node.
             * */
            let mut root_page = Page::new(pager.get_page(0)?);
            root_page.set_cell_count(0);
            root_page.set_root_node(true);
        }

        Ok(Self {
            pager,
            root_page_num,
        })
    }
}

pub const DB_FILE_PATH: &'static str = "test.db";
/*
*
* TESTS
* ***/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::Cursor;
    use crate::data::row::Row;
    use crate::errors::PagerError;

    fn cleanup(path: &str) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_table_new() {
        cleanup(DB_FILE_PATH);

        let table = Table::new(DB_FILE_PATH).unwrap();
        assert_eq!(table.root_page_num, 0);
    }

    #[test]
    fn test_get_row_slot() {
        cleanup(DB_FILE_PATH);

        let mut table = Table::new(DB_FILE_PATH).unwrap();
        let mut cursor = Cursor::new(&mut table);

        let slot = cursor.curr_value().unwrap();
        assert_eq!(slot.len(), ROW_SIZE);
    }

    #[test]
    fn test_insert_and_retrieve_row() {
        cleanup(DB_FILE_PATH);

        let mut table = Table::new(DB_FILE_PATH).unwrap();

        let row = Row {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        {
            let mut cursor = Cursor::new(&mut table);
            cursor.insert_leaf_page(1, &row).unwrap();
        }

        let mut retrieved = Row::new();

        {
            let mut cursor = Cursor::new(&mut table);
            let slot = cursor.curr_value().unwrap();
            retrieved.ingest_deserialized(slot);
        }

        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.username, "testuser");
        assert_eq!(retrieved.email, "test@example.com");
    }

    #[test]
    fn test_insert_multiple_rows() {
        cleanup(DB_FILE_PATH);

        let mut table = Table::new(DB_FILE_PATH).unwrap();

        for i in 0..5 {
            let row = Row {
                id: i as i32,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };

            let mut cursor = Cursor::new(&mut table);
            cursor.cell_num = i;
            cursor.insert_leaf_page(i as usize, &row).unwrap();
        }

        let mut cursor = Cursor::new(&mut table);

        for i in 0..5 {
            cursor.cell_num = i;

            let mut retrieved = Row::new();
            let slot = cursor.curr_value().unwrap();
            retrieved.ingest_deserialized(slot);

            assert_eq!(retrieved.id, i as i32);
            assert_eq!(retrieved.username, format!("user{}", i));
        }
    }

    #[test]
    fn test_table_max_rows_limit() {
        cleanup(DB_FILE_PATH);

        let mut table = Table::new(DB_FILE_PATH).unwrap();

        for i in 0..TABLE_MAX_ROWS {
            let row = Row {
                id: i as i32,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };

            let mut cursor = Cursor::new(&mut table);
            cursor.cell_num = i;
            cursor.insert_leaf_page(i, &row).unwrap();
        }

        let mut cursor = Cursor::new(&mut table);
        cursor.cell_num = TABLE_MAX_ROWS - 1;

        let mut retrieved = Row::new();
        let slot = cursor.curr_value().unwrap();
        retrieved.ingest_deserialized(slot);

        assert_eq!(retrieved.id, (TABLE_MAX_ROWS - 1) as i32);
    }

    #[test]
    fn test_flush_writes_to_disk() {
        let test_file = "test_flush.db";
        cleanup(test_file);

        let mut table = Table::new(test_file).unwrap();

        let row = Row {
            id: 42,
            username: "flushuser".to_string(),
            email: "flush@test.com".to_string(),
        };

        {
            let mut cursor = Cursor::new(&mut table);
            cursor.insert_leaf_page(42, &row).unwrap();
        }

        table.pager.flush(0).expect("flush should succeed");

        drop(table);

        let file_data = std::fs::read(test_file).expect("should read file");
        assert!(file_data.len() > 0);

        cleanup(test_file);
    }

    #[test]
    fn test_flush_nonexistent_page_returns_error() {
        let test_file = "test_flush_error.db";
        cleanup(test_file);

        let mut table = Table::new(test_file).unwrap();
        let result = table.pager.flush(99);

        assert!(result.is_err());
        if let Err(PagerError::FlushError { page_num }) = result {
            assert_eq!(page_num, 99);
        } else {
            panic!("expected FlushError");
        }

        cleanup(test_file);
    }

    #[test]
    fn test_table_drop_flushes_to_disk() {
        let test_file = "test_drop_flush.db";
        cleanup(test_file);

        {
            let mut table = Table::new(test_file).unwrap();

            let row = Row {
                id: 100,
                username: "dropuser".to_string(),
                email: "drop@test.com".to_string(),
            };

            let mut cursor = Cursor::new(&mut table);
            cursor.insert_leaf_page(100, &row).unwrap();
        }

        let file_data = std::fs::read(test_file).expect("should read file after drop");

        assert!(
            file_data.len() >= PAGE_SIZE,
            "file should have at least one page"
        );

        cleanup(test_file);
    }

    #[test]
    fn test_persistence_after_recreate() {
        let test_file = "test_persist.db";
        cleanup(test_file);

        {
            let mut table = Table::new(test_file).unwrap();

            for i in 0..3 {
                let row = Row {
                    id: i as i32 + 1,
                    username: format!("user{}", i),
                    email: format!("user{}@example.com", i),
                };

                let mut cursor = Cursor::new(&mut table);
                cursor.cell_num = i;
                cursor.insert_leaf_page(i, &row).unwrap();
            }
        }

        {
            let mut table = Table::new(test_file).unwrap();
            let mut cursor = Cursor::new(&mut table);

            for i in 0..3 {
                cursor.cell_num = i;

                let mut retrieved = Row::new();
                let slot = cursor.curr_value().unwrap();
                retrieved.ingest_deserialized(slot);

                assert_eq!(retrieved.id, i as i32 + 1);
                assert_eq!(retrieved.username, format!("user{}", i));
            }
        }

        cleanup(test_file);
    }
}

/*
*
* this is to write all pages to disk when pager goes out
*  of scope
* **/
impl Drop for Table {
    fn drop(&mut self) {
        let total_pages = self.pager.pages.len();

        for page in 0..total_pages {
            if self.pager.pages[page].is_some() {
                if let Err(e) = self.pager.flush(page) {
                    eprintln!("Failed to auto-flush page {}: {:?}", page, e);
                }
            }
        }
    }
}
