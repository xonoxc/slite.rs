#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    NodeInternal,
    NodeLeaf,
}

impl NodeType {
    pub fn from_u8(value: &u8) -> Self {
        match value {
            0 => NodeType::NodeInternal,
            1 => NodeType::NodeLeaf,
            _ => panic!("Unknown node type: {}", value),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            NodeType::NodeInternal => 0,
            NodeType::NodeLeaf => 1,
        }
    }
}
