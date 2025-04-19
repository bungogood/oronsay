use crate::sudoku::{Puzzle, Sudoku};

mod basic;

pub use basic::SolverBasic;

pub struct Stats {
    pub guesses: usize,
    pub no_guesses: usize,
    pub puzzles: usize,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            guesses: 0,
            no_guesses: 0,
            puzzles: 0,
        }
    }

    pub fn add(&mut self, other: &Stats) {
        self.guesses += other.guesses;
        self.no_guesses += other.no_guesses;
        self.puzzles += other.puzzles;
    }
}

pub trait Solver: Clone + Send + Sync + 'static {
    type State;

    fn make_state(&self) -> Self::State;
    fn solve(&self, puzzle: &Puzzle, state: &mut Self::State) -> Option<Sudoku>;
}
