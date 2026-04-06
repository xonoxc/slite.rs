use thiserror::Error;

#[derive(Debug, Error)]
pub enum InternalNodeError {
    #[error("Tried to access child_num {child_no} > num_keys {max_child_no_available}")]
    InvalidChildAccess {
        child_no: usize,
        max_child_no_available: usize,
    },
}
