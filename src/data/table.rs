use crate::{data::row::ROW_SIZE, pager::Pager};

pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;

pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

pub const MAX_USERNAME_SIZE: usize = 32;
pub const MAX_EMAIL_SIZE: usize = 32;

#[derive(Debug)]
pub struct Table {
    pub rows: usize,
    pub pager: Pager,
}

impl Table {
    /*
     * now the constructor will do some
     * more things
     * **/
    pub fn new(db_file_path: &str) -> Self {
        let pager = Pager::new(db_file_path);
        let rows = pager.file_length as usize / ROW_SIZE;

        Self { rows, pager }
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

    #[test]
    fn test_table_new() {
        let _ = std::fs::remove_file(DB_FILE_PATH);
        let table = Table::new(DB_FILE_PATH);
        assert_eq!(table.rows, 0);
    }

    #[test]
    fn test_get_row_slot() {
        let mut table = Table::new(DB_FILE_PATH);
        let mut cursor = Cursor::new(&mut table);
        let slot = cursor.curr_value().unwrap_or_else(|e| {
            panic!("error accessing slot: {}", e);
        });
        assert_eq!(slot.len(), ROW_SIZE);
    }

    #[test]
    fn test_insert_and_retrieve_row() {
        let mut table = Table::new(DB_FILE_PATH);
        let row = Row {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let mut cursor = Cursor::new(&mut table);
        let slot = cursor.curr_value().unwrap_or_else(|e| {
            panic!("error accessing slot: {}", e);
        });
        row.serialize(slot);
        table.rows = 1;

        let mut retrieved = Row::new();
        let mut cursor = Cursor::new(&mut table);
        let slot = cursor.curr_value().unwrap_or_else(|e| {
            panic!("error accessing slot: {}", e);
        });
        retrieved.ingest_deserialized(slot);

        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.username, "testuser");
        assert_eq!(retrieved.email, "test@example.com");
    }

    #[test]
    fn test_insert_multiple_rows() {
        let _ = std::fs::remove_file(DB_FILE_PATH);
        let mut table = Table::new(DB_FILE_PATH);

        for i in 0..5 {
            let row = Row {
                id: i as i32,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };
            {
                let mut cursor = Cursor::new(&mut table);
                cursor.row_num = i;
                let slot = cursor.curr_value().unwrap();
                row.serialize(slot);
            }
            table.rows += 1;
        }

        assert_eq!(table.rows, 5);

        let mut cursor = Cursor::new(&mut table);
        for i in 0..5 {
            cursor.row_num = i;
            let mut retrieved = Row::new();
            let slot = cursor.curr_value().unwrap_or_else(|e| {
                panic!("error accessing slot: {}", e);
            });
            retrieved.ingest_deserialized(slot);
            assert_eq!(retrieved.id, i as i32);
        }
    }

    #[test]
    fn test_table_max_rows_limit() {
        let _ = std::fs::remove_file(DB_FILE_PATH);
        let mut table = Table::new(DB_FILE_PATH);

        for i in 0..TABLE_MAX_ROWS {
            let row = Row {
                id: i as i32,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };
            {
                let mut cursor = Cursor::new(&mut table);
                let slot = cursor.curr_value().unwrap_or_else(|e| {
                    panic!("error accessing slot: {}", e);
                });
                row.serialize(slot);
            }
            table.rows += 1;
        }

        assert_eq!(table.rows, TABLE_MAX_ROWS);
    }

    #[test]
    fn test_flush_writes_to_disk() {
        let test_file = "test_flush.db";
        let _ = std::fs::remove_file(test_file);

        let mut table = Table::new(test_file);
        let row = Row {
            id: 42,
            username: "flushuser".to_string(),
            email: "flush@test.com".to_string(),
        };

        let mut cursor = Cursor::new(&mut table);
        let slot = cursor.curr_value().unwrap();
        row.serialize(slot);
        table.rows = 1;

        table.pager.flush(0).expect("flush should succeed");

        drop(table);

        let file_data = std::fs::read(test_file).expect("should read file");
        assert!(file_data.len() > 0, "file should have data");

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_flush_nonexistent_page_returns_error() {
        let test_file = "test_flush_error.db";
        let _ = std::fs::remove_file(test_file);

        let mut table = Table::new(test_file);
        let result = table.pager.flush(99);

        assert!(result.is_err());
        if let Err(PagerError::FlushError { page_num }) = result {
            assert_eq!(page_num, 99);
        } else {
            panic!("expected FlushError");
        }

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_table_drop_flushes_to_disk() {
        let test_file = "test_drop_flush.db";
        let _ = std::fs::remove_file(test_file);

        {
            let mut table = Table::new(test_file);
            let row = Row {
                id: 100,
                username: "dropuser".to_string(),
                email: "drop@test.com".to_string(),
            };

            let mut cursor = Cursor::new(&mut table);
            let slot = cursor.curr_value().unwrap();
            row.serialize(slot);
            table.rows = 1;
        }

        let file_data = std::fs::read(test_file).expect("should read file after drop");
        assert!(
            file_data.len() >= PAGE_SIZE,
            "file should have at least one page"
        );

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_persistence_after_recreate() {
        let test_file = "test_persist.db";
        let _ = std::fs::remove_file(test_file);

        {
            let mut table = Table::new(test_file);
            for i in 0..3 {
                let row = Row {
                    id: i as i32 + 1,
                    username: format!("user{}", i),
                    email: format!("user{}@example.com", i),
                };
                {
                    let mut cursor = Cursor::new(&mut table);
                    cursor.row_num = i;
                    let slot = cursor.curr_value().unwrap();
                    row.serialize(slot);
                }
                table.rows += 1;
            }
        }

        {
            let mut table = Table::new(test_file);
            let mut cursor = Cursor::new(&mut table);
            for i in 0..3 {
                cursor.row_num = i;
                let mut retrieved = Row::new();
                let slot = cursor.curr_value().unwrap();
                retrieved.ingest_deserialized(slot);
                assert_eq!(retrieved.id, i as i32 + 1);
                assert_eq!(retrieved.username, format!("user{}", i));
            }
        }

        let _ = std::fs::remove_file(test_file);
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

        /*
         * partial page handling at the end
         * **/
        let additional_rows = self.rows % ROWS_PER_PAGE;
        if additional_rows > 0 {
            if self.pager.get_page(total_pages).is_ok() {
                if let Err(e) = self.pager.flush(total_pages) {
                    eprintln!("Failed to auto-flush addtional_rows : {:?}", e);
                }
            }
        }
    }
}
