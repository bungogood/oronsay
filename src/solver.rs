use crate::sudoku::{Puzzle, Sudoku};

mod basic;

pub use basic::SolverBasic;

pub struct SolutionInfo {
    pub sudoku: Sudoku,
    pub guesses: usize,
}

pub trait Solver: Clone + Send + Sync + 'static {
    type State;

    fn make_state(&self) -> Self::State;
    fn solve(&self, puzzle: &Puzzle, state: &mut Self::State) -> Option<SolutionInfo>;
}
