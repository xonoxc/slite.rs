use std::{u32, usize};

use crate::data::row::Row;
use crate::data::table::Table;
use crate::trees::consts::{
    LEAF_NODE_CELL_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_MAX_CELLS, LEAF_NODE_VALUE_OFFSET,
};
use crate::trees::node_type::NodeType;
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

    pub fn leaf_node_find(&mut self, page_num: usize, key: usize) {
        /*
         * this function performs a binary search
         * given leaf node and page number.
         * **/
        let page = Page::new(self.table.pager.get_page(page_num).unwrap());
        let num_cells = page.cell_count();

        self.curr_page_num = page_num;

        let mut min_index = 0;

        let mut one_past_max_index = num_cells;

        while one_past_max_index != min_index {
            let index = (min_index + one_past_max_index) / 2;
            let index_key = page.get_cell_key(index as usize) as usize;

            if key == index_key {
                self.cell_num = index as usize;
                return;
            } else if key < index_key {
                one_past_max_index = index;
            } else {
                min_index = index + 1;
            }
        }

        self.cell_num = min_index as usize;
    }

    pub fn table_find(&mut self, key: usize) {
        let root_page = Page::new(self.table.pager.get_page(self.table.root_page_num).unwrap());

        if root_page.get_node_type() == NodeType::NodeLeaf {
            self.leaf_node_find(self.table.root_page_num, key);
        } else {
            todo!("internal node searching not implemented yet");
        }
    }

    /*
    pub fn new_table_end(&mut self) {
        let mut root_node_data = match self.table.pager.get_page(self.curr_page_num) {
            Ok(&mut val) => val,
            Err(e) => panic!("{}", e),
        };

        let page = Page::new(&mut root_node_data);
        let num_cells = page.cell_count();

        self.cell_num = num_cells as usize;
        self.at_table_end = true;
    }*/

    pub fn advance(&mut self) {
        let page = Page::new(self.table.pager.get_page(self.curr_page_num).unwrap());

        if self.cell_num < page.cell_count() as usize {
            self.cell_num += 1;
            return;
        }

        let next_page_num = self.curr_page_num + 1;

        if self.table.pager.get_page(next_page_num).is_err() {
            self.at_table_end = true;
            return;
        }

        self.curr_page_num = next_page_num;
        self.cell_num = 0;
    }

    pub fn curr_value(&mut self) -> Result<&mut [u8], PagerError> {
        let page_data = self.table.pager.get_page(self.curr_page_num)?;

        let start = LEAF_NODE_HEADER_SIZE + (self.cell_num * LEAF_NODE_CELL_SIZE);
        let value_offset = start + LEAF_NODE_VALUE_OFFSET;
        let end = value_offset + ROW_SIZE;

        Ok(&mut page_data[value_offset..end])
    }

    fn get_unused_page(&self) -> usize {
        self.table.pager.num_pages
    }

    fn split_node_and_insert(&mut self, key: usize, row: &Row) {
        let new_page_num = self.get_unused_page();
        self.table.pager.allocate_page(new_page_num);

        let (old_page_bytes, new_page_bytes) = self
            .table
            .pager
            .get_two_pages(self.curr_page_num, new_page_num)
            .unwrap();

        let mut old_page = Page::new(old_page_bytes);
        let mut new_page = Page::new(new_page_bytes);

        let key_u32 = key as i32;

        let num_cells = old_page.cell_count() as usize;
        let total = num_cells + 1;

        let insert_pos = (0..num_cells)
            .find(|&i| old_page.get_cell_key(i) >= key_u32)
            .unwrap_or(num_cells);

        let split_index = total / 2;

        for i in (0..total).rev() {
            let (k, r) = if i == insert_pos {
                (key, row)
            } else if i > insert_pos {
                let old_i = i - 1;
                (
                    old_page.get_cell_key(old_i) as usize,
                    &old_page.get_cell_row(old_i),
                )
            } else {
                (old_page.get_cell_key(i) as usize, &old_page.get_cell_row(i))
            };

            if i >= split_index {
                let dest = i - split_index;
                new_page.set_cell_key(k, dest);
                new_page.write_cell_value(r, dest);
            } else {
                old_page.set_cell_key(k, i);
                old_page.write_cell_value(r, i);
            }
        }

        old_page.set_cell_count(split_index as u32);
        new_page.set_cell_count((total - split_index) as u32);

        let routing_key = new_page.get_cell_key(0);

        if key >= routing_key as usize {
            self.curr_page_num = new_page_num;
        }
    }

    pub fn insert_leaf_page(
        &mut self,
        key: usize,
        row: &Row,
    ) -> Result<&mut [u8; 4096], PagerError> {
        let num_cells = {
            let last_page_bytes = self.table.pager.get_page(self.curr_page_num)?;
            let page = Page::new(last_page_bytes);
            page.cell_count() as usize
        };

        if num_cells >= LEAF_NODE_MAX_CELLS {
            self.split_node_and_insert(key, row);
        };

        let num_cells = {
            let last_page_bytes = self.table.pager.get_page(self.curr_page_num)?;
            let page = Page::new(last_page_bytes);
            page.cell_count() as usize
        };

        let last_page_bytes = self.table.pager.get_page(self.curr_page_num)?;

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
