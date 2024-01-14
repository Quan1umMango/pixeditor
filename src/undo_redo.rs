#[derive(Clone)]
pub struct UndoRedoManager<T> {
    data_stream: Vec<T>,
    pointer: usize,
}

impl<T> UndoRedoManager<T> {
    pub fn new() -> Self {
        Self {
            data_stream: Vec::new(),
            pointer: 0,
        }
    }
    pub fn push(&mut self, item: T) {
        let i = self.pointer;
        if i < self.data_stream.len() {
            // If there are items after the current pointer, remove them
            self.data_stream.truncate(i);
        }
        self.data_stream.push(item);
        self.pointer = self.data_stream.len();
    }

    pub fn undo(&mut self) -> Option<&T> {
        if self.pointer > 0 {
            self.pointer -= 1;
            return self.data_stream.get(self.pointer);
        }
        None
    }

    pub fn redo(&mut self) -> Option<&T> {
        if self.pointer < self.data_stream.len() - 1 {
            self.pointer += 1;
            return self.data_stream.get(self.pointer);
        }
        None
    }
}
