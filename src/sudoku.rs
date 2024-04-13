use crate::consts::N_CELLS;

#[derive(Clone, Copy)]
pub struct Sudoku {
    pub grid: [u8; N_CELLS],
}

impl Sudoku {
    pub fn new(grid: [u8; N_CELLS]) -> Self {
        Self { grid }
    }
}
