use crossbeam_channel::{Receiver, Sender};
use std::thread;

use crate::{
    consts::N_CELLS,
    solver::{Solver, SolverBasic, Stats},
    sudoku::Sudoku,
};

pub fn start_workers(
    num_threads: usize,
    line_length: usize,
    read_rx: Receiver<(usize, Vec<u8>)>,
    write_tx: Sender<(usize, Vec<u8>)>,
) -> Vec<thread::JoinHandle<Stats>> {
    (0..num_threads)
        .map(|_| {
            let rx = read_rx.clone();
            let tx = write_tx.clone();
            thread::spawn(move || {
                let mut solver = SolverBasic::new(1, true);
                let mut stats = Stats::new();
                for (index, chunk) in rx {
                    let solved_chunk = process_chunk(line_length, &mut solver, &mut stats, chunk);
                    tx.send((index, solved_chunk))
                        .expect("Error sending chunk to writer");
                }
                stats
            })
        })
        .collect()
}

fn process_chunk<S: Solver>(
    line_length: usize,
    solver: &mut S,
    stats: &mut Stats,
    chunk: Vec<u8>,
) -> Vec<u8> {
    let mut solved_chunk = Vec::with_capacity(chunk.len() * 3);

    for puzzle_slice in chunk.chunks_exact(line_length) {
        // assert!(puzzle_slice[N_CELLS] == b'\n', "Invalid chunk format");

        let sudoku = Sudoku::new(puzzle_slice[..N_CELLS].try_into().unwrap());

        solved_chunk.extend_from_slice(sudoku.grid.as_ref());
        solved_chunk.push(b',');

        let solution = solver.solve(&sudoku).expect("Failed to solve sudoku");

        solved_chunk.extend_from_slice(solution.grid.as_ref());
        solved_chunk.push(b'\n');

        stats.puzzles += 1;
        if solver.guesses() == 0 {
            stats.no_guesses += 1;
        }
        stats.guesses += solver.guesses();
    }

    solved_chunk
}
