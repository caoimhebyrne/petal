use super::position::Position;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Location {
    pub fn between(start: Position, end: Position) -> Self {
        Self {
            line: start.line,
            column: start.column,
            length: end.column - start.column,
        }
    }
}

impl From<Position> for Location {
    fn from(value: Position) -> Self {
        Self {
            line: value.line,
            column: value.column,
            length: 1,
        }
    }
}
