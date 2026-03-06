use crate::data::row::ROW_SIZE;

pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;

pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

#[derive(Debug)]
pub struct Table {
    pub rows: usize,
    pub pages: Vec<Option<[u8; PAGE_SIZE]>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            rows: 0,
            pages: vec![None; TABLE_MAX_PAGES],
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

/*
*
* TESTS
* ***/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::row::Row;

    #[test]
    fn test_table_new() {
        let table = Table::new();
        assert_eq!(table.rows, 0);
    }

    #[test]
    fn test_get_row_slot() {
        let mut table = Table::new();
        let slot = table.get_row_slot(0);
        assert_eq!(slot.len(), ROW_SIZE);
    }

    #[test]
    fn test_insert_and_retrieve_row() {
        let mut table = Table::new();
        let row = Row {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let slot = table.get_row_slot(0);
        row.serialize(slot);
        table.rows = 1;

        let mut retrieved = Row::new();
        let slot = table.get_row_slot(0);
        retrieved.ingest_deserialized(slot);

        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.username, "testuser");
        assert_eq!(retrieved.email, "test@example.com");
    }

    #[test]
    fn test_insert_multiple_rows() {
        let mut table = Table::new();

        for i in 0..5 {
            let row = Row {
                id: i as i32,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };
            let slot = table.get_row_slot(i);
            row.serialize(slot);
            table.rows += 1;
        }

        assert_eq!(table.rows, 5);

        for i in 0..5 {
            let mut retrieved = Row::new();
            let slot = table.get_row_slot(i);
            retrieved.ingest_deserialized(slot);
            assert_eq!(retrieved.id, i as i32);
        }
    }

    #[test]
    fn test_table_max_rows_limit() {
        let mut table = Table::new();

        for i in 0..TABLE_MAX_ROWS {
            let row = Row {
                id: i as i32,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };
            let slot = table.get_row_slot(i);
            row.serialize(slot);
            table.rows += 1;
        }

        assert_eq!(table.rows, TABLE_MAX_ROWS);
    }
}
