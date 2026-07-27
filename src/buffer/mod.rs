pub trait Buffer {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn insert(&mut self, offset: usize, text: &str);

    fn slice(&self, start: usize, end: usize) -> &str;

    fn delete(&mut self, start: usize, end: usize);

    fn chunks(&self, start: usize, end: usize) -> std::iter::Once<&str>;
}

pub struct GapBuffer {
    text: String,
}

impl Buffer for GapBuffer {
    fn len(&self) -> usize {
        self.text.len()
    }

    fn insert(&mut self, offset: usize, text: &str) {
        self.text.insert_str(offset, text);
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.text[start..end]
    }

    fn delete(&mut self, start: usize, end: usize) {
        self.text.drain(start..end);
    }

    fn chunks(&self, start: usize, end: usize) -> std::iter::Once<&str> {
        std::iter::once(&self.text[start..end])
    }
}

impl GapBuffer {
    pub fn new(text: &str) -> GapBuffer {
        GapBuffer {
            text: text.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::{Buffer, GapBuffer};

    #[test]
    fn buffer_len_empty() {
        let buf = GapBuffer::new("");
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn buffer_is_empty() {
        let buf = GapBuffer::new("");
        assert!(buf.is_empty());
    }

    #[test]
    fn buffer_insert() {
        let mut buf = GapBuffer::new("world!");
        buf.insert(0, "hello, ");
        assert_eq!(buf.len(), 13);
    }

    #[test]
    fn buffer_insert_at_start() {
        let mut buf = GapBuffer::new("world!");
        buf.insert(0, "hello, ");
        assert_eq!(buf.slice(0, 13), "hello, world!");
    }

    #[test]
    fn buffer_delete_range() {
        let mut buf = GapBuffer::new("hello, world!");
        buf.delete(4, 10);
        assert_eq!(buf.slice(0, 7), "hellld!");
        assert_eq!(buf.len(), 7);
    }

    #[test]
    fn buffer_chunks_returns_text() {
        let buf = GapBuffer::new("hello world");
        let result: String = buf.chunks(0, 5).collect();
        assert_eq!(result, "hello");
    }
}
