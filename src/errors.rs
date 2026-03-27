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

    #[error("error flushing data to the disk for page {page_num}")]
    FlushError { page_num: usize },

    #[error("[DB_INIT_ERROR]:: {cause}")]
    InitError { cause: String },
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("error parsing arguement {arg}")]
    ArgsPassError { arg: &'static str },
}
