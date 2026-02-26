#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub col: u8,
    pub row: u8,
}

impl Position {
    pub fn new(col: u8, row: u8) -> Self {
        Position { col, row }
    }

    pub fn from_index(index: usize, width: u8) -> Self {
        Position {
            col: (index % width as usize) as u8,
            row: (index / width as usize) as u8,
        }
    }

    pub fn to_index(&self, width: u8) -> usize {
        self.row as usize * width as usize + self.col as usize
    }

    pub fn is_valid(&self, width: u8, height: u8) -> bool {
        self.col < width && self.row < height
    }
}
