use crate::solver::Solver;
use crate::sudoku::{Puzzle, Sudoku};

use super::SolutionInfo;

type Bits = u32;
const ALL: Bits = 0x1ff;

type RowColSub = (usize, usize, usize);

#[derive(Clone, Default)]
pub struct BasicState {
    rows: [Bits; 9],
    cols: [Bits; 9],
    subs: [Bits; 9],
    todo: Vec<RowColSub>,
    num_todo: usize,
    guesses: usize,
    num_solutions: usize,
}

impl BasicState {
    pub fn new() -> Self {
        Self {
            rows: [ALL; 9],
            cols: [ALL; 9],
            subs: [ALL; 9],
            todo: vec![],
            num_todo: 0,
            guesses: 0,
            num_solutions: 0,
        }
    }

    fn setup(&mut self, puzzle: &Puzzle, solution: &mut Sudoku) -> bool {
        self.rows.fill(ALL);
        self.cols.fill(ALL);
        self.subs.fill(ALL);
        self.guesses = 0;
        self.num_solutions = 0;

        // Copy initial clues to the solution since our todo list won't include these cells.
        self.todo.clear();

        for row in 0..9 {
            for col in 0..9 {
                let cell = row * 9 + col;
                let sub = (row / 3) * 3 + col / 3;
                solution.grid[cell] = puzzle.grid[cell];
                if b'1' <= puzzle.grid[cell] && puzzle.grid[cell] <= b'9' {
                    // A given clue: clear availability bits for row, col, and box.
                    let value = 1u32 << (puzzle.grid[cell] as u32 - b'1' as u32);
                    if self.rows[row] & value != 0
                        && self.cols[col] & value != 0
                        && self.subs[sub] & value != 0
                    {
                        self.rows[row] ^= value;
                        self.cols[col] ^= value;
                        self.subs[sub] ^= value;
                    } else {
                        println!(
                            "Invalid puzzle: row: {}, col: {}, sub: {}, value: {}",
                            row, col, sub, value
                        );
                        return false;
                    }
                } else {
                    self.todo.push((row, col, sub));
                }
            }
        }
        self.num_todo = self.todo.len() - 1;
        true
    }

    fn mcv(&mut self, todo_index: usize) {
        let sublist = &mut self.todo[todo_index..];
        if let Some(min_element_index) = sublist
            .iter()
            .enumerate()
            .min_by_key(|&(_, &(row, col, sub))| {
                let candidates = self.rows[row] & self.cols[col] & self.subs[sub];
                candidates.count_ones()
            })
            .map(|(index, _)| index)
        {
            sublist.swap(0, min_element_index);
        }
    }
}

#[derive(Clone)]
pub struct SolverBasic {
    limit: usize,
    min_heuristic: bool,
}

impl SolverBasic {
    pub fn new(limit: usize, min_heuristic: bool) -> Self {
        Self {
            limit,
            min_heuristic,
        }
    }

    fn satisfy(&self, todo_index: usize, solution: &mut Sudoku, state: &mut BasicState) -> bool {
        if self.min_heuristic {
            state.mcv(todo_index);
        }

        let (row, col, sub) = state.todo[todo_index];

        let mut candidates = state.rows[row] & state.cols[col] & state.subs[sub];
        // println!("canditates: {}", candidates.count_ones());

        while candidates != 0 {
            let ci = candidates.trailing_zeros() as u8;
            let candidate = 1 << ci;

            // Only count assignment as a guess if there's more than one candidate.
            if candidates ^ candidate != 0 {
                state.guesses += 1;
            }

            // Clear the candidate from available candidate sets for row, col, box.
            state.rows[row] ^= candidate;
            state.cols[col] ^= candidate;
            state.subs[sub] ^= candidate;

            solution.grid[row * 9 + col] = b'1' + ci;
            // Recursively solve remaining cells and back out with the last solution.
            if todo_index < state.num_todo {
                self.satisfy(todo_index + 1, solution, state);
            } else {
                state.num_solutions += 1;
            }

            if state.num_solutions == self.limit {
                return true;
            }

            // Restore the candidate to available candidate sets for row, col, box.
            state.rows[row] ^= candidate;
            state.cols[col] ^= candidate;
            state.subs[sub] ^= candidate;

            candidates ^= candidate;
        }
        return false;
    }
}

impl Solver for SolverBasic {
    type State = BasicState;

    fn make_state(&self) -> Self::State {
        BasicState::default()
    }

    fn solve(&self, puzzle: &Puzzle, state: &mut Self::State) -> Option<SolutionInfo> {
        let mut solution = puzzle.sudoku();
        if state.setup(puzzle, &mut solution) && self.satisfy(0, &mut solution, state) {
            Some(SolutionInfo {
                sudoku: solution,
                guesses: state.guesses,
            })
        } else {
            None
        }
    }
}
