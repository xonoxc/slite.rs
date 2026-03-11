use thiserror::Error;

#[derive(Debug, Error)]
pub enum PagerError {
    #[error("page {page} is out of the bound (max pages per table: {max_allowed_pages})")]
    OutBoundSeek {
        page: usize,
        max_allowed_pages: usize,
    },

    #[error("page {page_seeked} not found")]
    PageNotFound { page_seeked: usize },
}
