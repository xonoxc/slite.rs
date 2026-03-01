#[derive(Debug)]
pub struct InputBuffer {
    pub buffer: Option<char>,
    pub buffer_length: i32,
    pub input_length: i32,
}

impl InputBuffer {
    pub fn new() -> Self {
        InputBuffer {
            buffer: None,
            buffer_length: 0,
            input_length: 0,
        }
    }
}
