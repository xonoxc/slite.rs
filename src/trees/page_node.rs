use std::{u8, u32, usize};

use crate::{
    data::{row::Row, table::PAGE_SIZE},
    trees::{
        consts::{
            INTERNAL_NODE_CELL_SIZE, INTERNAL_NODE_CHILD_SIZE, INTERNAL_NODE_HEADER_SIZE,
            INTERNAL_NODE_KEY_SIZE, INTERNAL_NODE_NUM_KEYS_OFFSET, INTERNAL_NODE_NUM_KEYS_SIZE,
            INTERNAL_NODE_RIGHT_CHILD_OFFSET, INTERNAL_NODE_RIGHT_CHILD_SIZE, IS_ROOT_OFFSET,
            LEAF_NODE_CELL_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_NUM_CELLS_OFFSET,
            LEAF_NODE_NUM_CELLS_SIZE, LEAF_NODE_VALUE_OFFSET, NODE_TYPE_OFFSET, NODE_TYPE_SIZE,
        },
        errors::InternalNodeError,
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
                .expect("invalid num cells bytes"),
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

    pub fn get_cell_key(&self, cell_num: usize) -> i32 {
        i32::from_le_bytes(
            self.get_node_cell(cell_num)[..LEAF_NODE_VALUE_OFFSET]
                .try_into()
                .expect("invalid cell key bytes"),
        )
    }

    pub fn get_cell(&self, cell_num: usize) -> (i32, Row) {
        (self.get_cell_key(cell_num), self.get_cell_row(cell_num))
    }

    pub fn insert_cell(&mut self, key: usize, row: &Row, cell_num: usize) {
        self.set_cell_key(key, cell_num);
        self.write_cell_value(row, cell_num);
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
        NodeType::from_u8(&self.data[NODE_TYPE_OFFSET])
    }

    pub fn set_node_type(&mut self, node_type: NodeType) {
        self.data[NODE_TYPE_OFFSET..NODE_TYPE_OFFSET + NODE_TYPE_SIZE]
            .copy_from_slice(&[node_type as u8]);
    }

    /*
     *   INTERNAL NODE METHODS
     * **/
    pub fn internal_node_right_child(&self) -> &[u8] {
        &self.data[INTERNAL_NODE_RIGHT_CHILD_OFFSET
            ..INTERNAL_NODE_RIGHT_CHILD_OFFSET + INTERNAL_NODE_RIGHT_CHILD_SIZE]
    }

    pub fn internal_node_cell(&self, cell_num: usize) -> &[u8] {
        &self.data[INTERNAL_NODE_HEADER_SIZE..cell_num * INTERNAL_NODE_CELL_SIZE]
    }

    pub fn internal_node_key(&self, key_num: usize) -> u32 {
        u32::from_le_bytes(
            self.internal_node_cell(key_num)
                [INTERNAL_NODE_CHILD_SIZE..INTERNAL_NODE_CHILD_SIZE + INTERNAL_NODE_KEY_SIZE]
                .try_into()
                .expect("invalid cell bytes while key lookup"),
        )
    }

    pub fn internal_node_child(&self, child_num: usize) -> Result<&[u8], InternalNodeError> {
        let num_keys = self.internal_node_num_keys();

        if child_num > num_keys {
            return Err(InternalNodeError::InvalidChildAccess {
                child_no: child_num,
                max_child_no_available: num_keys,
            });
        }

        if child_num == num_keys {
            return Ok(self.internal_node_right_child());
        }

        Ok(self.internal_node_cell(child_num))
    }

    pub fn is_root_node(&self) -> bool {
        self.data[IS_ROOT_OFFSET] != 0
    }

    pub fn set_root_node(&mut self, is_root: bool) {
        self.data[IS_ROOT_OFFSET] = is_root as u8
    }

    pub fn internal_node_num_keys(&self) -> usize {
        usize::from_le_bytes(
            self.data[INTERNAL_NODE_NUM_KEYS_OFFSET
                ..INTERNAL_NODE_NUM_KEYS_OFFSET + INTERNAL_NODE_NUM_KEYS_SIZE]
                .try_into()
                .expect("invalid num keys bytes"),
        )
    }

    pub fn print_leaf_node(&self) {
        let num_cells = self.cell_count();

        for i in 0..num_cells {
            let key = self.get_cell_key(i as usize);
            println!("  - {} : {}", i, key);
        }
    }
}
