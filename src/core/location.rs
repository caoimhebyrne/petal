use super::position::Position;

#[derive(Debug, Copy, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Location {
    pub fn new(position: Position, length: usize) -> Location {
        Location {
            line: position.line,
            column: position.column,
            length,
        }
    }
}
