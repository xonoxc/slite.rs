use crate::data::{row::ROW_SIZE, table::PAGE_SIZE};
use std::mem;

pub const NODE_TYPE_SIZE: usize = 1;
pub const NODE_TYPE_OFFSET: usize = 0;

pub const IS_ROOT_SIZE: usize = 1;
pub const IS_ROOT_OFFSET: usize = NODE_TYPE_OFFSET + NODE_TYPE_SIZE;

pub const PARENT_POINTER_SIZE: usize = mem::size_of::<u32>();
pub const PARENT_POINTER_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;

pub const COMMON_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

/*
* leaf node config
* **/
pub const LEAF_NODE_NUM_CELLS_SIZE: usize = mem::size_of::<u32>();
pub const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_HEADER_SIZE;
pub const LEAF_NODE_HEADER_SIZE: usize = COMMON_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE;

/*
*
* body of leaf nodes
* **/

pub const LEAF_NODE_KEY_SIZE: usize = mem::size_of::<u32>();
pub const LEAF_NODE_KEY_OFFSET: usize = 0;

pub const LEAF_NODE_VALUE_SIZE: usize = ROW_SIZE;
pub const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_OFFSET + LEAF_NODE_KEY_SIZE;

pub const LEAF_NODE_CELL_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
pub const LEAF_NODE_SPACE_FOR_CELLS: usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;

pub const LEAF_NODE_MAX_CELLS: usize = LEAF_NODE_SPACE_FOR_CELLS / LEAF_NODE_CELL_SIZE;
