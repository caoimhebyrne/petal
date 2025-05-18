#[derive(Debug, Copy, Clone, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }
}
