pub const N_CELLS: usize = 81;

pub struct Puzzle<'a> {
    // pub grid: &'a [u8; N_CELLS],
    pub grid: &'a [u8],
}

impl<'a> Puzzle<'a> {
    // pub fn new(grid: &'a [u8; N_CELLS]) -> Self {
    pub fn new(grid: &'a [u8]) -> Self {
        Self { grid }
    }

    pub fn sudoku(&self) -> Sudoku {
        Sudoku::new(self.grid.try_into().unwrap())
    }
}

#[derive(Clone, Copy)]
pub struct Sudoku {
    pub grid: [u8; N_CELLS],
}

impl Sudoku {
    pub fn new(grid: [u8; N_CELLS]) -> Self {
        Self { grid }
    }

    pub fn clean(&self) -> Sudoku {
        let mut new_grid = [0u8; N_CELLS];
        for i in 0..N_CELLS {
            new_grid[i] = match self.grid[i] {
                b'1'..=b'9' => self.grid[i],
                _ => b'.',
            };
        }
        Sudoku::new(new_grid)
    }

    // need a function to convert the grid to a string
    pub fn to_string(&self) -> String {
        let clean = self.clean();
        clean.grid.iter().map(|&c| c as char).collect()
    }

    pub fn pretty(&self) -> String {
        let mut result = String::new();
        result.push_str("┌───────┬───────┬───────┐\n");
        for row in 0..9 {
            if row % 3 == 0 && row != 0 {
                result.push_str("├───────┼───────┼───────┤\n");
            }
            for col in 0..9 {
                if col % 3 == 0 {
                    result.push_str("│ ");
                }
                let ch = match self.grid[row * 9 + col] {
                    b'1'..=b'9' => self.grid[row * 9 + col] as char,
                    _ => '.',
                };
                result.push_str(&format!("{} ", ch));
            }
            result.push_str("│\n");
        }
        result.push_str("└───────┴───────┴───────┘\n");
        result
    }
}
