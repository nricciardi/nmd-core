use getset::{CopyGetters, Getters, MutGetters, Setters};


/// Content with start and end
#[derive(Debug, Getters, CopyGetters, MutGetters, Setters)]
pub struct Span<T> {
    #[getset(get_copy = "pub", set = "pub")]
    start: usize,

    #[getset(get_copy = "pub", set = "pub")]
    end: usize,

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    content: T
}


impl<T> Span<T> {

    pub fn new(start: usize, end: usize, content: T) -> Self {
        Self {
            start,
            end,
            content
        }
    }

    pub fn add_offset(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
    }
}