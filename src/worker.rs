use std::thread;

use crossbeam_channel::{Receiver, Sender};

struct Sudoku {
    puzzle: [u8; 81],
    solution: [u8; 81],
}

impl Sudoku {
    fn new(puzzle: [u8; 81]) -> Self {
        Self {
            puzzle,
            solution: puzzle,
        }
    }

    fn solve(&mut self) {
        // Solve the puzzle
        self.solution = self.puzzle;
    }
}

pub fn start_workers(
    num_threads: usize,
    read_rx: Receiver<(usize, Vec<u8>)>,
    write_tx: Sender<(usize, Vec<u8>)>,
) -> Vec<thread::JoinHandle<()>> {
    (0..num_threads)
        .map(|_| {
            let rx = read_rx.clone();
            let tx = write_tx.clone();
            thread::spawn(move || {
                for (index, chunk) in rx {
                    // Allocate a new vector to hold the processed chunk (duplicated Sudokus)

                    // its longer than 2 times the size of the chunk
                    let mut solved_chunk = Vec::with_capacity(chunk.len() * 3);

                    // Iterate over the chunk in steps of 82 bytes
                    for puzzle_slice in chunk.chunks_exact(82) {
                        if puzzle_slice[81] != b'\n' {
                            panic!("Not implemented header parsing");
                        }

                        let mut sudoku = Sudoku::new(puzzle_slice[..81].try_into().unwrap());
                        sudoku.solve();

                        solved_chunk.extend_from_slice(sudoku.puzzle.as_ref());
                        solved_chunk.push(b',');
                        solved_chunk.extend_from_slice(sudoku.solution.as_ref());
                        solved_chunk.push(b'\n');
                    }
                    tx.send((index, solved_chunk)).unwrap();
                }
            })
        })
        .collect()
}
