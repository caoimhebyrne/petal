#![allow(clippy::should_implement_trait)]

pub struct Stream<T> {
    elements: Vec<T>,
    index: usize,
}

impl<T> Stream<T> {
    /// Creates a new [Stream] with elements from a [Vec].
    pub fn new(elements: Vec<T>) -> Self {
        Self { elements, index: 0 }
    }

    // Advances the stream by a certain [size].
    pub fn advance_by(&mut self, size: usize) {
        self.index += size;
    }

    /// Advances the stream by one, returning the element before the advance occurred.
    pub fn next(&mut self) -> Option<&T> {
        let element = self.elements.get(self.index);

        self.index += 1;

        element
    }

    /// Attempts to read the next element in the stream.
    /// Returns the element at the current position if there are elements left in the stream, otherwise [Option::None].
    pub fn peek(&self) -> Option<&T> {
        self.elements.get(self.index)
    }

    /// Returns whether the stream has any elements remaining.
    pub fn has_elements(&self) -> bool {
        self.index < self.elements.len()
    }
}
