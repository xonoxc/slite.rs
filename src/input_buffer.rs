#[derive(Debug)]
pub struct InputBuffer {
    pub buffer: String,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
