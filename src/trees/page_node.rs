use std::usize;

use crate::{
    data::{row::Row, table::PAGE_SIZE},
    trees::{
        consts::{
            LEAF_NODE_CELL_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_NUM_CELLS_OFFSET,
            LEAF_NODE_NUM_CELLS_SIZE, LEAF_NODE_VALUE_OFFSET, NODE_TYPE_OFFSET, NODE_TYPE_SIZE,
        },
        node_type::NodeType,
    },
};

#[derive(Debug)]
pub struct Page<'a> {
    pub data: &'a mut [u8; PAGE_SIZE],
}

impl<'a> Page<'a> {
    pub fn new(data: &'a mut [u8; PAGE_SIZE]) -> Self {
        Self { data }
    }

    pub fn cell_count(&self) -> u32 {
        u32::from_le_bytes(
            self.data[LEAF_NODE_NUM_CELLS_OFFSET..][..LEAF_NODE_NUM_CELLS_SIZE]
                .try_into()
                .unwrap(),
        )
    }

    pub fn set_cell_count(&mut self, count: u32) {
        self.data[LEAF_NODE_NUM_CELLS_OFFSET..][..LEAF_NODE_NUM_CELLS_SIZE]
            .copy_from_slice(&count.to_le_bytes());
    }

    pub fn get_node_cell(&self, cell_num: usize) -> &[u8] {
        let start = LEAF_NODE_HEADER_SIZE + (cell_num * LEAF_NODE_CELL_SIZE);
        let end = start + LEAF_NODE_CELL_SIZE;

        &self.data[start..end]
    }

    pub fn get_node_cell_mut(&mut self, cell_num: usize) -> &mut [u8] {
        let start = LEAF_NODE_HEADER_SIZE + (cell_num * LEAF_NODE_CELL_SIZE);
        let end = start + LEAF_NODE_CELL_SIZE;

        &mut self.data[start..end]
    }

    pub fn get_mutable_node_cell(&mut self, cell_num: usize) -> &mut [u8] {
        let start = LEAF_NODE_HEADER_SIZE + (cell_num * LEAF_NODE_CELL_SIZE);
        let end = start + LEAF_NODE_CELL_SIZE;

        &mut self.data[start..end]
    }

    pub fn get_cell_key(&self, cell_num: usize) -> i32 {
        i32::from_le_bytes(
            self.get_node_cell(cell_num)[..LEAF_NODE_VALUE_OFFSET]
                .try_into()
                .unwrap(),
        )
    }

    pub fn set_cell_key(&mut self, key: usize, cell_num: usize) {
        self.get_node_cell_mut(cell_num)[..LEAF_NODE_VALUE_OFFSET]
            .copy_from_slice(&key.to_le_bytes());
    }

    pub fn get_cell_value(&self, cell_num: usize) -> &[u8] {
        &self.get_node_cell(cell_num)[LEAF_NODE_VALUE_OFFSET..]
    }

    pub fn write_cell_value(&mut self, value: &Row, cell_num: usize) {
        value.serialize(&mut self.get_node_cell_mut(cell_num)[..LEAF_NODE_VALUE_OFFSET]);
    }

    pub fn get_cell_row(&self, cell_num: usize) -> Row {
        let mut new_row = Row::new();
        new_row.ingest_deserialized(&self.get_node_cell(cell_num)[LEAF_NODE_VALUE_OFFSET..]);
        new_row
    }

    pub fn get_node_type(&self) -> NodeType {
        let raw_data =
            &self.data[NODE_TYPE_OFFSET..NODE_TYPE_OFFSET + NODE_TYPE_SIZE][NODE_TYPE_OFFSET];
        NodeType::from_u8(raw_data)
    }

    pub fn print_leaf_node(&self) {
        let num_cells = self.cell_count();

        for i in 0..num_cells {
            let key = self.get_cell_key(i as usize);
            println!("  - {} : {}", i, key);
        }
    }
}
