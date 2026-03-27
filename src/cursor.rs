use core::panic;
use std::{u32, usize};

use crate::data::row::Row;
use crate::data::table::Table;
use crate::trees::consts::{
    LEAF_NODE_CELL_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_MAX_CELLS, LEAF_NODE_VALUE_OFFSET,
};
use crate::trees::page_node::Page;
use crate::{data::row::ROW_SIZE, errors::PagerError};

#[derive(Debug)]
pub struct Cursor<'a> {
    pub table: &'a mut Table,
    pub curr_page_num: usize,
    pub cell_num: usize,
    pub at_table_end: bool,
}

impl<'a> Cursor<'a> {
    pub fn new(table: &'a mut Table) -> Self {
        let root_page_num = table.root_page_num;

        Self {
            table,
            curr_page_num: root_page_num,
            at_table_end: false,
            cell_num: 0,
        }
    }

    pub fn new_table_end(&mut self) {
        let mut root_node_data = match self.table.pager.get_page(self.curr_page_num) {
            Ok(&mut val) => val,
            Err(e) => panic!("{}", e),
        };

        let page = Page::new(&mut root_node_data);
        let num_cells = page.cell_count();

        self.cell_num = num_cells as usize;
        self.at_table_end = true;
    }

    pub fn advance(&mut self) {
        let page = Page::new(self.table.pager.get_page(self.curr_page_num).unwrap());

        self.curr_page_num += 1;
        if self.cell_num >= page.cell_count() as usize {
            self.at_table_end = true;
        }
    }

    pub fn curr_value(&mut self) -> Result<&mut [u8], PagerError> {
        let page_data = self.table.pager.get_page(self.curr_page_num)?;

        let start = LEAF_NODE_HEADER_SIZE + (self.cell_num * LEAF_NODE_CELL_SIZE);
        let value_offset = start + LEAF_NODE_VALUE_OFFSET;
        let end = value_offset + ROW_SIZE;

        Ok(&mut page_data[value_offset..end])
    }

    pub fn insert_leaf_page(
        &mut self,
        key: usize,
        row: &Row,
    ) -> Result<&mut [u8; 4096], PagerError> {
        let last_page_bytes = self.table.pager.get_page(self.curr_page_num)?;

        let num_cells = {
            let page = Page::new(last_page_bytes);
            page.cell_count() as usize
        };

        if num_cells >= LEAF_NODE_MAX_CELLS {
            todo!("implement node splitting")
        };

        if self.cell_num < num_cells {
            for i in (self.cell_num..num_cells).rev() {
                let dest_start = LEAF_NODE_HEADER_SIZE + ((i + 1) * LEAF_NODE_CELL_SIZE);
                let src_start = LEAF_NODE_HEADER_SIZE + (i * LEAF_NODE_CELL_SIZE);

                last_page_bytes.copy_within(src_start..src_start + LEAF_NODE_CELL_SIZE, dest_start);
            }
        }

        let cell_start = LEAF_NODE_HEADER_SIZE + (self.cell_num * LEAF_NODE_CELL_SIZE);
        let value_offset = cell_start + LEAF_NODE_VALUE_OFFSET;

        {
            let mut page = Page::new(last_page_bytes);

            page.set_cell_key(key, self.cell_num);
        }

        let destination = &mut last_page_bytes[value_offset..(value_offset + ROW_SIZE)];

        row.serialize(destination);

        {
            let mut page = Page::new(last_page_bytes);
            page.set_cell_count((num_cells + 1) as u32);
        }

        Ok(last_page_bytes)
    }
}
