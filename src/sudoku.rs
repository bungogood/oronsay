use crate::consts::N_CELLS;

pub struct Sudoku {
    pub grid: [u8; N_CELLS],
}

impl Sudoku {
    pub fn new(grid: [u8; N_CELLS]) -> Self {
        Self { grid }
    }

    pub fn solve(&mut self) -> bool {
        self.grid = [b'.'; N_CELLS];
        true
    }
}
